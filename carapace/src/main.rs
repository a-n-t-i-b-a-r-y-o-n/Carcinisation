use futures_util::future::{join3, join_all};
use tonic::transport::Server;
use homoplasy::plugin::plugin_manager::PluginManager;
use homoplasy::plugin::plugin_server_ext::PluginRoutesExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let port = 50052;
    let addr = format!("[::]:{}", port).parse()?;

    let plugin_dir_1 = "/media/medium/Projects/muonium/source/experiments/rust/plugin_test_4/carcinisation/carapace/plugins_uds/";

    let mut plugin_count = 0;
    for plugin_dir in [plugin_dir_1] {
        println!("[+] Loading plugins from \"{}\"...", plugin_dir);
        plugin_count += PluginManager::load_plugins(&plugin_dir).await?;
    }

    if plugin_count == 0 {
        println!("[-] No plugins loaded.")
    }

    println!("[+] Starting IPC server...");

    let ipc_server = PluginManager::start_ipc().await?;

    println!("[+] Starting plugins...");
    
    let plugin_handles = PluginManager::start_plugins().await?;

    let proxy_server = Server::builder()
        .add_plugin_routes()
        .serve(addr);

    println!("[+] Server started on port {}", port);

    // Combine all the tasks and futures
    _ = join3(proxy_server, ipc_server, join_all(plugin_handles)).await;

    Ok(())
}
