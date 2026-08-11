use anyhow::Result;
use std::ffi::OsStr;
use std::fs;
use std::fs::Permissions;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::pin::Pin;
use tokio::io::AsyncWrite;
use tokio::process::Command;
use tokio::spawn;
use tokio::task::JoinHandle;

pub(crate) struct PluginProcess {
    pub(crate) process: JoinHandle<()>,
    pub(crate) stdin: Pin<Box<dyn AsyncWrite + Send + Sync>>,
}

impl PluginProcess {
    pub(crate) async fn spawn_plugin_process<A, B>(entrypoint: &A, working_dir: &B) -> Result<PluginProcess>
    where
        A: AsRef<OsStr> + AsRef<Path>,
        B: AsRef<OsStr> + AsRef<Path>,
    {
        // Make sure it's executable
        fs::set_permissions(entrypoint, Permissions::from_mode(0o755))?;

        // Spawn the plugin binary process
        let mut child = Command::new(entrypoint)
            .current_dir(working_dir)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::inherit())
            .spawn()?;

        // Take stdin
        let stdin = Box::pin(child.stdin.take().unwrap());

        // Wrap with a join handle
        let process = spawn(async move {
            // TODO: Handle plugin child process errors?
            _ = child.wait().await;
        });

        Ok(PluginProcess {
            process,
            stdin,
        })
    }
}
