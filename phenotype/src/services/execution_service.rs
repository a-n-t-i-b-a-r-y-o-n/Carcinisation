use std::ffi::OsStr;
use std::fs;
use std::fs::Permissions;
use std::os::unix::fs::PermissionsExt;
use anyhow::Result;
use std::path::Path;
use tokio::process::{Child, Command};


pub struct ExecutionService;

impl ExecutionService {

    /// Start the given plugin server binary
    pub fn start_plugin_server<A, B>(entrypoint: &A, working_dir: &B) -> Result<Child> 
    where
        A: AsRef<OsStr> + AsRef<Path>,
        B: AsRef<OsStr> + AsRef<Path>,
    {
        // Make sure it's executable
        fs::set_permissions(entrypoint, Permissions::from_mode(0o755))?;

        // Spawn the plugin binary process
        let child = Command::new(entrypoint)
            .current_dir(working_dir)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()?;

        Ok(child)
    }

}