use std::convert::Infallible;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::process::Command;
use std::task::{Context, Poll};
use std::thread::sleep;
use std::time::Duration;
use anyhow::{anyhow, Result};
use bytes::Bytes;
use prost_reflect::{DescriptorError, DescriptorPool};
use tempfile::TempDir;
use tonic::body::Body;
use tonic::codegen::BoxFuture;
use tonic::codegen::http::{Request, Response};
use tonic::service::AxumBody;
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
    name: String,
    methods: Vec<PluginMethod>,
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
                            name: method.full_name().to_string(),
                            request: PluginRequest { name: method.input().full_name().to_string() },
                            response: PluginResponse { name: method.output().full_name().to_string() },
                        }
                    )

                }
                
                // Add to the list of returned services
                services.push(
                    PluginService {
                        name: service.full_name().to_string(),
                        methods,
                    }
                );
            }
        }

        Ok(services)

    }

}

/// Service name for request routing
impl DynamicNamedService for PluginService {
    fn name(&self) -> &str { &self.name }
}

impl Service<Request<AxumBody>> for PluginService {
    type Response = Response<Body>;
    type Error = Infallible;
    type Future = BoxFuture<Self::Response, Self::Error>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<std::result::Result<(), Self::Error>> {
        // TODO: Actually poll?
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<AxumBody>) -> Self::Future {

        // Copy request path URI to owned
        let full_path = req.uri().path().to_owned();

        // Copy required service info to owned
        let service_name = self.name.to_owned();
        let methods = self.methods.to_owned();

        // Remove the leading service name and make the result owned
        let service_prefix = format!("/{}/", service_name);
        let method_name = full_path.trim_start_matches(&service_prefix).to_owned();

        let future = async move {

            tokio::time::sleep(Duration::from_secs(1)).await;

            if let Some(m) = methods.iter().filter(|m| m.name == method_name).next() {
                // TODO: Invoke the plugin method

            }

            // TODO: Handle unmatched routes

            Ok(Response::new(Body::empty()))

        };


        Box::pin(future)

    }
}