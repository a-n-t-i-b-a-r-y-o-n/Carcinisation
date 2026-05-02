use std::convert::Infallible;
use std::future::poll_fn;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::FromStr;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;
use anyhow::{anyhow, Result};
use bytes::Bytes;
use parking_lot::RwLock;
use prost_reflect::DescriptorPool;
use tempfile::TempDir;
use tonic::body::Body;
use tonic::codegen::BoxFuture;
use tonic::codegen::http::{Request, Response};
use tonic::service::AxumBody;
use tonic::{Code, Status};
use tonic::transport::{Channel, Endpoint, Uri};
use tower_service::Service;
use crate::server::dynamic_named_service::DynamicNamedService;

#[derive(Clone)]
pub struct PluginRequest {
    name: String,
}

#[derive(Clone)]
pub struct PluginResponse {
    name: String,
}

#[derive(Clone)]
pub struct PluginMethod {
    name: String,
    request: PluginRequest,
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
    server: RwLock<Option<Uri>>,
}

/// RPC Service implemented in a plugin
impl PluginService {

    /// Generate a descriptor set from reading a given proto file
    fn generate_descriptor_set(input_proto: &PathBuf, scratch_dir: &TempDir) -> Result<PathBuf> {

        // Get containing directory for input (required for protoc)
        let input_dir = input_proto.parent().unwrap_or(Path::new("/"));

        // Output descriptor set
        let output_file = scratch_dir.path().join("descriptor_set.bin");

        let child = Command::new("protoc")
            .args([
                "--include_imports",
                &format!("--descriptor_set_out={}", output_file.display()),
                &format!("--proto_path={}", input_dir.display()),
                &format!("{}", input_proto.display())
            ])
            .spawn()?
            .wait()?;

        if !child.success() {
            Err(anyhow!("Error: protoc exited code {:#?}", child.code()))
        } else {
            Ok(output_file)
        }
    }

    /// Read a raw DescriptorSet file from disk
    fn parse_descriptor_set(file_path: &PathBuf) -> Result<DescriptorPool> {
        if !file_path.exists() {
            return Err(anyhow!("Cannot read nonexistent descriptor set: {}", &file_path.display()))
        }

        let raw = std::fs::read(&file_path)?;

        match DescriptorPool::decode(Bytes::from(raw)) {
            Ok(dp) => Ok(dp),
            Err(e) => Err(anyhow!(e))
        }
    }

    /// Parse services from a proto file
    pub fn new_from_proto(proto_path: &PathBuf) -> Result<Vec<Self>> {

        // Get absolute path to input proto file
        let input_proto = proto_path.canonicalize()?;

        // Create temp dir to write outputs
        let scratch_dir = tempfile::tempdir()?;

        // Generate descriptor set in temp dir
        let descriptor_set = Self::generate_descriptor_set(&input_proto, &scratch_dir)?;

        // Parse the descriptor set file into a pool
        let descriptor_pool = Self::parse_descriptor_set(&descriptor_set)?;

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
                            server: RwLock::new(None),
                        })
                    }
                );
            }
        }

        Ok(services)

    }

    /// Set the plugin server port number
    pub(crate) fn bind_to_server(&mut self, port_number: u32) -> Result<()> {
        // TODO: Use HTTPS
        let uri = Uri::from_str(&format!("http://[::1]:{port_number}"))?;

        *self.inner.server.write() = Some(uri.clone());

        // Build service endpoint object
        let endpoint = Endpoint::from(uri)
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(30))
            .tcp_nodelay(true);

        // Lazy-built channel to talk to end point
        let channel = endpoint.connect_lazy();

        // Update local channel handle
        *self.inner.channel.write() = Some(channel);

        Ok(())
    }
}

/// Service name for request routing
impl DynamicNamedService for PluginService {
    fn name(&self) -> &str { &self.inner.name }
}

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
                let status = Self::map_status(
                    Status::new(
                        Code::Unimplemented,
                        format!(
                            "Unknown method {} on service {}.",
                            method_name,
                            inner.name,
                        ),
                    )
                );

                return Ok(status);
            }

            // Get a handle on the channel
            let mut channel = match inner.channel.read().clone() {
                None => {
                    let status = Self::map_status(
                        Status::new(
                            Code::Unavailable,
                            format!("Service {} plugin not yet started", inner.name),
                        )
                    );

                    return Ok(status);
                }
                Some(c) => c
            };

            // Rewrite the request URI
            *req.uri_mut() = match Self::extract_path_and_query(req.uri()) {
                Err(e) => {
                    let status = Self::map_status(
                        Status::new(
                            Code::Internal,
                            format!("Failed to map URI: {e}"),
                        )
                    );

                    return Ok(status);
                }
                Ok(uri) => uri,
            };

            // Poll until plugin server is ready
            if let Err(e) = poll_fn(|cx| Channel::poll_ready(&mut channel, cx)).await {
                return Ok(Self::map_status(Status::from_error(Box::new(e))));
            }

            // Proxy request to plugin and return response
            match channel.call(req).await {
                Ok(response) => Ok(response),
                Err(e) => Ok(Self::map_status(Status::from_error(Box::new(e))))
            }
        };

        // Return the Boxed Pinned handler
        Box::pin(future)

    }
}

impl PluginService {
    /// Map a Tonic [`Status`] to an HTTP response
    fn map_status(status: Status) -> Response<Body> {
        status.into_http()
    }

    /// Map a request from an [`AxumBody`] to a Tonic [`Body`]
    fn map_request(request: Request<AxumBody>) -> Request<Body> {
        let (parts, body) = request.into_parts();
        Request::from_parts(parts, Body::new(body))
    }

    /// Extract just the path and query (i.e. not the `http://host:port` part)
    fn extract_path_and_query(uri: &Uri) -> Result<Uri> {
        Ok(
            uri.path_and_query()
                .map(|p| p.as_str())
                .unwrap_or("/")
                .parse()?
        )
    }
}