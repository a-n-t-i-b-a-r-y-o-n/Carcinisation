use std::path::PathBuf;
use std::sync::Arc;

use parking_lot::{Mutex, RwLock};
use tokio::process::Child;

use morphology::package::PackageV1;
use crate::plugin::plugin_service::PluginService;
use crate::plugin::plugin_status::PluginStatus;

pub mod plugin_manager;
pub mod plugin_service;
mod plugin_status;
pub mod plugin_server_ext;

#[derive(Clone)]
pub struct Plugin {
    inner: Arc<PluginInner>,
}

struct PluginInner {
    package: Mutex<PackageV1>,
    services: Mutex<Vec<PluginService>>,
    status: RwLock<PluginStatus>,
    port: u32,
    extracted_dir: RwLock<PathBuf>,
    process: RwLock<Option<Child>>,
}


impl Plugin {

    pub fn new(package: PackageV1, services: Vec<PluginService>, port: u32, extracted: PathBuf) -> Self {
        Plugin {
            inner: Arc::new(
                PluginInner {
                    package: Mutex::new(package),
                    services: Mutex::new(services),
                    status: RwLock::new(PluginStatus::NotStarted),
                    port,
                    extracted_dir: RwLock::new(extracted),
                    process: RwLock::new(None)
                }
            )
        }
    }

    pub fn get_package(&self) -> PackageV1 {
        self.inner.package.lock().to_owned()
    }

    pub fn get_services(&self) -> Vec<PluginService> {
        self.inner.services.lock().to_owned()
    }

    pub fn get_status(&self) -> PluginStatus {
        self.inner.status.read().to_owned()
    }
}