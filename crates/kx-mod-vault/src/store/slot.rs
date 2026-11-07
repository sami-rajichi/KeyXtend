//! One sealed blob and the blob a copy that does not read is set aside in, the newest such copy winning. A blob the
//! store set aside, or one that does not read, reads as damaged; one that cannot be set aside is never written over.

use std::sync::Arc;

use kx_module_api::Zeroizing;
use kx_module_api::ports::{BlobName, SealedError, SealedStore};

use super::StoreError;
use super::bytes::Bad;

/// What a load found: the value, if one was saved and read, and whether what was saved was damaged.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Read<T> {
    pub(crate) value: Option<T>,
    pub(crate) damaged: bool,
}

/// A named blob in the sealed store.
pub(crate) struct Slot {
    sealed: Arc<dyn SealedStore>,
    name: &'static str,
    aside: &'static str,
    /// A blob that did not read could not be set aside, so saves are refused.
    held: bool,
}

impl Slot {
    /// Blob `name` in `sealed`, set aside as `aside`.
    pub(crate) fn new(
        sealed: Arc<dyn SealedStore>,
        name: &'static str,
        aside: &'static str,
    ) -> Self {
        Self {
            sealed,
            name,
            aside,
            held: false,
        }
    }

    /// Reads the blob with `decode`.
    ///
    /// # Errors
    /// `Newer` for a newer version's blob, or the sealed store's error but `Damaged`, which reads as damaged.
    pub(crate) fn load<T>(
        &mut self,
        decode: impl FnOnce(&[u8]) -> Result<T, Bad>,
    ) -> Result<Read<T>, StoreError> {
        let read = match self.sealed.load(&BlobName::new(self.name)?) {
            Ok(read) => read,
            Err(SealedError::Damaged) => return Ok(Read::empty(true)),
            Err(error) => return Err(error.into()),
        };
        let Some(bytes) = read.map(Zeroizing::new) else {
            return Ok(Read::empty(false));
        };
        match decode(&bytes) {
            Ok(value) => Ok(Read {
                value: Some(value),
                damaged: false,
            }),
            Err(Bad::Newer) => Err(StoreError::Newer),
            Err(Bad::Damaged) => {
                self.set_aside(&bytes);
                Ok(Read::empty(true))
            }
        }
    }

    /// Saves `bytes` in place of what was saved.
    ///
    /// # Errors
    /// `Held` while a blob that did not read could not be set aside, or the sealed store's error.
    pub(crate) fn save(&self, bytes: &[u8]) -> Result<(), StoreError> {
        if self.held {
            return Err(StoreError::Held);
        }
        Ok(self.sealed.save(&BlobName::new(self.name)?, bytes)?)
    }

    /// Copies `bytes`, a blob that did not read, to its aside blob; when that fails, saves stop for the run.
    fn set_aside(&mut self, bytes: &[u8]) {
        tracing::warn!(blob = self.name, "a blob did not read, so it is set aside");
        let copied = BlobName::new(self.aside).and_then(|aside| self.sealed.save(&aside, bytes));
        if let Err(error) = copied {
            tracing::warn!(blob = self.name, %error, "the unread blob was not set aside");
            self.held = true;
        }
    }
}

impl<T> Read<T> {
    /// Nothing read, damaged or not.
    const fn empty(damaged: bool) -> Self {
        Self {
            value: None,
            damaged,
        }
    }
}
