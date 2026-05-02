use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{anyhow, Result};
use convert_case::ccase;
use lazy_static::lazy_static;
use parking_lot::RwLock;
use tempfile::TempDir;

use morphology::package::{PackageV1, PluginPackage};
use morphology::package::manifest::PackageManifest;
use phenotype::services::communication_service::CommunicationService;
use phenotype::services::execution_service::ExecutionService;
use phenotype::services::port_service::PortService;
use crate::plugin::Plugin;
use crate::plugin::plugin_service::PluginService;
use crate::plugin::plugin_status::PluginStatus;
use crate::server::dynamic_named_service::DynamicNamedService;

lazy_static! {
    // Static plugin manager singleton
    static ref SINGLETON: PluginManager = PluginManager::new();
}

/// Plugin loading interface
#[derive(Clone)]
pub struct PluginManager {
    inner: Arc<PluginManagerInner>,
}

struct PluginManagerInner {
    plugins: RwLock<Vec<Plugin>>,
    working_dir: TempDir,
}

impl<'p> PluginManager {

    /// Create a new [`PluginManager`] instance
    fn new() -> Self {
        // Create scratch directory for live plugins
        let temp_dir = TempDir::with_prefix("plugins-")
            .expect("Failed to create working directory for PluginManager");

        // Return default, empty plugin manager
        PluginManager {
            inner: Arc::new(PluginManagerInner {
                plugins: RwLock::new(vec!()),
                working_dir: temp_dir,
            })
        }
    }

    /// Build the list of package files in the given folder
    fn enumerate_packages(package_folder: &Path) -> Result<Vec<PackageV1>> {
        // Enumerate valid packages in the given directory
        let packages = fs::read_dir(package_folder)?
            // Filter readable `.tar.xz` files
            .filter_map(|entry| {
                match entry.as_ref() {
                    Ok(file) if file.file_type().is_ok_and(|f| f.is_file()) &&
                        file.file_name().to_ascii_lowercase().to_str()?.ends_with(".tar.xz")
                    => Some(file.path()),
                    _ => None,
                }
            })
            // Read and parse the package into a Plugin
            .filter_map(|path| PackageV1::read(&path))
            .collect::<Vec<_>>();

        Ok(packages)
    }

    /// Extract the important contents from the package
    fn extract_package(package: &mut PackageV1, extraction_root: &Path) -> Result<PathBuf> {
        // Get output directory name from plugin name
        let package_name = ccase!(snake, &package.manifest.plugin_name());
        let package_dir = extraction_root.join(&package_name);

        // Delete existing plugin directory if necessary
        if package_dir.exists() {
            if let Err(e) = fs::remove_dir_all(&package_dir) {
                return Err(anyhow!(e))
            }
        }
        // Create new plugin directory
        if let Err(e) = fs::create_dir(&package_dir) {
            return Err(anyhow!(e))
        }

        // Extract proto files directory
        let proto_path = PathBuf::from(package.manifest.plugin_proto_dir());
        let _ = package.extract_dir_to(&proto_path, &package_dir)?;

        // Extract entrypoint directory
        let entrypoint_path = PathBuf::from(package.manifest.plugin_execution_dir());
        let _ = package.extract_dir_to(&entrypoint_path, &package_dir)?;

        Ok(package_dir)
    }

    /// Parse the directory of proto files into a list of services from each file
    fn parse_proto_files(proto_dir: &PathBuf) -> Result<Vec<PluginService>> {
        // Select protobuf files
        let protos = fs::read_dir(proto_dir)?
            .filter_map(|entry| {
                match entry.as_ref() {
                    Ok(file) if file.file_type().is_ok_and(|f| f.is_file()) &&
                        file.file_name().to_ascii_lowercase().to_str()?.ends_with(".proto")
                    => Some(file.path()),
                    _ => None,
                }
            })
            .collect::<Vec<_>>();

        let mut services = vec!();
        for proto in protos {

            // Parse services from proto file
            match PluginService::new_from_proto(&proto) {
                Ok(mut proto_services) => {
                    services.append(&mut proto_services);
                }
                Err(e) => {
                    // Gracefully handle the error
                    println!("Error parsing proto \"{}\": {:#?}", proto.display(), e)
                }
            }
        }

        // De-duplicate any added services by name
        services.dedup_by(|a, b| a.name() == b.name());

        Ok(services)
    }

    /// Load the plugins from disk
    pub async fn load_plugins<S>(plugin_folder: &S) -> Result<usize>
    where
        S: AsRef<OsStr> + AsRef<Path>
    {
        let path = Path::new(plugin_folder);
        if !path.exists() {
            return Err(anyhow!("Plugin folder \"{}\" does not exist", path.display()));
        }

        // Get list of package files
        let packages = Self::enumerate_packages(&path)?;

        // Build list of plugins from packages
        let mut loaded_count = 0;
        for mut package in packages {

            // Extract relevant package contents
            let extract_root = Self::extract_package(&mut package, path)?;

            // Get extracted proto directory
            let proto_path = extract_root.join(package.manifest.plugin_proto_dir());

            // Get a new port number
            let port_number = PortService::request_port_assignment()?;

            // Parse protos to services
            let services = Self::parse_proto_files(&proto_path)?;

            // Build plugin object
            let plugin = Plugin::new(package, services, port_number);

            // Update singleton's plugin list
            (&*SINGLETON).inner
                .plugins
                .write()
                .push(plugin);

            // Update plugin load counter
            loaded_count+=1;
        }

        Ok(loaded_count)
    }

    /// Get the list of loaded plugins
    pub fn get_plugins() -> Vec<Plugin> {
        (&*SINGLETON).inner.plugins.read().to_owned()
    }

    /// Initialize the plugin manager
    pub async fn start_plugins() -> Result<usize> {

        let inner = &*SINGLETON.inner;
        let plugin_directory = inner.working_dir.path().to_path_buf();
        let mut plugins = inner.plugins.write();

        let mut started_count = 0;
        for plugin in plugins.iter_mut() {
            // Package information
            let package = plugin.get_package();

            // Plugin port number
            let port = plugin.inner.port;

            // Get execution paths
            let working_dir = plugin_directory.join(package.manifest.plugin_execution_dir());
            let entrypoint = plugin_directory.join(package.manifest.plugin_entrypoint());

            println!("Plugin entrypoint: {}", entrypoint.display());

            // Set server port for proxy plugin service
            for service in plugin.inner.services.lock().iter_mut() {
                service.bind_to_port(port)?;
            }

            // Start the plugin binary
            let mut process = ExecutionService::start_plugin_server(&entrypoint, &working_dir)?;

            // Communicate IPC channel info to plugin
            _ = CommunicationService::initialize_plugin_server(&mut process, port).await;

            // Update the process associated with the plugin
            *plugin.inner.process.write() = Some(process);

            // Update plugin status
            *plugin.inner.status.write() = PluginStatus::Active;

            // Update started plugin counter
            started_count+=1;
        }

        Ok(started_count)
    }
}
