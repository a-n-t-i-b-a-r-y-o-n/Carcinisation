mod cli;

use clap::Parser;
use futures_util::future::{join3, join_all};
use tonic::transport::Server;
use homoplasy::plugin::plugin_manager::PluginManager;
use homoplasy::plugin::plugin_server_ext::PluginRoutesExt;
use crate::cli::Cli;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse CLI args
    let args = Cli::parse();

    // Set the server address
    let port = args.port_number;
    let addr = format!("[::]:{}", port).parse()?;

    // Load each specified plugin
    let mut plugin_count = 0;
    for plugin_dir in args.plugin_paths {
        println!("[-] Loading plugins from \"{}\"...", plugin_dir.display());
        plugin_count += PluginManager::load_plugins(&plugin_dir).await?;
    }

    if plugin_count == 0 {
        println!("[-] No plugins loaded.")
    }

    println!("[+] Starting IPC server...");

    // Start the plugin IPC server
    let ipc_server = PluginManager::start_ipc().await?;

    println!("[+] Starting plugins...");

    // Start each plugin
    let plugin_handles = PluginManager::start_plugins().await?;

    // Start the proxy server
    let proxy_server = Server::builder()
        .add_plugin_routes()
        .serve(addr);

    println!("[+] Server started on port {}", port);

    // Combine all the tasks and futures
    _ = join3(proxy_server, ipc_server, join_all(plugin_handles)).await;

    Ok(())
}
