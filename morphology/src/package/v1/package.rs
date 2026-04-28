use std::{fs, io};
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use anyhow::anyhow;
use anyhow::Result;
use convert_case::ccase;
use liblzma::read::XzDecoder;
use liblzma::write::AutoFinishXzEncoder;
use tar::{Archive, Builder};
use tempfile::TempDir;
use tempfile::NamedTempFile as TempFile;

use crate::package::{
    manifest::PackageManifest,
    PackageManifestV1,
    PluginPackage
};

type ArchiveFormat = Archive<XzDecoder<File>>;
#[allow(dead_code)]
type ArchiveFormatBuilder = Builder<AutoFinishXzEncoder<File>>;

/// Version 1 of the plugin package format
pub struct PackageV1 {
    pub manifest: PackageManifestV1,
    pub file: PathBuf,
}


/// PluginPackage implementation for Version 1
impl PluginPackage<PackageV1, PackageManifestV1> for PackageV1 {
    fn manifest(&self) -> PackageManifestV1 {
        self.manifest.clone()
    }

    fn extract_file(&mut self, filename: &str) -> Result<TempFile> {

        // Open the archive
        let archive: &mut ArchiveFormat = &mut Self::open_archive(&self.file)?;

        // Get list of all entries in the archive
        let mut entries = archive.entries()?;

        // Try to find the entry by its file path
        let file_entry = Self::find_entry(&mut entries, filename);

        // Return error if not found
        if file_entry.is_none() {
            return Err(anyhow!("File not found in archive"))
        }

        // Unwrap found entry
        let file_entry = file_entry.unwrap();

        // Create temporary output file
        let mut temp_file = TempFile::new()?;

        // Unpack raw data into temp file
        {
            let mut reader = BufReader::new(file_entry);
            let mut writer = BufWriter::new(&temp_file);

            io::copy(&mut reader, &mut writer)?;
            writer.flush()?;
        }

        temp_file.seek(SeekFrom::Start(0))?;

        Ok(temp_file)
    }

    fn extract_dir(&mut self, filename: &str) -> Result<TempDir> {
        // Ensure directory path ends with '/'
        let mut path_in_archive = String::from(filename);
        if !path_in_archive.ends_with('/') {
            path_in_archive.push('/');
        }

        // Open the archive
        let archive: &mut ArchiveFormat = &mut Self::open_archive(&self.file)?;

        // Create temp output dir
        let temp_dir = tempfile::tempdir()?;

        // Get list of all entries in the archive
        let entries = archive.entries()?;

        let mut unpacked = 0;
        for entry in entries.filter(|e| e.is_ok()) {
            let mut entry = entry?;

            if !entry.path()?.starts_with(&path_in_archive) {
                continue;
            }

            entry.unpack_in(temp_dir.path())?;
            unpacked += 1;
        }

        if unpacked == 0 {
            Err(anyhow!("Directory not found in archive"))
        }
        else {
            Ok(temp_dir)
        }
    }

    fn read(path: &PathBuf) -> Option<PackageV1> {
        if !path.exists() {
            println!("Package file {:?} does not exist!", path);

            return None;
        }

        match Self::open_archive(path) {
            Ok(mut archive) => {
                let mut entries = archive.entries().unwrap();

                match Self::find_entry(&mut entries, "manifest.json") {
                    None => {
                        println!("No manifest found!");

                        None
                    }
                    Some(mut entry) => {
                        // Read the manifest
                        let mut bytes = vec!();
                        entry.read_to_end(&mut bytes).unwrap();

                        let manifest = PackageManifestV1::from(bytes);

                        Some(
                            PackageV1 {
                                manifest,
                                file: path.to_path_buf(),
                            }
                        )
                    }
                }
            }
            Err(e) => {
                println!("Failed to open file: {:?}", e);

                None
            }
        }
    }

    /// Write a given plugin package to the given output directory
    fn write(&self, path: &PathBuf) -> Result<()> {

        let output_directory = path.canonicalize()?;

        let package_name = ccase!(snake, &self.manifest.plugin_name());
        let package_filename = format!("package_{}.tar.xz", package_name);

        let output_path = output_directory.join(package_filename);
        let mut output_file = Self::create_archive(&output_path)?;

        let manifest_content = serde_json::to_vec_pretty(&self.manifest())?;

        let manifest_path = &output_directory.join("manifest.json");
        let mut manifest_file = File::create(manifest_path)?;
        manifest_file.write_all(&manifest_content)?;

        output_file.append_dir_all("", &output_directory)?;

        fs::remove_file(manifest_path)?;

        Ok(())

    }
}