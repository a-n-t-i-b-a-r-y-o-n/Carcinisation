use anyhow::Result;

pub trait Checksums: From<Vec<u8>> + Sized {
    /// Generate a compatible checksum
    fn generate_checksum<C>(content: C) -> String
    where C: AsRef<[u8]>;
    
    /// Check file content against the declared
    fn validate_file_checksum<C>(&self, path: &str, content: C) -> Result<String>
    where C: AsRef<[u8]>;
}