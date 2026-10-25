use std::{fs, io};
use std::ffi::OsStr;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use anyhow::anyhow;
use anyhow::Result;
use convert_case::ccase;
use ed25519::Signature;
use liblzma::read::XzDecoder;
use liblzma::write::AutoFinishXzEncoder;
use tar::{Archive, Builder, EntryType};
use tempfile::TempDir;
use tempfile::NamedTempFile as TempFile;

use crate::package::{
    manifest::PackageManifest,
    ChecksumsV1,
    PackageManifestV1,
    PluginPackage
};
use crate::package::checksums::Checksums;

type ArchiveFormat = Archive<XzDecoder<File>>;
#[allow(dead_code)]
type ArchiveFormatBuilder = Builder<AutoFinishXzEncoder<File>>;

/// Version 1 of the plugin package format
#[derive(Clone)]
pub struct PackageV1 {
    pub signature: Signature,
    pub checksums: ChecksumsV1,
    pub manifest: PackageManifestV1,
    pub file: PathBuf,
}


/// PluginPackage implementation for Version 1
impl PluginPackage<PackageV1, PackageManifestV1> for PackageV1 {
    fn manifest(&self) -> PackageManifestV1 {
        self.manifest.clone()
    }

    fn extract_file<S>(&mut self, path_inside_archive: &S) -> Result<TempFile>
    where
        S: AsRef<OsStr> + AsRef<Path> + ?Sized
    {
        // Create temporary output file
        let mut temp_file = TempFile::new()?;

        // Get homogenous, sized representations both paths
        let path_in = PathBuf::from(path_inside_archive);
        let path_out = PathBuf::from(temp_file.path());
        
        // Write all contents
        match self.extract_file_to(&path_in, &path_out) {
            Ok(_) => {
                // Seek back to file start
                temp_file.seek(SeekFrom::Start(0))?;
                
                // Return created file
                Ok(temp_file)
            }
            Err(e) => Err(anyhow!(e))
        }
    }

    fn extract_file_to<S>(&mut self, path_inside_archive: &S, path_outside_archive: &S) -> Result<PathBuf>
    where
        S: AsRef<OsStr> + AsRef<Path>
    {
        // Make filenames proper paths
        let path_in = PathBuf::from(path_inside_archive);
        let path_out = PathBuf::from(path_outside_archive);

        // Open the archive
        let archive: &mut ArchiveFormat = &mut Self::open_archive(&self.file)?;

        // Get list of all entries in the archive
        let mut entries = archive.entries()?;

        // Try to find the entry by its file path
        let file_entry = Self::find_entry(&mut entries, &path_in);

        // Return error if not found
        if file_entry.is_none() {
            return Err(anyhow!("File not found in archive"))
        }

        // Unwrap found entry
        let file_entry = file_entry.unwrap();

        // Unpack raw data into given file
        {
            // Open output file
            let file = File::create(&path_out)?;
            
            // Create reader/writer
            let mut reader = BufReader::new(file_entry);
            let mut writer = BufWriter::new(file);

            // Copy all data
            io::copy(&mut reader, &mut writer)?;
            writer.flush()?;
        }

        Ok(path_out)
    }

    fn extract_dir<S>(&mut self, path_inside_archive: &S) -> Result<TempDir>
    where
        S: AsRef<OsStr> + AsRef<Path> + ?Sized
    {
        // Create temp output dir
        let temp_dir = tempfile::tempdir()?;

        // Get homogenous, sized representations both paths
        let path_in = PathBuf::from(path_inside_archive);
        let path_out = PathBuf::from(temp_dir.path());
        
        // Extract into temp directory
        match self.extract_dir_to(&path_in, &path_out) {
            Ok(_) => Ok(temp_dir),
            Err(e) => Err(anyhow!(e))
        }
    }

    fn extract_dir_to<S>(&mut self, path_inside_archive: &S, path_outside_archive: &S) -> Result<PathBuf>
    where
        S: AsRef<OsStr> + AsRef<Path>
    {
        // Make filenames proper paths
        let path_in = PathBuf::from(path_inside_archive);
        let path_out = PathBuf::from(path_outside_archive);

        // Open the archive
        let archive: &mut ArchiveFormat = &mut Self::open_archive(&self.file)?;

        // Get list of all entries in the archive
        let entries = archive.entries()?;

        let mut unpacked = 0;
        for entry in entries.filter(|e| e.is_ok()) {
            let mut entry = entry?;

            if !entry.path()?.starts_with(&path_in) {
                continue;
            }

            entry.unpack_in(&path_out)?;
            unpacked += 1;
        }

        if unpacked == 0 {
            Err(anyhow!("Directory not found in archive"))
        }
        else {
            Ok(path_out)
        }
    }

