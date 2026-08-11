use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct BootstrapMessage {
    pub server_socket: String,
    pub ipc_socket: String,
}

impl<'a> BootstrapMessage {
    pub fn serialize(&self) -> Result<Vec<u8>> {
        match serde_json::to_string(&self) {
            Ok(mut s) => {
                // Add a trailing newline
                s.push('\n');
                Ok(s.into_bytes())
            },
            Err(e) => Err(anyhow!("Failed to serialize JSON: {:?}", e))
        }
    }

    pub fn deserialize(json: &'a str) -> Result<Self> {
        match serde_json::from_str::<Self>(json) {
            Ok(s) => Ok(s),
            Err(e) => Err(anyhow!("Failed to deserialize JSON: {:?}", e)),
        }
    }
}