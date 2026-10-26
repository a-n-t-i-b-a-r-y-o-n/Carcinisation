use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use anyhow::{anyhow, Result};
use convert_case::ccase;
use liblzma::write::XzEncoder;
use tar::Builder;
use morphology::package::{
    checksums::Checksums,
    ChecksumsV1,
};
use morphology::package::v1::signature::PluginSigner;
use crate::cli::Command;

/// Calculate checksums recursively for a given folder with a given path prefix
fn recursive_checksums(prefix: &str, path: &PathBuf, depth: Option<usize>) -> Result<HashMap<String, String>> {
    // Checksum tracker
    let mut checksums = HashMap::new();

    // Root file or folder
    let root = path.display().to_string();

    for file in path.read_dir()? {
        // Unwrap the file result
        let file = file?;

        // Recurse into subdirectories
        if file.file_type()?.is_dir() {
            // Increment depth counter
            let depth = if depth.is_none() { Some(0) }
            else { Some(depth.unwrap() + 1) };

            // Get the subdirectory name
            let sub_root = file.path();
            let sub_root = sub_root.file_name().unwrap()
                .display().to_string();

            // Append the subdirectory to the path prefix we have so far
            let prefix = format!("{}/{}", prefix, sub_root);

            // Recurse one level deeper
            let recursion = recursive_checksums(&prefix, &file.path(), depth)?;

            // Add to our growing map of checksums
            checksums.extend(recursion);

            continue;
        }

        // Get the relative path without the root
        let relative_path = file.path().display().to_string();
        let relative_path = relative_path.trim_start_matches(&root);
        let relative_path = format!("{}{}", prefix, relative_path);

        // Open the file for reading
        let mut file_content = File::open(file.path())?;

        // Read the file content
        let mut bytes = vec!();
        _ = file_content.read_to_end(&mut bytes);

        // Calculate the file's checksum
        let checksum = ChecksumsV1::generate_checksum(&bytes);

        // Add to our map of checksums
        checksums.insert(String::from(relative_path), checksum);
    }

    // Trim leading slashes, if any
    if depth.is_none() {
        checksums = checksums.iter()
            .map(|(path, checksum)| {
                (String::from(path.trim_start_matches("/")), String::from(checksum))
            })
            .collect::<HashMap<String, String>>();
    }

    Ok(checksums)
}

/// Get the relative entrypoint filename
fn get_entrypoint_filename(execution_dir: &Option<PathBuf>, entry_point: &PathBuf) -> String {
    match &execution_dir {
        None => {
            // If no execution directory given, assume we'll put the entrypoint in the root of "server/"
            entry_point.file_name()
                .unwrap()
                .display()
                .to_string()
        },
        Some(directory) => {
            // Get the full path to the entrypoint
            let full_entrypoint_path = entry_point.file_name()
                .unwrap()
                .display()
                .to_string();

            // Get the root it's in
            let execution_directory_path = directory.display().to_string();

            // Get the relative entrypoint path within the root, which we'll put into "server/"
            let relative_entrypoint_path = full_entrypoint_path.trim_start_matches(&execution_directory_path);

            String::from(relative_entrypoint_path)
        }
    }
}

/// Calculate checksums for the entrypoint and/or executable directory
fn get_execution_checksums(execution_dir: &Option<PathBuf>, entry_point: &PathBuf, entrypoint_filename: &str) -> Result<HashMap<String, String>> {
    let mut checksums = HashMap::new();

    if execution_dir.is_some() {
        // Get checksums of everything in the execution directory with "server/*" paths
        let execution_checksums =
            recursive_checksums("server", &execution_dir.clone().unwrap(), None)?;

        // Add to our map of checksums
        checksums.extend(execution_checksums)
    } else {
        // Read the entrypoint file directly
        let mut entrypoint_file = File::open(&entry_point)?;
        let mut bytes = vec!();
        entrypoint_file.read_to_end(&mut bytes)?;

        // Generate a checksum for the entrypoint file alone
        let entrypoint_checksum = ChecksumsV1::generate_checksum(&bytes);

        // Add to our map of checksums
        checksums.insert(format!("server/{}", entrypoint_filename), entrypoint_checksum);
    }

    Ok(checksums)
}

/// Calculate checksums for the given additional files
fn get_additional_files_checksums(additional_files: &Vec<PathBuf>) -> Result<HashMap<String, String>> {
    let mut checksums = HashMap::new();

    for path in additional_files {
        if path.is_dir() {
            // Calculate checksums for the additional files
            let additional_checksums = recursive_checksums("server", &path, None)?;

            // Add to our map of checksums
            checksums.extend(additional_checksums);
        } else {
            // Open the file for reading
            let mut additional_file = File::open(&path)?;

            // Read the file
            let mut bytes = vec!();
            additional_file.read_to_end(&mut bytes)?;

            // Calculate the checksum
            let additional_checksum = ChecksumsV1::generate_checksum(&bytes);

            // Get the relative path we'll use inside "server/"
            let relative_path = format!("server/{}", path.file_name().unwrap().display());

            // Add to our map of checksums
            checksums.insert(relative_path, additional_checksum);
        }
    }

    Ok(checksums)
}

