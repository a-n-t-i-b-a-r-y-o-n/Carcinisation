use std::path::PathBuf;
use clap::Parser;

#[derive(Parser)]
#[command(name = "Convergence", arg_required_else_help = true)]
pub(crate) struct Cli {
    /// Server port number
    #[arg(long = "port", default_value_t = 50052)]
    pub(crate) port_number: u32,

    /// One or more plugins to load
    #[arg(long = "plugin")]
    pub(crate) plugin_paths: Vec<PathBuf>,
}