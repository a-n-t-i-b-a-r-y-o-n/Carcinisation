use anyhow::anyhow;

use crate::protocol::message::{
    ProtocolMessage,
    start_server::{
        StartServerCommand,
        StartServerResponse
    }
};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
use tokio::process::Child;

pub struct CommunicationService;

impl CommunicationService {

    pub async fn initialize_plugin_server(process: &mut Child, port: u32) -> anyhow::Result<()> {

        println!("Attempting to init plugin server");

        // Server startup command
        let command = StartServerCommand {
            port,
        };

        let message = &command.serialize()?;

        println!("Serialized command");

        let stdin = process.stdin.as_mut().unwrap();
        let stdout = process.stdout.take().unwrap();

        _ = stdin.write_all(message.as_bytes()).await;
        _ = stdin.write(b"\n").await;
        stdin.flush().await?;

        println!("Sent command");

        println!("Waiting for response...");

        let mut reader = tokio::io::BufReader::new(stdout);
        let mut line = String::new();
        _ = reader.read_line(&mut line).await?;

        println!("Server response: {:?}", &line);

        match StartServerResponse::deserialize(&line) {
            Ok(response) => {
                if response.success {
                    println!("Plugin server started on port {}!", port);
                    Ok(())
                }
                else {
                    Err(anyhow!("Server failed to start: {}", &response.message))
                }
            }
            Err(e) => Err(anyhow!("Failed to deserialize server response: {:?}", e))
        }
    }

}