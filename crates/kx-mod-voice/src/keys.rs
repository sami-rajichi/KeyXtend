//! The cloud keys (ADR-0047 decision 10): each the user's own, sealed as one blob per service in `keys` on the voice
//! shelf with its own key, and handed out as a `Secret` wiped after use. A hidden run reads and writes none.

use std::collections::BTreeSet;
use std::fmt;
use std::sync::Arc;

use kx_module_api::ports::{BlobName, CloudAsk, SealedError, SealedShelf};
use kx_module_api::{Redacted, Secret, Zeroizing};
use thiserror::Error;

/// The shelf's folder of keys, one blob per service, named by its id.
pub const KEYS: &str = "keys";
/// The warning logged when a key could not be read before an upload; it holds the error's kind, never the key.
pub const KEY_UNREAD: &str = "a cloud key could not be read";

/// Why a key was not read, set or removed; no variant holds the key.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Error)]
pub enum KeyError {
    /// A hidden run keeps no key (ADR-0037).
    #[error("a hidden run keeps no key")]
    Hidden,
    /// The key is empty, too long, or holds a byte a header may not carry.
    #[error("the key does not fit a request's header")]
    Unfit,
    /// The shelf failed: `BadName` for an id that names no blob, `Damaged` for a blob that is no key, or `Locked`.
    #[error(transparent)]
    Shelf(#[from] SealedError),
}

/// The keys on the voice shelf; in a hidden run it has no shelf, so it reads and writes nothing.
pub struct Keys {
    shelf: Option<Arc<dyn SealedShelf>>,
}

impl Keys {
    /// The keys on `shelf`, or none at all in a `hidden` run.
    #[must_use]
    pub fn new(shelf: Arc<dyn SealedShelf>, hidden: bool) -> Self {
        Self {
            shelf: (!hidden).then_some(shelf),
        }
    }

    /// Service `id`'s key; `None` when none is set or the run is hidden.
    ///
    /// # Errors
    /// `Shelf` with the shelf's error, or `Damaged` for a blob that is no key.
    pub fn get(&self, id: &str) -> Result<Option<Secret>, KeyError> {
        let Some(sealed) = &self.shelf else {
            return Ok(None);
        };
        let Some(bytes) = sealed.load(&folder()?, &BlobName::new(id)?)? else {
            return Ok(None);
        };
        // The buffer moves into the key, or into a wiped buffer when it is no text.
        let text = match String::from_utf8(bytes) {
            Ok(text) => text,
            Err(odd) => {
                drop(Zeroizing::new(odd.into_bytes()));
                String::new()
            }
        };
        let key: Secret = Redacted::new(Zeroizing::new(text));
        if CloudAsk::key_fits(key.expose()) {
            Ok(Some(key))
        } else {
            Err(SealedError::Damaged.into())
        }
    }

    /// True when service `id` has a key blob, by the folder's names alone, so no key is opened; false in a hidden run.
    ///
    /// # Errors
    /// `Shelf` with `BadName` for an id that names no blob, or the shelf's error.
    pub fn has(&self, id: &str) -> Result<bool, KeyError> {
        let Some(sealed) = &self.shelf else {
            return Ok(false);
        };
        let name = BlobName::new(id)?;
        Ok(sealed.names(&folder()?)?.contains(&name))
    }

    /// The services with a key blob, from the folder's names alone, so no key is opened; none in a hidden run.
    ///
    /// # Errors
    /// `Shelf` with the shelf's error.
    pub fn listed(&self) -> Result<BTreeSet<String>, KeyError> {
        let Some(sealed) = &self.shelf else {
            return Ok(BTreeSet::new());
        };
        let names = sealed.names(&folder()?)?;
        Ok(names.iter().map(|n| n.as_str().to_owned()).collect())
    }

    /// How many characters service `id`'s key holds, for its dots; 0 for none or a hidden run. The key is wiped as
    /// this returns.
    ///
    /// # Errors
    /// As `get`.
    pub fn dots(&self, id: &str) -> Result<u32, KeyError> {
        let key = self.get(id)?;
        let count = key.map_or(0, |k| k.expose().chars().count());
        Ok(u32::try_from(count).unwrap_or(u32::MAX))
    }

    /// Seals `key` as service `id`'s, in place of the one before; the key is wiped as this returns.
    ///
    /// # Errors
    /// `Hidden` in a hidden run, `Unfit` for a key no header could carry, else the shelf's error.
    pub fn set(&self, id: &str, key: Secret) -> Result<(), KeyError> {
        let sealed = self.shelf.as_ref().ok_or(KeyError::Hidden)?;
        if !CloudAsk::key_fits(key.expose()) {
            return Err(KeyError::Unfit);
        }
        let name = BlobName::new(id)?;
        let saved = sealed.save(&folder()?, &name, key.expose().as_bytes());
        drop(key);
        saved.map_err(KeyError::Shelf)
    }

    /// Deletes service `id`'s key blob at once; a missing one is no error.
    ///
    /// # Errors
    /// `Hidden` in a hidden run, else the shelf's error.
    pub fn remove(&self, id: &str) -> Result<(), KeyError> {
        let sealed = self.shelf.as_ref().ok_or(KeyError::Hidden)?;
        Ok(sealed.remove(&folder()?, &BlobName::new(id)?)?)
    }
}

/// The keys' folder on the shelf.
fn folder() -> Result<BlobName, SealedError> {
    BlobName::new(KEYS)
}

/// Says only whether the run is hidden, never a key.
impl fmt::Debug for Keys {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Keys")
            .field("hidden", &self.shelf.is_none())
            .finish()
    }
}

#[cfg(test)]
mod tests;
