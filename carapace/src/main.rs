use tonic::service::Routes;
use tonic::transport::Server;
use homoplasy::plugin::plugin_manager::PluginManager;
use homoplasy::server::DynamicRoutesExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let addr = "[::1]:50052".parse()?;

    let mut manager = PluginManager { };
    manager.init().await;

    let services = manager.get_services();

    Server::builder()
        .add_dynamic_routes(services)
        .serve(addr)
        .await?;

    Ok(())
}