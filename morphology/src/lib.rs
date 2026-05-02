pub mod package;

#[test]
fn package_dump() {

    use std::path::Path;
    use crate::package::PackageV1;

    let path = Path::new("test/test_input/package_test_package_1.tar.xz").to_path_buf();
    let archive = PackageV1::open_archive(&path);

    assert!(archive.is_ok());

    let mut archive = archive.unwrap();

    let entries = archive.entries();

    assert!(entries.is_ok());

    let entries = entries.unwrap();

    for entry in entries {
        let e = &entry.unwrap();
        let p = e.path().unwrap();
        let s = p.to_str().unwrap();

        println!("Entry: {}", s);
    }
}

#[test]
fn test_writing() {

    use std::path::Path;
    use crate::package::{
        PluginPackage,
        PackageV1,
        PackageManifestV1,
    };

    let path = Path::new("test/test_input").to_path_buf();
    let manifest_content = r#"{
    "manifest_version": 1,
    "package_version": 1,
    "plugin_name": "plugin-contract test",
    "plugin_version": "3.2.1",
    "proto_dir": "/proto",
    "execution_dir": "",
    "entry_point": ""
}"#;

    let manifest: PackageManifestV1 = serde_json::from_str(manifest_content).unwrap();

    let package = PackageV1 {
        manifest,
        file: Path::new("test/test_input/package_plugin_contract_test.tar.xz").to_path_buf(),
    };

    match package.write(&path) {
        Ok(_) => {
            println!("Wrote package!")
        }
        Err(e) => {
            println!("Failed to write package: {:?}", e)
        }
    }

}

#[test]
fn test_reading() {

    use std::path::Path;
    use crate::package::v1::PackageV1;
    use crate::package::PluginPackage;
    use crate::package::manifest::PackageManifest;

    let path = Path::new("test/test_input/package_plugin_contract_test.tar.xz").to_path_buf();

    match PackageV1::read(&path) {
        Some(package) => {
            println!("Read package!");

            println!("Plugin name: {}", package.manifest().plugin_name());
            println!("Plugin version: {}", package.manifest().plugin_version());
        }
        None => {
            println!("Unable to read package!")
        }
    }
}

#[test]
fn test_file_extraction() {

    use std::path::Path;
    use std::io::Read;
    use crate::package::PackageV1;
    use crate::package::PackageManifestV1;
    use crate::package::PluginPackage;

    let path = Path::new("test/test_input/package_plugin_contract_test.tar.xz").to_path_buf();
    let manifest_content = r#"{
    "manifest_version": 1,
    "package_version": 1,
    "plugin_name": "plugin-contract test",
    "plugin_version": "3.2.1",
    "proto_dir": "/proto",
    "execution_dir": "",
    "entry_point": ""
}"#;

    let package = PackageV1::read(&path);

    assert!(package.is_some());

    let mut package = package.unwrap();

    let extracted = package.extract_file("manifest.json");

    assert!(extracted.is_ok());

    let mut extracted = extracted.unwrap();

    let mut extracted_content = String::new();
    let read = extracted.read_to_string(&mut extracted_content);

    assert!(read.is_ok());
    let read = read.unwrap();
    assert_ne!(read, 0);

    let written_manifest: PackageManifestV1 = serde_json::from_str(manifest_content).unwrap();
    let read_manifest: PackageManifestV1 = serde_json::from_str(&extracted_content).unwrap();

    assert_eq!(written_manifest, read_manifest);

}

#[test]
fn test_dir_extraction() {

    use std::fs;
    use std::path::Path;
    use crate::package::v1::PackageV1;
    use crate::package::PluginPackage;

    let path = Path::new("test/test_input/package_test_package_1.tar.xz").to_path_buf();
    let package = PackageV1::read(&path);

    assert!(package.is_some());

    let mut package = package.unwrap();

    let extracted = package.extract_dir("proto");

    if extracted.is_err() {
        println!("{:?}", extracted.unwrap());
        return
    }

    assert!(&extracted.is_ok());

    let extracted = extracted.unwrap();

    println!("Extracted: {:?}", extracted);

    let extracted_path = extracted.path();

    assert!(extracted_path.is_dir());

    assert!(extracted_path.read_dir().is_ok());

    let proto_dir = extracted_path.join("proto");

    let extracted_files = fs::read_dir(proto_dir).unwrap();
    let count = extracted_files.count();

    assert_ne!(count, 0);

    assert!(count > 1);
}