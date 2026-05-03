use tonic::transport::Server;
use tonic::transport::server::Router;
use crate::plugin::plugin_manager::PluginManager;
use crate::server::{DynamicRoutesExt, Named};

/// Extension trait for [`Server`] allowing plugin route creation
pub trait PluginRoutesExt {
    fn add_plugin_routes(&mut self) -> Router;
}

impl PluginRoutesExt for Server {
    fn add_plugin_routes(&mut self) -> Router {
        // Get all plugins
        let plugins = PluginManager::get_plugins();

        // Get all services from all plugins
        let mut services = plugins.iter()
            .flat_map(|p| p.get_services())
            .map(|s| s.to_owned())
            .collect::<Vec<_>>();

        // De-duplicate services by name, just in case
        services.dedup_by(|a, b| a.name() == b.name());

        // Add each service as a dynamic route
        self.add_dynamic_routes(services)
    }
}