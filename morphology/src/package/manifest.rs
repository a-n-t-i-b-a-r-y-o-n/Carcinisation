use serde_repr::{Deserialize_repr, Serialize_repr};
use strum_macros::{EnumString, VariantNames};
use crate::package::PackageVersion;

#[derive(
    Copy, Clone,
    Debug, Default,
    PartialEq,
    EnumString, VariantNames,
    Deserialize_repr, Serialize_repr
)]
#[strum(ascii_case_insensitive)]
#[repr(usize)]
pub enum ManifestVersion {
    #[default]
    V1 = 1,
}

pub trait PluginData: From<Vec<u8>> + Sized {

}

pub trait PackageManifest: From<Vec<u8>> + Sized {

    /// Format version number of this manifest
    fn manifest_version(&self) -> ManifestVersion;
    /// Version number of the package containing this manifest
    fn package_version(&self) -> PackageVersion;

    /// Name of the plugin in this package
    fn plugin_name(&self) -> String;
    /// Version number of the plugin in this package
    fn plugin_version(&self) -> String;
    /// Path to the folder of .proto files
    fn plugin_proto_dir(&self) -> String;
    /// Path to the entrypoint working directory
    fn plugin_execution_dir(&self) -> String;
    /// Path to entrypoint executable in working directory
    fn plugin_entrypoint(&self) -> String;
}