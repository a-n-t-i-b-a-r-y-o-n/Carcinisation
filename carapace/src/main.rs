use clap::Parser;
use morphology::package::{PackageV1, PluginPackage};
use crate::cli::{Cli, Command};
use crate::pack::package_plugin;

pub mod cli;
mod pack;

fn main() -> Result<(), Box<dyn std::error::Error>> {

    // Parse CLI args
    let args = Cli::parse();
    match args.command {
        Command::Pack { .. } => {
            let package = package_plugin(args.command)?;

            println!("Wrote output package: {}", package.display());
        }
        Command::Verify { package_file } => {

            match PackageV1::read(&package_file) {
                None => {
                    println!("Failed to read package!");
                }
                Some(package) => {
                    match package.validate() {
                        Ok(_) => println!("Package verification successful!"),
                        Err(e) => println!("Package verification failed: {}", e),
                    }
                }
            }

        },
        Command::Dump { package_file } => {
            let mut archive = PackageV1::open_archive(&package_file)?;
            let entries = archive.entries()?;

            println!("Package {}:\n", &package_file.file_name().unwrap().display());
            for entry in entries {
                let path = entry?.path()?.display().to_string();

                println!("{}", path);
            }
        }
    }

    Ok(())
}