pub(crate) fn package_plugin(command: Command) -> Result<PathBuf> {
    match command {
        Command::Pack {
            plugin_name,
            plugin_version,
            proto_dir,
            execution_dir,
            entry_point,
            signing_key,
            additional_files,
            output_directory
        } => {

            // Check if output directory exists
            if !fs::exists(&output_directory)? {
                // Check if the parent exists
                if fs::exists(&output_directory.parent().unwrap())? {
                    // Create the leaf
                    fs::create_dir(&output_directory)?;
                } else {
                    return Err(anyhow!("Output directory parent does not exist!"));
                }
            }

            // Get the entrypoint filename relative to the execution directory
            let entrypoint_filename = get_entrypoint_filename(&execution_dir, &entry_point);

            // Build the manifest
            let manifest_content = format!(
r#"{{
  "manifest_version": 1,
  "package_version": 1,
  "plugin_name": "{plugin_name}",
  "plugin_version": "{plugin_version}",
  "proto_dir": "proto",
  "execution_dir": "server",
  "entry_point": "{entrypoint_filename}"
}}"#
            );

            // Checksum tracker
            let mut checksums = HashMap::new();

            // Write the manifest to a temporary file
            let mut manifest_file = tempfile::tempfile()?;
            manifest_file.write_all(manifest_content.as_bytes())?;

            // Write the manifest checksum
            let manifest_checksum = ChecksumsV1::generate_checksum(manifest_content.as_bytes());
            _ = checksums.insert(String::from("manifest.json"), manifest_checksum);

            // Get protobuf file checksums with "proto/*" paths
            let proto_checksums =
                recursive_checksums("proto", &proto_dir, None)?;

            // Add to our map of checksums
            checksums.extend(proto_checksums);

            // Calculate checksums for the entrypoint and/or executable directory
            let execution_checksums =
                get_execution_checksums(&execution_dir, &entry_point, &entrypoint_filename)?;
            checksums.extend(execution_checksums);

            // Check for any additional files/folders
            if additional_files.is_some() {
                // Calculate checksums for the additional files/folders
                let additional_checksums = get_additional_files_checksums(&additional_files.clone().unwrap())?;
                checksums.extend(additional_checksums);
            }

            // Build the checksum file content
            let checksum_content = ChecksumsV1::from(checksums);

            // Write the checksums to a temporary file
            let mut checksums_file = tempfile::tempfile()?;
            checksums_file.write_all(&checksum_content.raw_bytes)?;

            // Parse the PEM signing key
            let signer = PluginSigner::from(&signing_key);

            // Sign the checksum file
            let signature = signer.sign(&checksum_content.raw_bytes);

            // Write the signature to a temporary file
            let mut signature_file = tempfile::tempfile()?;
            signature_file.write_all(&signature.to_vec())?;

            // Determine the output package filename
            let package_name = ccase!(snake, &plugin_name);
            let package_filename = format!("{}.tar.xz", package_name);

            // Get the output file path
            let output_path = output_directory.join(package_filename);

            // Open the output file for writing
            let output_file = File::create(&output_path)?;

            // Build the XZ-compressed tarball
            let compressor = XzEncoder::new_parallel(output_file, 9);
            let mut tarball = Builder::new(XzEncoder::auto_finish(compressor));

            // Put the manifest, checksums, and signature in the package
            tarball.append_file("manifest.json", &mut manifest_file)?;
            tarball.append_file("checksums.txt", &mut checksums_file)?;
            tarball.append_file("signature.bin", &mut signature_file)?;

            // Put the protobuf directory in the package
            tarball.append_dir_all("proto", &proto_dir)?;

            // Append the execution directory or entrypoint
            if execution_dir.is_none() {
                // Open the entrypoint file
                let mut entrypoint_file = File::open(&entry_point)?;

                // Add entrypoint file directly
                tarball.append_file(&format!("server/{}", entrypoint_filename), &mut entrypoint_file)?;
            } else {
                // Add the whole execution directory
                tarball.append_dir_all("server", &mut execution_dir.unwrap())?;
            }

            // Append any additional files or folders
            if additional_files.is_some() {
                let additional_files = additional_files.unwrap();

                for additional_file in additional_files {

                    // Append entire directories
                    if additional_file.is_dir() {
                        tarball.append_dir_all("server", &additional_file)?;
                    } else {
                        // Get the filename
                        let additional_filename = additional_file.file_name().unwrap()
                            .display().to_string();

                        // Open the additional file
                        let mut file = File::open(&additional_file)?;

                        // Append to the tarball
                        tarball.append_file(&format!("server/{}", additional_filename), &mut file)?;
                    }

                }
            }

            // Finalize the tarball
            tarball.finish()?;

            Ok(output_path)
        },
        _ => {
            Err(anyhow!("Invalid command received for package_plugin() function"))
        }
    }
}