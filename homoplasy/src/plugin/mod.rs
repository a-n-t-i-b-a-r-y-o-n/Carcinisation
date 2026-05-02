use morphology::package::PackageV1;
use crate::plugin::plugin_service::PluginService;
use crate::plugin::plugin_status::PluginStatus;

pub mod plugin_manager;
pub mod plugin_service;
mod plugin_status;
pub mod plugin_server_ext;

#[derive(Clone)]
pub struct Plugin {
    package: PackageV1,
    services: Vec<PluginService>,
    status: PluginStatus
}

impl Plugin {

}