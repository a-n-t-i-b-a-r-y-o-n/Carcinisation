use std::fs;
use std::fs::Permissions;
use std::os::unix::fs::PermissionsExt;
use anyhow::Result;
use std::path::PathBuf;
use tokio::process::{Child, Command};


pub struct ExecutionService;

impl ExecutionService {

    /// Start the given plugin server binary
    pub fn start_plugin_server(entrypoint: &PathBuf, working_dir: &PathBuf) -> Result<Child> {
        // Make sure it's executable
        fs::set_permissions(entrypoint, Permissions::from_mode(0o755))?;

        let child = Command::new(&entrypoint)
            .current_dir(&working_dir)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()?;

        Ok(child)
    }

}