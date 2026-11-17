//! The voice shelf behind a watch that counts each load or listing of the cloud keys' folder, so a test tells a key's
//! read from the store's own reads.

use kx_mod_voice::keys::KEYS;
use kx_module_api::ports::{BlobName, Room, SealedError, SealedShelf};
use kx_platform_fake::FakeShelf;
use std::sync::atomic::{AtomicUsize, Ordering};

/// A fake shelf whose key reads are counted.
pub struct Watched {
    shelf: FakeShelf,
    reads: AtomicUsize,
}

impl Watched {
    /// Watches `shelf`, a clone of the rig's.
    pub const fn new(shelf: FakeShelf) -> Self {
        Self {
            shelf,
            reads: AtomicUsize::new(0),
        }
    }

    /// The loads and listings of the keys' folder so far.
    pub fn key_reads(&self) -> usize {
        self.reads.load(Ordering::SeqCst)
    }

    /// Counts a read of `folder` when it is the keys' folder.
    fn note(&self, folder: &BlobName) {
        if folder.as_str() == KEYS {
            self.reads.fetch_add(1, Ordering::SeqCst);
        }
    }
}

impl SealedShelf for Watched {
    fn load(&self, folder: &BlobName, name: &BlobName) -> Result<Option<Vec<u8>>, SealedError> {
        self.note(folder);
        self.shelf.load(folder, name)
    }

    fn save(&self, folder: &BlobName, name: &BlobName, bytes: &[u8]) -> Result<(), SealedError> {
        self.shelf.save(folder, name, bytes)
    }

    fn remove(&self, folder: &BlobName, name: &BlobName) -> Result<(), SealedError> {
        self.shelf.remove(folder, name)
    }

    fn drop_folder(&self, folder: &BlobName) -> Result<(), SealedError> {
        self.shelf.drop_folder(folder)
    }

    fn folders(&self) -> Result<Vec<BlobName>, SealedError> {
        self.shelf.folders()
    }

    fn names(&self, folder: &BlobName) -> Result<Vec<BlobName>, SealedError> {
        self.note(folder);
        self.shelf.names(folder)
    }

    fn room(&self) -> Result<Room, SealedError> {
        self.shelf.room()
    }

    fn free(&self) -> Result<u64, SealedError> {
        self.shelf.free()
    }
}
