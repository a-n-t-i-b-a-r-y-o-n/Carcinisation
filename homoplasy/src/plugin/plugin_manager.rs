use std::ffi::OsStr;
use std::fs;
use std::fs::remove_file;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use anyhow::{Result, anyhow};
use convert_case::ccase;
use lazy_static::lazy_static;
use parking_lot::RwLock;
use tempfile::{NamedTempFile, TempDir, TempPath};
use tokio::io::AsyncWriteExt;
use tokio::net::UnixListener;
use tokio::spawn;
use tokio::task::JoinHandle;
use tonic::codegen::tokio_stream::wrappers::UnixListenerStream;
use tonic::transport::Server;
use crate::plugin::Plugin;
use crate::plugin::plugin_service::PluginService;
use crate::plugin::plugin_status::PluginStatus;
use crate::server::Named;
use morphology::package::manifest::PackageManifest;
use morphology::package::{PackageV1, PluginPackage};
use phenotype::bootstrap_message::BootstrapMessage;
use phenotype::phenotype_server::PhenotypeServer;
use phenotype::PhenotypeService;
use crate::plugin::plugin_process::PluginProcess;

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
    ipc_socket: RwLock<TempPath>,
    plugins: RwLock<Vec<Plugin>>,
    working_dir: TempDir,
}

impl<'p> PluginManager {

    /// Create a new [`PluginManager`] instance
    fn new() -> Self {
        // TODO: UE-93 - Cache extracted plugins
        // Create scratch directory for live plugins
        let temp_dir = TempDir::with_prefix("plugins-")
            .expect("Failed to create working directory for PluginManager");

        // Create IPC socket path for plugins
        let ipc_socket = NamedTempFile::new()
            .expect("Failed to create IPC socket for PluginManager")
            .into_temp_path();

        // Delete the temp file on disk. Will be recreated on bind and cleaned up when dropped.
        remove_file(&ipc_socket)
            .expect("Failed to remove temporary IPC socket");

        // Return default, empty plugin manager
        PluginManager {
            inner: Arc::new(PluginManagerInner {
                ipc_socket: RwLock::new(ipc_socket),
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

        // Parse all services
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
        // Check plugin package storage folder path
        let packages_folder = Path::new(plugin_folder);
        if !packages_folder.exists() {
            return Err(anyhow!("Plugin folder \"{}\" does not exist", packages_folder.display()));
        }

        // Working directory for plugin extraction
        let extraction_dir = (&*SINGLETON).inner.working_dir.path();

        // Get list of package files
        let packages = Self::enumerate_packages(&packages_folder)?;

        // Build list of plugins by loading each package
        let mut loaded_count = 0;
        for mut package in packages {

            let package_filename = &package.file.file_name().unwrap().display();

            println!("[-] Validating package: \"{}\"", package_filename);

            // Verify the package
            match package.validate() {
                Ok(_) => {}
                Err(e) => {
                    println!("[!] Error validating package: {}", e);
                    continue;
                }
            }

            println!("[-] Loading package: \"{}\"", package_filename);

            // Extract documented package contents
            let extract_root = Self::extract_package(&mut package, extraction_dir)?;

            // Get extracted proto directory
            let proto_path = extract_root.join(package.manifest.plugin_proto_dir());

            // Create a new socket for the plugin's gRPC server
            let plugin_socket = NamedTempFile::new()?.into_temp_path();

            // Delete the temp file off disk, retaining auto-cleanup on drop
            remove_file(&plugin_socket)?;

            // Parse protos to proxy services
            let mut services = Self::parse_proto_files(&proto_path)?;

            // Configure each proxy service to use the socket
            for service in services.iter_mut() {
                service.bind_to_server(plugin_socket.to_path_buf())?;
            }

            println!("[+] Loaded plugin: {} v{}", package.manifest.plugin_name(), package.manifest.plugin_version());

            // Build plugin object
            let plugin = Plugin::new(package, services, plugin_socket, extract_root);

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

    /// Start the IPC server on the dedicated socket
    pub async fn start_ipc() -> Result<JoinHandle<()>> {
        // Get singleton instance
        let inner = &*SINGLETON.inner;

        // Get the path to the IPC socket
        let socket_path = inner.ipc_socket.read();
        let socket_path = socket_path.to_path_buf();

        // Set up a unix domain listener stream on the socket
        let listener = UnixListener::bind(socket_path)?;
        let stream = UnixListenerStream::new(listener);

        // Build a gRPC server on the socket
        let server = Server::builder()
            .add_service(PhenotypeServer::new(PhenotypeService::default()))
            .serve_with_incoming(stream);

        // Wrap the server future
        let server_future = async move {
            // TODO: Handle IPC server errors?
            _ = server.await;
        };

        // Spawn the server future and get the handle
        let handle = spawn(server_future);

        // Return the handle
        Ok(handle)
    }

    /// Initialize the plugin manager
    pub async fn start_plugins() -> Result<Vec<JoinHandle<()>>> {

        let inner = &*SINGLETON.inner;
        let mut plugins = inner.plugins.write();

        // Get the global IPC socket
        let ipc_socket = inner.ipc_socket.read();

        // Build vec of plugin handles
        let mut handles = vec!();

        for plugin in plugins.iter_mut() {
            // Package information
            let package = plugin.get_package();

            // Get plugin extraction directory
            let plugin_dir = plugin.inner.extracted_dir.read();

            // Get absolute execution paths
            let working_dir = plugin_dir.join(package.manifest.plugin_execution_dir());
            let entrypoint = working_dir.join(package.manifest.plugin_entrypoint());

            println!("[-] Plugin entrypoint: {}", entrypoint.display());

            // Launch the plugin binary
            let mut plugin_process = PluginProcess::spawn_plugin_process(&entrypoint, &working_dir).await?;

            // Send the bootstrap message with socket paths
            let message = BootstrapMessage {
                server_socket: plugin.get_server_socket().as_path().display().to_string(),
                ipc_socket: ipc_socket.to_path_buf().display().to_string(),
            };
            _ = plugin_process.stdin.write(&message.serialize()?).await?;
            _ = plugin_process.stdin.flush().await;

            // Update plugin status
            *plugin.inner.status.write() = PluginStatus::Active;

            // Add this handle to the list
            handles.push(plugin_process.process)
        }

        Ok(handles)
    }
}
