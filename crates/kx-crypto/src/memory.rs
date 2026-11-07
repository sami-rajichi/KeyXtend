//! The sealed stores held in memory alone, for a hidden run (ADR-0037 decision 2): nothing reaches the disk, so the
//! learned words and copies of that run end with it. Each blob is `Zeroizing`, so it is wiped once let go.

use crate::lock;
use kx_module_api::ports::{BlobName, MAX_BLOB_BYTES, SealedError, SealedStore};
use std::collections::BTreeMap;
use std::fmt;
use std::sync::Mutex;
use zeroize::Zeroizing;

mod shelf;

pub use shelf::MemoryShelf;

/// Blobs by name, each wiped when it is replaced, removed or dropped.
type Blobs = BTreeMap<BlobName, Zeroizing<Vec<u8>>>;

/// A copy of `bytes` to hold; `Io` over `MAX_BLOB_BYTES`, as the file stores refuse it.
fn held(bytes: &[u8]) -> Result<Zeroizing<Vec<u8>>, SealedError> {
    if bytes.len() > MAX_BLOB_BYTES {
        return Err(SealedError::Io);
    }
    Ok(Zeroizing::new(bytes.to_vec()))
}

/// The sealed store in memory: one blob per name, for this run only.
#[derive(Default)]
pub struct MemoryStore {
    blobs: Mutex<Blobs>,
}

impl MemoryStore {
    /// An empty store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl SealedStore for MemoryStore {
    fn load(&self, name: &BlobName) -> Result<Option<Vec<u8>>, SealedError> {
        Ok(lock(&self.blobs).get(name).map(|blob| blob.to_vec()))
    }

    fn save(&self, name: &BlobName, bytes: &[u8]) -> Result<(), SealedError> {
        let blob = held(bytes)?;
        lock(&self.blobs).insert(name.clone(), blob);
        Ok(())
    }
}

impl fmt::Debug for MemoryStore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // The blobs hold the user's words, so neither they nor their names are shown.
        f.debug_struct("MemoryStore").finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests;
