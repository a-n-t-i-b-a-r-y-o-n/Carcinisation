use serde::{Deserialize, Serialize};
use crate::protocol::message::ProtocolMessage;

/// Command to start the plugin server
#[derive(Deserialize, Serialize)]
pub struct StartServerCommand {
    pub port: u32
}

/// Response from the plugin server
#[derive(Deserialize, Serialize)]
pub struct StartServerResponse {
    pub success: bool,
    pub message: String,
}


impl ProtocolMessage<'_> for StartServerCommand { }
impl ProtocolMessage<'_> for StartServerResponse { }