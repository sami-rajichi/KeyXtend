//! `SealedFiles`: the sealed store over one folder, one `<name>.sealed` file per blob, under one data key.
//! The tag covers the blob's name; a blob that will not open becomes `<name>.damaged`, so `load` reports it once.

use crate::consts::KEY_FILE;
use crate::guard::KeyGuard;
use crate::sealer::{Sealer, blob_file};
use kx_module_api::ports::{BlobName, SealedError, SealedStore};
use std::fmt;
use std::path::PathBuf;

/// The sealed store over the folder `dir`; the guard wraps its data key.
pub struct SealedFiles {
    dir: PathBuf,
    sealer: Sealer,
}

impl SealedFiles {
    /// A store over `dir`, which the first save creates.
    #[must_use]
    pub fn new(dir: PathBuf, guard: Box<dyn KeyGuard>) -> Self {
        Self {
            sealer: Sealer::new(dir.join(KEY_FILE), guard),
            dir,
        }
    }
}

impl SealedStore for SealedFiles {
    fn load(&self, name: &BlobName) -> Result<Option<Vec<u8>>, SealedError> {
        let path = blob_file(&self.dir, name);
        self.sealer.load(&path, name.as_str().as_bytes())
    }

    fn save(&self, name: &BlobName, bytes: &[u8]) -> Result<(), SealedError> {
        let path = blob_file(&self.dir, name);
        self.sealer.save(&path, name.as_str().as_bytes(), bytes)
    }
}

impl fmt::Debug for SealedFiles {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // The folder's path holds the account name, and the key is secret, so neither is shown.
        f.debug_struct("SealedFiles").finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests;
