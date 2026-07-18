use tonic::transport::Server;
use homoplasy::plugin::plugin_manager::PluginManager;
use homoplasy::plugin::plugin_server_ext::PluginRoutesExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let port = 50052;
    let addr = format!("[::1]:{}", port).parse()?;
    
    let plugin_dir_1 = "/media/medium/Projects/muonium/source/experiments/rust/plugin_test_4/plugins/speaker/target/debug/plugin_package";
    let plugin_dir_2 = "/media/medium/Projects/muonium/source/experiments/rust/plugin_test_4/plugins/microphone/target/debug/plugin_package";
    let plugin_dir_3 = "/media/medium/Projects/muonium/source/experiments/rust/plugin_test_4/plugins/vad/target/debug/plugin_package";
    let plugin_dir_4 = "/media/medium/Projects/muonium/source/experiments/rust/plugin_test_4/plugins/wakeword/target/debug/plugin_package";
    let plugin_dir_5 = "/media/medium/Projects/muonium/source/experiments/rust/plugin_test_4/plugins/asr/target/debug/plugin_package";
    let plugin_dir_6 = "/media/medium/Projects/muonium/source/experiments/rust/plugin_test_4/plugins/tts/target/debug/plugin_package";
    let plugin_dir_7 = "/media/medium/Projects/muonium/source/experiments/rust/plugin_test_4/plugins/llm/target/debug/plugin_package";

    let mut plugin_count = 0;
    for plugin_dir in [plugin_dir_1, plugin_dir_2, plugin_dir_3, plugin_dir_4, plugin_dir_5, plugin_dir_6, plugin_dir_7] {
        println!("[+] Loading plugins from \"{}\"...", plugin_dir);
        plugin_count += PluginManager::load_plugins(&plugin_dir).await?;
    }

    if plugin_count == 0 {
        println!("[-] No plugins loaded.")
    }

    println!("[+] Starting plugins...");
    
    PluginManager::start_plugins().await?;

    let server = Server::builder()
        .add_plugin_routes()
        .serve(addr);

    println!("[+] Server started on port {}", port);

    server.await?;

    Ok(())
}