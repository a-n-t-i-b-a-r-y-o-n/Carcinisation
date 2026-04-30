use std::collections::HashMap;

use lazy_static::lazy_static;
use parking_lot::Mutex;
use tonic::transport::server::Router;

use crate::plugin::plugin_id::PluginID;
use crate::plugin::plugin_service::PluginService;

lazy_static! {
    // Static list of plugin connections
    pub static ref PLUGIN_SERVICES: HashMap<PluginID, Mutex<Box<()>>> = HashMap::new();
}

/// Plugin loading interface
pub struct PluginManager { }

impl PluginManager {

    /// Initialize the plugin manager
    pub async fn init(&mut self) {

        // TODO: Load plugins at runtime
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
    
    pub fn get_services(&self) -> Vec<PluginService> {
        
        // TODO: Return list of services at runtime
        vec!()
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
