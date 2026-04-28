use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

// Message types
pub mod start_server;

pub trait ProtocolMessage<'a>: Deserialize<'a> + Serialize {
    fn serialize(&self) -> Result<String> {
        match serde_json::to_string(&self) {
            Ok(s) => Ok(s),
            Err(e) => Err(anyhow!("Failed to serialize JSON: {:?}", e))
        }
    }

    fn deserialize(json: &'a str) -> Result<Self> {
        match serde_json::from_str::<Self>(json) {
            Ok(s) => Ok(s),
            Err(e) => Err(anyhow!("Failed to deserialize JSON: {:?}", e)),
        }
    }
}
