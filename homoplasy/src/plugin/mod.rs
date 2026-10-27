use std::path::PathBuf;
use std::sync::Arc;
use parking_lot::{Mutex, RwLock};
use tempfile::TempPath;
use crate::plugin::plugin_service::PluginService;
use crate::plugin::plugin_status::PluginStatus;

use morphology::package::PackageV1;

pub mod plugin_manager;
pub mod plugin_service;
mod plugin_status;
pub mod plugin_server_ext;
mod plugin_process;
mod file_descriptor_set;

#[derive(Clone)]
pub struct Plugin {
    inner: Arc<PluginInner>,
}

struct PluginInner {
    package: Mutex<PackageV1>,
    services: Mutex<Vec<PluginService>>,
    status: RwLock<PluginStatus>,
    server_socket: RwLock<TempPath>,
    extracted_dir: RwLock<PathBuf>,
}


impl Plugin {

    pub fn new(package: PackageV1, services: Vec<PluginService>, socket: TempPath, extracted: PathBuf) -> Self {
        Plugin {
            inner: Arc::new(
                PluginInner {
                    package: Mutex::new(package),
                    services: Mutex::new(services),
                    status: RwLock::new(PluginStatus::NotStarted),
                    server_socket: RwLock::new(socket),
                    extracted_dir: RwLock::new(extracted),
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

    pub fn get_server_socket(&self) -> PathBuf {
        self.inner.server_socket.read().to_path_buf()
    }

    pub fn get_status(&self) -> PluginStatus {
        self.inner.status.read().to_owned()
    }
}