    fn read<S>(path: &S) -> Option<PackageV1>
    where
        S: AsRef<OsStr> + AsRef<Path> + ?Sized
    {
        let path = Path::new(path);
        if !path.exists() {
            println!("Package file {:?} does not exist!", path);

            return None;
        }

        // Metadata files to read from the archive
        let mut signature = None;
        let mut checksums = None;
        let mut manifest = None;

        // Open the archive to scan for metadata files
        match Self::open_archive(path) {
            Ok(mut archive) => {

                // Create an iterator over the Ok entries
                let entries = archive.entries().unwrap()
                    .into_iter().filter(|e| e.is_ok())
                    .map(|e| e.unwrap());

                // Stream entries from the tar file
                for mut entry in entries {

                    // Get the entry path or continue
                    let path = match entry.path() {
                        Ok(p) => {
                            match p.to_str() {
                                Some(s) => String::from(s),
                                None => continue,
                            }
                        }
                        Err(_) => continue,
                    };

                    // Check if we found the manifest
                    if path.eq_ignore_ascii_case("manifest.json") {

                        // Read the manifest from bytes
                        let mut bytes = vec!();
                        entry.read_to_end(&mut bytes).unwrap();

                        // Parse the package manifest
                        manifest = Some(PackageManifestV1::from(bytes));
                    }

                    // Check if we found the checksums
                    if path.eq_ignore_ascii_case("checksums.txt") {

                        // Read the checksums from bytes
                        let mut bytes = vec!();
                        entry.read_to_end(&mut bytes).unwrap();

                        // Parse the checksums file
                        checksums = Some(ChecksumsV1::from(bytes));
                    }

                    // Check if we found the signature
                    if path.eq_ignore_ascii_case("signature.bin") {
                        // Read the checksums from bytes
                        let mut bytes = vec!();
                        entry.read_to_end(&mut bytes).unwrap();

                        // Parse the digital signature
                        signature = Some(Signature::from_slice(&bytes).unwrap())
                    }

                }

            }
            Err(e) => {
                println!("Failed to open file: {:?}", e);

                return None;
            }
        }

        // Check that we found the metadata
        if checksums.is_none() || manifest.is_none() || signature.is_none() {

            if checksums.is_none() {
                println!("No checksums found!");
            }
            if manifest.is_none() {
                println!("No manifest found!");
            }
            if signature.is_none() {
                println!("No signature found!");
            }

            return None;
        }

        let signature = signature.unwrap();
        let checksums = checksums.unwrap();
        let manifest = manifest.unwrap();
        let file = path.to_path_buf();

        Some(
            PackageV1 {
                signature,
                checksums,
                manifest,
                file,
            }
        )
    }

    fn validate(&self) -> Result<()> {
        if !self.file.exists() {
            return Err(anyhow!("Cannot validate nonexistent package!"));
        }

        // Open the file to check each file hash
        match Self::open_archive(&self.file) {
            Ok(mut archive) => {
                // Create an iterator over the Ok entries
                let entries = archive.entries()?.into_iter();

                // Iterate everything in the archive
                for entry in entries {

                    // Try to get the entry
                    let mut entry = match entry {
                        Ok(entry) => entry,
                        Err(err) => return Err(anyhow!("Found invalid entry in archive: {}", err.to_string())),
                    };

                    // Try to get the path
                    let path = match entry.path() {
                        Ok(path) => {
                            match path.to_str() {
                                Some(path) => String::from(path),
                                None => return Err(anyhow!("Found entry in archive with no path")),
                            }
                        }
                        Err(err) => return Err(anyhow!("Unable to get archive entry path: {}", err.to_string())),
                    };

                    // Skip the checksum and signature files at this stage
                    if path.eq_ignore_ascii_case("checksums.txt") || path.eq_ignore_ascii_case("signature.bin") {
                        continue;
                    }

                    // Skip checking folders
                    if path.ends_with("/") || entry.header().entry_type() == EntryType::Directory {
                        continue;
                    }

                    // Read the entry content
                    let mut entry_content = vec!();
                    entry.read_to_end(&mut entry_content)?;

                    // Check the content hash
                    let _checksum = match self.checksums.validate_file_checksum(&path, &entry_content) {
                        Ok(c) => c,
                        Err(e) => return Err(e),
                    };
                }
            }
            Err(e) => {
                return Err(e)
            }
        }

        Ok(())
    }

    fn write<S>(&self, path: &S) -> Result<()>
    where
        S: AsRef<OsStr> + AsRef<Path> + ?Sized
    {
        let path = Path::new(path);
        let input_directory = path.canonicalize()?;
        let output_directory = input_directory.parent().unwrap();

        let package_name = ccase!(snake, &self.manifest.plugin_name());
        let package_filename = format!("package_{}.tar.xz", package_name);

        let output_path = output_directory.join(package_filename);
        let mut output_file = Self::create_archive(&output_path)?;

        let manifest_content = serde_json::to_vec_pretty(&self.manifest())?;

        let manifest_path = &output_directory.join("manifest.json");
        let mut manifest_file = File::create(manifest_path)?;
        manifest_file.write_all(&manifest_content)?;
        _ = manifest_file.flush()?;

        let mut checksums_content = String::new();
        for (file, checksum) in &self.checksums.checksums {
            checksums_content = format!("{}{} - {}\n", checksums_content, file, checksum)
        }

        let checksums_path = &output_directory.join("checksums.txt");
        let mut checksums_file = File::create(checksums_path)?;
        checksums_file.write_all(checksums_content.as_bytes())?;
        _ = checksums_file.flush()?;

        output_file.append_file("manifest.json", &mut File::open(&manifest_path)?)?;
        output_file.append_file("checksums.txt", &mut File::open(&checksums_path)?)?;

        output_file.append_dir_all("", &input_directory)?;

        _ = output_file.finish();

        fs::remove_file(manifest_path)?;
        fs::remove_file(checksums_path)?;

        Ok(())

    }
}