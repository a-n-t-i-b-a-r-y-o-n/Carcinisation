use std::ops::Deref;
use std::path::Path;
use tonic::service::Routes;
use tonic::transport::Server;
use homoplasy::plugin::plugin_manager::PluginManager;
use homoplasy::plugin::plugin_server_ext::PluginRoutesExt;
use homoplasy::plugin::plugin_service::PluginService;
use homoplasy::server::DynamicRoutesExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let addr = "[::1]:50052".parse()?;
    
    let plugin_dir = "/media/medium/Projects/muonium/source/experiments/rust/plugin_test_3/plugins";

    println!("[-] Loading plugins from \"{}\"...", plugin_dir);

    let plugin_count = PluginManager::load_plugins(&plugin_dir).await?;
    if plugin_count == 0 {
        println!("[-] No plugins loaded.")
    }

    println!("[-] Starting plugins...");
    
    PluginManager::start_plugins().await;

    Server::builder()
        .add_plugin_routes()
        .serve(addr)
        .await?;

    Ok(())
}