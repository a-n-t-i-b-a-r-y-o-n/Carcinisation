use std::collections::HashMap;
use std::str::from_utf8;

use anyhow::{anyhow, Result};
use hex;
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};

use crate::package::checksums::Checksums;

/// Basic checksums file containing a SHA3-256 hash of each file.
///
/// Expected content shape:
/// ```text
/// relative/path/to/fileA - abc123def456
/// relative/path/to/fileB - 098fed765cba
/// ```
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct ChecksumsV1 {
    pub raw_bytes: Vec<u8>,
    pub checksums: HashMap<String, String>,
}

impl From<Vec<u8>> for ChecksumsV1 {
    fn from(value: Vec<u8>) -> Self {
        let full_content = from_utf8(&value).unwrap_or("");
        if full_content.is_empty() {
            return Self {
                raw_bytes: value,
                checksums: HashMap::new()
            }
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
            raw_bytes: value,
            checksums: hashes,
        }
    }
}

impl Checksums for ChecksumsV1 {
    fn generate_checksum<C>(content: C) -> String
    where
        C: AsRef<[u8]>
    {
        // Hash the given contents
        let mut hasher = Sha3_256::new();
        hasher.update(content);
        let hash = hasher.finalize().0;

        // Get the actual checksum in lowercase ASCII
        hex::encode(&hash)
    }

    fn validate_file_checksum<C>(&self, path: &str, content: C) -> Result<String>
    where
        C: AsRef<[u8]>
    {
        // Get the checksum in lowercase ASCII
        let desired_checksum = match self.checksums.get(path) {
            Some(c) => c.to_ascii_lowercase(),
            None => return Err(anyhow!("No checksum given for file: {}", path)),
        };

        // Calculate the actual checksum
        let actual_checksum = Self::generate_checksum(content);

        // Return the checksum if they match
        if desired_checksum == actual_checksum {
            Ok(actual_checksum)
        } else {
            Err(anyhow!("Checksums do not match! {} != {}", desired_checksum, actual_checksum))
        }
    }
}