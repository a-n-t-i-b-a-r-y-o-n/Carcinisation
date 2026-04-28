pub mod v1;
pub mod manifest;

pub use anyhow::Result;

pub use tempfile::TempDir;
pub use tempfile::NamedTempFile as TempFile;

// Re-export latest package version for convenience
pub use crate::package::{
    v1::PackageV1,
    v1::manifest::PackageManifestV1,
};


use std::path::PathBuf;
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
    fn extract_file(&mut self, path_in_archive: &str) -> Result<TempFile>;

    // Extract a directory from the package
    fn extract_dir(&mut self, path_in_archive: &str) -> Result<TempDir>;

    /// Read the plugin package from disk
    fn read(path: &PathBuf) -> Option<TPackage>;

    /// Write the plugin package to disk
    fn write(&self, path: &PathBuf) -> Result<()>;
}