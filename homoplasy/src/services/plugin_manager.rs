use tonic::transport::server::Router;

/// Plugin loading interface
pub struct PluginManager { }

impl PluginManager {

    /// Initialize the plugin manager
    pub async fn init(&mut self) {

        /*
        // Mutable plugin connectors
        let mut connectors = (&*crate::plugin::generated::PLUGIN_CONNECTORS).iter()
            .map(|(_, pc)| pc.lock())
            .collect::<Vec<_>>();

        // Handles to plugin connector init functions
        let handles = connectors.iter_mut()
            .map(|h| h.init());

        // Initialize all the plugins
        futures::future::join_all(handles).await;

         */
    }

    /// Add tonic gRPC services from plugins to a Router
    pub fn add_tonic_services(&self, builder: Router) -> Router {

        /*

        let mut builder = builder;
        for (_, pc) in (&*crate::plugin::generated::PLUGIN_CONNECTORS).iter() {
            builder = pc.lock().add_tonic_services(builder);
        }

         */

        builder
    }
}
