pub trait Checksums: From<Vec<u8>> + Sized {
    /// Get the declared checksum for a given file
    fn get_file_checksum(&self, path: &str) -> Option<String>;
}