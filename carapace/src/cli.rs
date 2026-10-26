use std::path::PathBuf;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "Carapace", arg_required_else_help = true)]
pub(crate) struct Cli {
    /// Subcommands and their args
    #[command(subcommand)]
    pub(crate) command: Command,
}

#[derive(Subcommand)]
pub(crate) enum Command {
    Pack {
        /// Human-friendly name for the plugin
        #[arg(short = 'n', long = "name")]
        plugin_name: String,

        /// SemVer version of the plugin
        #[arg(short = 'v', long = "version")]
        plugin_version: String,

        /// Path to the protobuf directory to include
        #[arg(short = 'p', long = "proto")]
        proto_dir: PathBuf,

        /// Path to an optional execution directory to include
        #[arg(short = 'x', long = "execution")]
        execution_dir: Option<PathBuf>,

        /// Entrypoint filename within the execution directory
        #[arg(short = 'e', long = "entrypoint")]
        entry_point: PathBuf,

        /// Private key used to sign the package
        #[arg(short = 'k', long = "key")]
        signing_key: PathBuf,

        /// Additional files/folder to include in the package
        #[arg(short = 'a', long = "additional")]
        additional_files: Option<Vec<PathBuf>>,

        /// Output directory for the packed result
        #[arg(short = 'o', long = "output")]
        output_directory: PathBuf,
    },
    Verify {
        /// Path to the package to verify
        package_file: PathBuf,
    },
    Dump {
        /// Path to the package to dump
        package_file: PathBuf,
    }
}