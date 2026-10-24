//! The sealed store: small blobs the platform keeps sealed under the user's OS key (ADR-0010), which only
//! `SecretStore` reaches.

use crate::{Capability, ServiceId, ServiceKey};
use thiserror::Error;

/// The shortest blob name, in bytes.
pub const MIN_BLOB_LEN: usize = 1;
/// The longest blob name, in bytes.
pub const MAX_BLOB_LEN: usize = 32;

/// Why a blob could not be named, loaded or saved.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Error)]
pub enum SealedError {
    /// The name is not lower-case letters, digits and dashes within the length limits.
    #[error("the blob name is not lower-case letters, digits and dashes within the length limits")]
    BadName,
    /// The blob would not open, and the adapter has set its file aside.
    #[error("the sealed blob was damaged and set aside")]
    Damaged,
    /// The OS key that seals blobs is unavailable.
    #[error("the system key that seals blobs is unavailable")]
    Locked,
    /// The file could not be read or written.
    #[error("the sealed blob could not be read or written")]
    Io,
}

/// A checked blob name, such as `predict-en`: `MIN_BLOB_LEN` to `MAX_BLOB_LEN` of `a-z`, `0-9` and `-`.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BlobName(String);

impl BlobName {
    /// Checks and wraps `text`.
    ///
    /// # Errors
    /// `BadName` when `text` has another character or length.
    pub fn new(text: &str) -> Result<Self, SealedError> {
        let plain = |b: u8| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-';
        let sized = (MIN_BLOB_LEN..=MAX_BLOB_LEN).contains(&text.len());
        if sized && text.bytes().all(plain) {
            Ok(Self(text.to_owned()))
        } else {
            Err(SealedError::BadName)
        }
    }

    /// The name as text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Keeps blobs sealed on disk, one per name; the platform provides it.
pub trait SealedStore: Send + Sync {
    /// The opened bytes saved under `name`, or `None` when nothing was saved.
    ///
    /// # Errors
    /// `Damaged` when the blob would not open, `Locked` without the OS key, `Io` when the file cannot be read.
    fn load(&self, name: &BlobName) -> Result<Option<Vec<u8>>, SealedError>;

    /// Seals `bytes` under `name`, in place of what was there.
    ///
    /// # Errors
    /// `Locked` without the OS key, `Io` when the file cannot be written.
    fn save(&self, name: &BlobName, bytes: &[u8]) -> Result<(), SealedError>;
}

/// The sealed store service; it needs `SecretStore`.
pub struct SealedStoreKey;

impl ServiceKey for SealedStoreKey {
    type Api = dyn SealedStore;
    const ID: ServiceId = ServiceId::new("sealed-store");
    const CAPABILITY: Option<Capability> = Some(Capability::SecretStore);
}

#[cfg(test)]
mod tests;
