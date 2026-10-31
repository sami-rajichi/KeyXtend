//! `SealedShelfFiles`: the sealed shelf over one root folder (ADR-0034 decision 4), with one key file in the root
//! and one `<name>.sealed` file per blob in each `<folder>`. The tag covers the folder and the name, so a file
//! moved elsewhere will not open; deletes are permanent.

use crate::consts::{AAD_SEP, KEY_FILE};
use crate::guard::KeyGuard;
use crate::sealer::{Sealer, blob_file};
use kx_module_api::ports::{BlobName, Room, SealedError, SealedShelf};
use std::fmt;
use std::fs;
use std::io;
use std::path::PathBuf;

mod list;
mod room;

pub use room::DiskRoom;

/// The sealed shelf over the folder `root`; the guard wraps its data key and `disk` reads its drive.
/// Its room's `used` is the sealed blobs' file sizes, seal overhead in, and leaves out the key, damaged and temp files.
pub struct SealedShelfFiles {
    root: PathBuf,
    sealer: Sealer,
    disk: Box<dyn DiskRoom>,
}

impl SealedShelfFiles {
    /// A shelf over `root`, which the first save creates.
    #[must_use]
    pub fn new(root: PathBuf, guard: Box<dyn KeyGuard>, disk: Box<dyn DiskRoom>) -> Self {
        Self {
            sealer: Sealer::new(root.join(KEY_FILE), guard),
            root,
            disk,
        }
    }

    /// The folder `folder` under the root.
    fn dir(&self, folder: &BlobName) -> PathBuf {
        self.root.join(folder.as_str())
    }

    /// The bytes the sealed blobs of every folder take.
    fn used(&self) -> Result<u64, SealedError> {
        let mut used: u64 = 0;
        for folder in list::folders(&self.root)? {
            for (_, size) in list::blobs(&self.dir(&folder))? {
                used = used.saturating_add(size);
            }
        }
        Ok(used)
    }
}

/// The AAD of blob `name` in `folder`: both names around `AAD_SEP`, so each pair gives other bytes.
fn aad(folder: &BlobName, name: &BlobName) -> Vec<u8> {
    let (folder, name) = (folder.as_str().as_bytes(), name.as_str().as_bytes());
    [folder, &[AAD_SEP], name].concat()
}

impl SealedShelf for SealedShelfFiles {
    fn load(&self, folder: &BlobName, name: &BlobName) -> Result<Option<Vec<u8>>, SealedError> {
        let path = blob_file(&self.dir(folder), name);
        self.sealer.load(&path, &aad(folder, name))
    }

    fn save(&self, folder: &BlobName, name: &BlobName, bytes: &[u8]) -> Result<(), SealedError> {
        let path = blob_file(&self.dir(folder), name);
        self.sealer.save(&path, &aad(folder, name), bytes)
    }

    fn remove(&self, folder: &BlobName, name: &BlobName) -> Result<(), SealedError> {
        self.sealer.remove(&blob_file(&self.dir(folder), name))
    }

    fn drop_folder(&self, folder: &BlobName) -> Result<(), SealedError> {
        // `remove_dir_all` deletes a link to a folder, never what it points to.
        match self.sealer.alone(|| fs::remove_dir_all(self.dir(folder))) {
            Err(e) if e.kind() != io::ErrorKind::NotFound => Err(SealedError::Io),
            _ => Ok(()),
        }
    }

    fn folders(&self) -> Result<Vec<BlobName>, SealedError> {
        self.sealer.alone(|| list::folders(&self.root))
    }

    fn names(&self, folder: &BlobName) -> Result<Vec<BlobName>, SealedError> {
        let blobs = self.sealer.alone(|| list::blobs(&self.dir(folder)))?;
        Ok(blobs.into_iter().map(|(name, _)| name).collect())
    }

    fn room(&self) -> Result<Room, SealedError> {
        let used = self.sealer.alone(|| self.used())?;
        let at = room::nearest(&self.root);
        let free = self.disk.free(at).map_err(|_| SealedError::Io)?;
        let drive = self.disk.drive(at).map_err(|_| SealedError::Io)?;
        Ok(Room { used, free, drive })
    }

    /// The drive alone, outside the sealer's lock, so a copy never waits on a walk of every file.
    fn free(&self) -> Result<u64, SealedError> {
        let at = room::nearest(&self.root);
        self.disk.free(at).map_err(|_| SealedError::Io)
    }
}

impl fmt::Debug for SealedShelfFiles {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // The root's path holds the account name, and the key is secret, so neither is shown.
        f.debug_struct("SealedShelfFiles").finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests;
