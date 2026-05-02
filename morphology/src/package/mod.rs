pub mod v1;
pub mod manifest;

use std::ffi::OsStr;
pub use anyhow::Result;

pub use tempfile::TempDir;
pub use tempfile::NamedTempFile as TempFile;

// Re-export latest package version for convenience
pub use crate::package::{
    v1::PackageV1,
    v1::manifest::PackageManifestV1,
};


use std::path::{Path, PathBuf};
use serde_repr::{Deserialize_repr, Serialize_repr};
use strum_macros::{EnumString, VariantNames};

#[derive(
    Copy, Clone,
    Debug, Default,
    PartialEq,
    EnumString, VariantNames,
    Deserialize_repr, Serialize_repr
)]
#[strum(ascii_case_insensitive)]
#[repr(usize)]
pub enum PackageVersion {
    #[default]
    V1 = 1,
}


/// Archive package containing a plugin
pub trait PluginPackage<TPackage, TManifest> {

    /// Get the package manifest
    fn manifest(&self) -> TManifest;

    /// Extract a file from the package
    fn extract_file<S>(&mut self, path_inside_archive: &S) -> Result<TempFile>
    where S: AsRef<OsStr> + AsRef<Path> + ?Sized;

    /// Extract a file from the package to a given directory
    fn extract_file_to<S>(&mut self, path_inside_archive: &S, path_outside_archive: &S) -> Result<PathBuf>
    where S: AsRef<OsStr> + AsRef<Path>;

    /// Extract a directory from the package
    fn extract_dir<S>(&mut self, path_inside_archive: &S) -> Result<TempDir>
    where S: AsRef<OsStr> + AsRef<Path> + ?Sized;

    /// Extract a directory from the package to a given directory
    fn extract_dir_to<S>(&mut self, path_inside_archive: &S, path_outside_archive: &S) -> Result<PathBuf>
    where S: AsRef<OsStr> + AsRef<Path>;

    /// Read the plugin package from disk
    fn read<S>(path: &S) -> Option<TPackage>
    where S: AsRef<OsStr> + AsRef<Path> + ?Sized;

    /// Write the plugin package to disk
    fn write<S>(&self, path: &S) -> Result<()>
    where S: AsRef<OsStr> + AsRef<Path> + ?Sized;
}