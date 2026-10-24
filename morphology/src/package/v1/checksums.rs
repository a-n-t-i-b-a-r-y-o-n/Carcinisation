use std::collections::HashMap;
use std::str::from_utf8;
use serde::{Deserialize, Serialize};
use crate::package::checksums::Checksums;

/// Basic checksums file containing a SHA-3 hash of each file.
///
/// Expected content shape:
/// ```text
/// relative/path/to/fileA - abc123def456
/// relative/path/to/fileB - 098fed765cba
/// ```
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct ChecksumsV1 {
    checksums: HashMap<String, String>,
}

impl From<Vec<u8>> for ChecksumsV1 {
    fn from(value: Vec<u8>) -> Self {
        let full_content = from_utf8(&value).unwrap_or("");
        if full_content.is_empty() {
            return Self { checksums: HashMap::new() }
        };

        let mut hashes = HashMap::new();

        // Parse the checksums line by line
        for line in full_content.lines() {
            // Split on the separator, then check it
            let split = line.split(" - ").collect::<Vec<&str>>();
            if split.is_empty() || split.len() != 2 {
                continue;
            }

            // Add to the list of hashes
            hashes.insert(String::from(split[0]), String::from(split[1]));
        }

        Self {
            checksums: hashes,
        }
    }
}

impl Checksums for ChecksumsV1 {
    fn get_file_checksum(&self, path: &str) -> Option<String> {
        match self.checksums.get(path) {
            Some(s) => Some(String::from(s)),
            None => None,
        }
    }
}