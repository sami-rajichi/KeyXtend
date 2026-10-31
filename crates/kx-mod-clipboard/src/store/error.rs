//! Why a load failed: a list from a newer version of the app, found before anything was changed, or the
//! shelf's own error.

use kx_module_api::ports::SealedError;
use thiserror::Error;

/// Why the store could not load the history.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Error)]
pub enum StoreError {
    /// A list was saved by a newer version of the app; the shelf was left as it was.
    #[error("a clipboard list was saved by a newer version")]
    Newer,
    /// The shelf failed, such as `Locked` without the OS key.
    #[error(transparent)]
    Shelf(#[from] SealedError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_shelf_error_reads_as_the_shelf_says_and_a_newer_list_says_so() {
        let shelf = StoreError::from(SealedError::Locked);
        assert_eq!(shelf, StoreError::Shelf(SealedError::Locked));
        assert_eq!(shelf.to_string(), SealedError::Locked.to_string());
        assert!(StoreError::Newer.to_string().contains("newer version"));
    }
}
