mod archive;
pub mod manifest;
pub mod package;
pub mod checksums;
pub mod signature;

// Re-export main package version
pub use package::PackageV1;