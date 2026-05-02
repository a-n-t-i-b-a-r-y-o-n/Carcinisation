use std::ffi::OsStr;
use std::fs::File;
use std::path::{Path, PathBuf};
use anyhow::Result;
use liblzma::read::XzDecoder;
use liblzma::write::{AutoFinishXzEncoder, XzEncoder};
use tar::{Archive, Builder, Entries, Entry};
use crate::package::PackageV1;

type ArchiveFormat = Archive<XzDecoder<File>>;
type ArchiveFormatBuilder = Builder<AutoFinishXzEncoder<File>>;

/// Package file archive internal operations
impl PackageV1 {

    /// Create a package archive and open it for writing
    pub(crate) fn create_archive<S>(output_path: &S) -> Result<ArchiveFormatBuilder>
    where
        S: AsRef<OsStr> + AsRef<Path> + ?Sized
    { 
        let output_file = File::create(output_path)?;
        let compressor = XzEncoder::new_parallel(output_file, 9);
        let tarball = Builder::new(XzEncoder::auto_finish(compressor));
        Ok(tarball)
    }

    /// Open a package archive for reading
    pub(crate) fn open_archive<S>(input_path: &S) -> Result<ArchiveFormat>
    where
        S: AsRef<OsStr> + AsRef<Path> + ?Sized
    {
        let input_file = File::open(input_path)?;
        let decompressor = XzDecoder::new_parallel(input_file);
        let archive = Archive::new(decompressor);
        Ok(archive)
    }

    /// Find an entry in the package archive
    pub(crate) fn find_entry<'e, S>(entries: &mut Entries<'e, XzDecoder<File>>, entry_path: &S)
        -> Option<Entry<'e, XzDecoder<File>>>
    where
        S: AsRef<OsStr> + AsRef<Path> + ?Sized
    {
        let path = Path::new(entry_path);
        // Iterate entries
        entries.find(
            |r| {
                // Entries that are Ok()
                r.as_ref().is_ok_and(
                    |e| {
                        // Entries with an Ok() path
                        e.path().is_ok_and(
                            // Paths that serialize to given one
                            |p| {
                                p.to_str().is_some_and(|p| p == path)
                            }
                        )
                    }
                )
            }
        ).map(|r| r.unwrap())
    }
}
