//! Why a load failed (Ruling 26): a list from a newer version of the app, found before anything was changed, or the
//! shelf's own error.

use std::error::Error;
use std::fmt;

use kx_module_api::ports::SealedError;

/// Why the store could not load the history.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum StoreError {
    /// A list was saved by a newer version of the app; the shelf was left as it was.
    Newer,
    /// The shelf failed, such as `Locked` without the OS key.
    Shelf(SealedError),
}

impl From<SealedError> for StoreError {
    fn from(error: SealedError) -> Self {
        Self::Shelf(error)
    }
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Newer => f.write_str("a clipboard list was saved by a newer version"),
            Self::Shelf(error) => error.fmt(f),
        }
    }
}

impl Error for StoreError {}
