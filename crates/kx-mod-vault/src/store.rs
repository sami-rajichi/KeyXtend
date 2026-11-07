//! The store (design "Protection"): the whole notebook as one sealed blob behind `VaultStoreKey`, read at start and
//! saved after each change. A blob the store set aside, or one that opens but does not read, starts the notebook empty
//! with a notice; one from a newer version is never read or written over, and only error kinds are logged.

mod bytes;
mod codec;
mod slot;

pub use codec::{HEAD_LEN, entry_len};

use std::sync::Arc;

use kx_module_api::ports::{SealedError, SealedStore};
use thiserror::Error;

use crate::book::Notebook;
pub(crate) use bytes::{
    Bad, Count, FLAG, LEN, Reader, Sink, put_bytes, put_flag, put_len, put_text,
};
pub(crate) use slot::Slot;

/// The notebook's blob.
pub(crate) const BLOB: &str = "notebook";
/// Where a notebook that opens but does not read is kept, the newest such copy winning.
const ASIDE: &str = "notebook-aside";

/// Why a blob did not load or save.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Error)]
pub enum StoreError {
    /// A newer version saved it, so this one leaves it alone.
    #[error("a newer version saved it")]
    Newer,
    /// A blob that did not read could not be set aside, so nothing is written over it this run.
    #[error("the unread blob could not be set aside")]
    Held,
    /// The sealed store failed.
    #[error(transparent)]
    Sealed(#[from] SealedError),
}

/// What a load found.
#[derive(Debug, PartialEq, Eq)]
pub struct Loaded {
    /// The notebook, empty when nothing was saved or what was saved was damaged.
    pub book: Notebook,
    /// What was saved was damaged, so the module shows `keys::VAULT_DAMAGED`.
    pub damaged: bool,
}

/// The notebook's blob in the sealed store.
pub struct Store {
    slot: Slot,
}

impl Store {
    /// A store over `sealed`.
    #[must_use]
    pub fn new(sealed: Arc<dyn SealedStore>) -> Self {
        Self {
            slot: Slot::new(sealed, BLOB, ASIDE),
        }
    }

    /// Reads the notebook.
    ///
    /// # Errors
    /// `Newer` for a newer version's notebook, or the sealed store's error but `Damaged`, which starts empty.
    pub fn load(&mut self) -> Result<Loaded, StoreError> {
        let read = self.slot.load(codec::decode)?;
        Ok(Loaded {
            book: read.value.map(Notebook::new).unwrap_or_default(),
            damaged: read.damaged,
        })
    }

    /// Saves `book` in place of what was saved.
    ///
    /// # Errors
    /// `Held` while an unread notebook could not be set aside, or the sealed store's error.
    pub fn save(&self, book: &Notebook) -> Result<(), StoreError> {
        self.slot.save(&codec::encode(book.entries()))
    }

    /// The notebook's bytes, the part a backup box holds, in a wiped buffer.
    #[must_use]
    pub fn part(book: &Notebook) -> kx_module_api::Zeroizing<Vec<u8>> {
        codec::encode(book.entries())
    }

    /// The entries of a backup box's notebook part, read as a load reads the notebook.
    pub(crate) fn read_part(bytes: &[u8]) -> Result<Vec<crate::entry::Entry>, Bad> {
        codec::decode(bytes)
    }
}

#[cfg(test)]
mod props;
#[cfg(test)]
mod tests;
