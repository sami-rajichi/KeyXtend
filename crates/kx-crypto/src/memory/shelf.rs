//! `MemoryShelf`: the sealed shelf in memory for a hidden run, whose copies end with it; it asks the disk only about
//! its drive's room.

use super::{Blobs, held};
use crate::lock;
use crate::shelf::{DiskRoom, nearest};
use kx_module_api::ports::{BlobName, Room, SealedError, SealedShelf};
use std::collections::BTreeMap;
use std::fmt;
use std::path::PathBuf;
use std::sync::Mutex;

/// The sealed shelf in memory: folders of blobs, never written to disk.
/// Its room's `used` is the bytes it holds, and `disk` tells the free bytes and the name of the drive of `at`.
pub struct MemoryShelf {
    folders: Mutex<BTreeMap<BlobName, Blobs>>,
    at: PathBuf,
    disk: Box<dyn DiskRoom>,
}

impl MemoryShelf {
    /// An empty shelf whose room asks `disk` about the drive of `at`, where it writes nothing.
    #[must_use]
    pub fn new(at: PathBuf, disk: Box<dyn DiskRoom>) -> Self {
        Self {
            folders: Mutex::default(),
            at,
            disk,
        }
    }

    /// The bytes every blob holds together.
    fn used(&self) -> u64 {
        let folders = lock(&self.folders);
        let sizes = folders.values().flat_map(BTreeMap::values);
        sizes
            .map(|blob| u64::try_from(blob.len()).unwrap_or(u64::MAX))
            .fold(0, u64::saturating_add)
    }
}

impl SealedShelf for MemoryShelf {
    fn load(&self, folder: &BlobName, name: &BlobName) -> Result<Option<Vec<u8>>, SealedError> {
        let folders = lock(&self.folders);
        let blob = folders.get(folder).and_then(|blobs| blobs.get(name));
        Ok(blob.map(|blob| blob.to_vec()))
    }

    fn save(&self, folder: &BlobName, name: &BlobName, bytes: &[u8]) -> Result<(), SealedError> {
        let blob = held(bytes)?;
        let mut folders = lock(&self.folders);
        folders
            .entry(folder.clone())
            .or_default()
            .insert(name.clone(), blob);
        Ok(())
    }

    fn remove(&self, folder: &BlobName, name: &BlobName) -> Result<(), SealedError> {
        if let Some(blobs) = lock(&self.folders).get_mut(folder) {
            blobs.remove(name);
        }
        Ok(())
    }

    fn drop_folder(&self, folder: &BlobName) -> Result<(), SealedError> {
        lock(&self.folders).remove(folder);
        Ok(())
    }

    fn folders(&self) -> Result<Vec<BlobName>, SealedError> {
        Ok(lock(&self.folders).keys().cloned().collect())
    }

    fn names(&self, folder: &BlobName) -> Result<Vec<BlobName>, SealedError> {
        let folders = lock(&self.folders);
        let blobs = folders.get(folder);
        Ok(blobs.map_or_else(Vec::new, |b| b.keys().cloned().collect()))
    }

    fn room(&self) -> Result<Room, SealedError> {
        let used = self.used();
        let at = nearest(&self.at);
        let free = self.disk.free(at).map_err(|_| SealedError::Io)?;
        let drive = self.disk.drive(at).map_err(|_| SealedError::Io)?;
        Ok(Room { used, free, drive })
    }

    /// The free bytes alone, as the file shelf asks them, so a copy never waits on the drive's name.
    fn free(&self) -> Result<u64, SealedError> {
        let at = nearest(&self.at);
        self.disk.free(at).map_err(|_| SealedError::Io)
    }
}

impl fmt::Debug for MemoryShelf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // The place's path holds the account name, and the blobs are the user's copies, so neither is shown.
        f.debug_struct("MemoryShelf").finish_non_exhaustive()
    }
}
