use tonic::transport::Server;
use crate::server::DynamicRoutesExt;

pub mod plugin;
pub mod server;


// Re-export required parts of dependencies
pub use morphology;
