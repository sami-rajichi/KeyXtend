//! Listing the shelf: folders and sealed files named by a `BlobName`; anything else on disk is left out.

use crate::consts::SEALED_EXT;
use kx_module_api::ports::{BlobName, SealedError};
use std::ffi::OsStr;
use std::fs::{self, DirEntry};
use std::io;
use std::path::Path;

/// The entries of `dir` that `pick` keeps, sorted; none when `dir` is missing.
fn listed<T: Ord>(
    dir: &Path,
    pick: impl Fn(&DirEntry) -> io::Result<Option<T>>,
) -> Result<Vec<T>, SealedError> {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(_) => return Err(SealedError::Io),
    };
    let mut found = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|_| SealedError::Io)?;
        found.extend(pick(&entry).map_err(|_| SealedError::Io)?);
    }
    found.sort();
    Ok(found)
}

/// `text` as a `BlobName`, if it is one.
fn blob_name(text: &OsStr) -> Option<BlobName> {
    BlobName::new(text.to_str()?).ok()
}

/// The folders under `root` whose names are `BlobName`s.
pub(super) fn folders(root: &Path) -> Result<Vec<BlobName>, SealedError> {
    listed(root, |entry| {
        let dir = entry.file_type()?.is_dir();
        Ok(dir.then(|| blob_name(&entry.file_name())).flatten())
    })
}

/// The sealed blobs in `dir`, with each file's size.
pub(super) fn blobs(dir: &Path) -> Result<Vec<(BlobName, u64)>, SealedError> {
    listed(dir, |entry| {
        let path = entry.path();
        let sealed = path.extension() == Some(OsStr::new(SEALED_EXT));
        let name = path.file_stem().and_then(blob_name);
        match name {
            Some(name) if sealed && entry.file_type()?.is_file() => {
                Ok(Some((name, entry.metadata()?.len())))
            }
            _ => Ok(None),
        }
    })
}
