use crate::package::{PackageVersion, manifest::{PackageManifest, ManifestVersion}};

use serde::{Deserialize, Serialize};

/// V1 plugin package manifest
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct PackageManifestV1 {
    manifest_version: ManifestVersion,
    package_version: PackageVersion,
    plugin_name: String,
    plugin_version: String,
    proto_dir: String,
    execution_dir: String,
    entry_point: String,
}

/// Parse V1 package manifest from bytes
impl From<Vec<u8>> for PackageManifestV1 {
    fn from(value: Vec<u8>) -> Self {
        let json = String::from_utf8(value).unwrap_or(String::from("{}"));

        serde_json::from_str(&json).unwrap_or_default()
    }
}

impl PackageManifest for PackageManifestV1 {
    fn manifest_version(&self) -> ManifestVersion {
        ManifestVersion::V1
    }

    fn package_version(&self) -> PackageVersion {
        self.package_version.clone()
    }

    fn plugin_name(&self) -> String {
        String::from(&self.plugin_name)
    }

    fn plugin_version(&self) -> String {
        String::from(&self.plugin_version)
    }

    fn plugin_proto_dir(&self) -> String {
        String::from(&self.proto_dir)
    }

    fn plugin_execution_dir(&self) -> String {
        self.execution_dir.clone()
    }

    fn plugin_entrypoint(&self) -> String {
        String::from(&self.entry_point)
    }
}