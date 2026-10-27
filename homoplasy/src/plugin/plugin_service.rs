use std::convert::Infallible;
use std::error::Error;
use std::ffi::OsStr;
use std::future::poll_fn;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;
use anyhow::Result;
use hyper_util::rt::TokioIo;
use parking_lot::RwLock;
use tokio::net::UnixStream;
use tonic::body::Body;
use tonic::codegen::BoxFuture;
use tonic::codegen::http::{Request, Response};
use tonic::service::AxumBody;
use tonic::{Code, Status};
use tonic::transport::{Channel, Endpoint, Uri};
use tower::service_fn;
use tower_service::Service;
use crate::plugin::file_descriptor_set::generate_descriptor_pool;
use crate::server::Named;

#[derive(Clone)]
pub struct PluginRequest {
    #[allow(dead_code)]
    name: String,
}

#[derive(Clone)]
pub struct PluginResponse {
    #[allow(dead_code)]
    name: String,
}

#[derive(Clone)]
pub struct PluginMethod {
    name: String,
    #[allow(dead_code)]
    request: PluginRequest,
    #[allow(dead_code)]
    response: PluginResponse,
}

#[derive(Clone)]
pub struct PluginService {
    inner: Arc<PluginServiceInner>
}

struct PluginServiceInner {
    name: String,
    methods: RwLock<Vec<PluginMethod>>,
    channel: RwLock<Option<Channel>>,
}

/// RPC Service implemented in a plugin
impl PluginService {

    /// Parse services from a proto file
    pub fn new_from_proto(proto_path: &PathBuf) -> Result<Vec<Self>> {

        // Get absolute path to input proto file
        let input_proto = proto_path.canonicalize()?;

        // Generate descriptors for the proto file
        let descriptor_pool = generate_descriptor_pool(&input_proto)?;

        // Build list of plugin services
        let mut services = vec!();
        for file in descriptor_pool.files() {
            
            for service in file.services() {

                // Build list of methods in this service
                let mut methods = vec!();
                for method in service.methods() {

                    // Get full method information
                    methods.push(
                        PluginMethod {
                            name: method.name().to_string(),
                            request: PluginRequest { name: method.input().full_name().to_string() },
                            response: PluginResponse { name: method.output().full_name().to_string() },
                        }
                    )

                }
                
                // Add to the list of returned services
                services.push(
                    PluginService {
                        inner: Arc::new(PluginServiceInner {
                            name: service.full_name().to_string(),
                            methods: RwLock::new(methods),
                            channel: RwLock::new(None),
                        })
                    }
                );
            }
        }

        Ok(services)

    }

    /// Set the plugin server port number
    pub(crate) fn bind_to_server<S>(&mut self, socket_file: S) -> Result<()>
    where
        S: AsRef<OsStr> + AsRef<Path> + Send + Clone
    {
        // Convert to owned PathBuf
        let socket_path: PathBuf = AsRef::<Path>::as_ref(&socket_file).to_path_buf();

        // TODO: Use TLS?
        // TODO: Lower proxy connection timeouts?
        // Build service endpoint object
        let endpoint = Endpoint::from_static("http://uds.localhost")
            .connect_timeout(Duration::from_secs(30))
            .timeout(Duration::from_secs(30));

        // Lazily built channel to talk to endpoint
        let channel = endpoint.connect_with_connector_lazy(service_fn(move |_: Uri| {
            let socket = socket_path.clone();
            async move {
                let stream = UnixStream::connect(socket).await?;
                Ok::<_, std::io::Error>(TokioIo::new(stream))
            }
        }));

        // Update local channel handle
        *self.inner.channel.write() = Some(channel);

        Ok(())
    }
}

/// Service name for request routing
impl Named for PluginService {
    fn name(&self) -> &str { &self.inner.name }
}

/// Make each plugin service a usable Tonic service
impl Service<Request<AxumBody>> for PluginService {
    type Response = Response<Body>;
    type Error = Infallible;
    type Future = BoxFuture<Self::Response, Self::Error>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<std::result::Result<(), Self::Error>> {
        // Get channel handle or return pending
        let mut channel = match self.inner.channel.read().clone() {
            None => return Poll::Pending,
            Some(c) => c
        };

        // Poll, assuming pending on error
        match channel.poll_ready(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(result) => {
                match result {
                    Ok(_) => Poll::Ready(Ok(())),
                    Err(e) => {
                        println!("Failed to poll channel: {e}");
                        Poll::Pending
                    }
                }
            }
        }
    }

    fn call(&mut self, req: Request<AxumBody>) -> Self::Future {
        // Get handle on self
        let inner = self.inner.clone();

        // Method handler future
        let future = async move {

            // Copy request path URI to owned
            let path = req.uri().path().to_owned();

            // Map the request like a generated client would
            let mut req = Self::map_request(req);

            // Copy required service info to owned
            let service_name = inner.name.to_owned();

            // Remove the leading service name and make the result owned
            let service_prefix = format!("/{}/", service_name);
            let method_name = path.trim_start_matches(&service_prefix).to_owned();

            // Handle unimplemented methods
            if !inner.methods.read().iter().any(|m| m.name == method_name) {
                let status = Status::new(
                    Code::Unimplemented,
                    format!(
                        "Unknown method {} on service {}.",
                        method_name,
                        inner.name,
                    ),
                ).into_http();

                return Ok(status);
            }

            // Get a handle on the channel
            let mut channel = match inner.channel.read().clone() {
                None => {
                    let status = Status::new(
                        Code::Unavailable,
                        format!("Service {} plugin not yet started", inner.name),
                    ).into_http();

                    return Ok(status);
                }
                Some(c) => c
            };
            
            // Extract request URI path and query
            let path_and_query = req.uri().path_and_query()
                .map(|p| p.as_str())
                .unwrap_or("/")
                .parse();

            // Rewrite the request URI
            *req.uri_mut() = match path_and_query {
                Ok(uri) => uri,
                Err(e) => {
                    // Return failure
                    return Ok(Self::map_error(e))
                }
            };

            // Try to poll the server until ready
            if let Err(e) = poll_fn(|cx| Channel::poll_ready(&mut channel, cx)).await {
                return Ok(Self::map_error(e));
            }

            // Proxy request to plugin and return response
            match channel.call(req).await {
                Ok(response) => Ok(response),
                Err(e) => Ok(Self::map_error(e))
            }
        };

        // Return the Boxed Pinned handler
        Box::pin(future)

    }
}

impl PluginService {
    /// Map a generic error to an HTTP response
    fn map_error<E>(error: E) -> Response<Body>
    where
        E: Error + Send + Sync + 'static
    {
        Status::from_error(Box::new(error)).into_http()
    }

    /// Map a request from an [`AxumBody`] to a Tonic [`Body`]
    fn map_request(request: Request<AxumBody>) -> Request<Body> {
        let (parts, body) = request.into_parts();
        Request::from_parts(parts, Body::new(body))
    }
}