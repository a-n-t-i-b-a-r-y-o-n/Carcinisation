mod archive;
pub mod manifest;
pub mod package;
pub mod checksums;

// Re-export main package version
pub use package::PackageV1;