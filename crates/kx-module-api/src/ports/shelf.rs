//! The sealed shelf (ADR-0034 decision 4): blobs sealed under the user's OS key in named folders, such as one per day,
//! which only `SecretStore` reaches. Folders and blobs are named by `BlobName`, and a blob stays under
//! `MAX_BLOB_BYTES`, as in the sealed store.

use super::{BlobName, SealedError};
use crate::{Capability, ServiceId, ServiceKey};

/// The shelf's room on its drive, for the storage line.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Room {
    /// Bytes the shelf's files take.
    pub used: u64,
    /// Bytes free on the shelf's drive.
    pub free: u64,
    /// The drive's name to show, such as `D:`.
    pub drive: String,
}

/// Keeps sealed blobs in folders, one per name in each; the platform provides it.
pub trait SealedShelf: Send + Sync {
    /// The opened bytes saved as `name` in `folder`, or `None` when nothing was saved.
    ///
    /// # Errors
    /// `Damaged` when the blob would not open, here or moved from elsewhere, `Locked` without the OS key, `Io` when
    /// the file cannot be read.
    fn load(&self, folder: &BlobName, name: &BlobName) -> Result<Option<Vec<u8>>, SealedError>;

    /// Seals `bytes` as `name` in `folder`, in place of what was there, and makes the folder when it is missing.
    ///
    /// # Errors
    /// `Locked` without the OS key, `Io` when the file cannot be written or `bytes` is over `MAX_BLOB_BYTES`.
    fn save(&self, folder: &BlobName, name: &BlobName, bytes: &[u8]) -> Result<(), SealedError>;

    /// Deletes `name` from `folder` at once, with no recycle bin; a missing blob is no error.
    ///
    /// # Errors
    /// `Io` when the file cannot be deleted.
    fn remove(&self, folder: &BlobName, name: &BlobName) -> Result<(), SealedError>;

    /// Deletes `folder` and every blob in it at once; a missing folder is no error.
    ///
    /// # Errors
    /// `Io` when a file cannot be deleted.
    fn drop_folder(&self, folder: &BlobName) -> Result<(), SealedError>;

    /// The folders on the shelf, sorted; a folder whose name is not a `BlobName` is left out.
    ///
    /// # Errors
    /// `Io` when the shelf cannot be listed.
    fn folders(&self) -> Result<Vec<BlobName>, SealedError>;

    /// The blob names in `folder`, sorted, or none when it is missing; a file whose name is not a blob's is left out.
    ///
    /// # Errors
    /// `Io` when the folder cannot be listed.
    fn names(&self, folder: &BlobName) -> Result<Vec<BlobName>, SealedError>;

    /// What the shelf takes and what its drive has free.
    ///
    /// # Errors
    /// `Io` when the drive cannot be read.
    fn room(&self) -> Result<Room, SealedError>;
}

/// The sealed shelf service; it needs `SecretStore`.
pub struct SealedShelfKey;

impl ServiceKey for SealedShelfKey {
    type Api = dyn SealedShelf;
    const ID: ServiceId = ServiceId::new("sealed-shelf");
    const CAPABILITY: Option<Capability> = Some(Capability::SecretStore);
}

#[cfg(test)]
mod tests;
