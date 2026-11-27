//! The system's Save and folder dialogs (ADR-0056 decision 6), for the worker's `pick` job: each opens above our
//! topmost windows, adds nothing to the recent lists and answers a path or a Cancel. The job checks a path with the
//! ask's own `PickAsk::check` before it tells the host.

#[cfg(not(windows))]
mod none;
#[cfg(windows)]
mod win;

pub use kx_ipc::FileKind;
use std::fmt;
use thiserror::Error;

/// The system's dialogs for the `pick` job.
#[cfg(not(windows))]
pub use none::NoDialogs as LiveDialogs;
/// The system's dialogs for the `pick` job.
#[cfg(windows)]
pub use win::WinDialogs as LiveDialogs;

/// What a dialog answered.
#[derive(Clone, PartialEq, Eq)]
pub enum Picked {
    /// The path the user chose, not yet checked.
    Path(String),
    /// The user cancelled the dialog (edge 6).
    Cancelled,
}

impl fmt::Debug for Picked {
    /// Shows a path's length, never the path, which may name the user.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Path(path) => f.debug_tuple("Path").field(&path.len()).finish(),
            Self::Cancelled => f.write_str("Cancelled"),
        }
    }
}

/// No dialog showed, or its answer could not be read; it keeps the system's code and names no path.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Error)]
#[error("no dialog showed (system code {code:#010x})")]
pub struct Unshown {
    /// The system's error code: an `HRESULT`'s bits on Windows.
    pub code: u32,
}

/// The system's dialogs, or a script in tests.
pub trait Dialogs {
    /// Shows the Save dialog at `folder`, proposing `name` and offering only `kind`.
    ///
    /// # Errors
    /// `Unshown` when no dialog showed or its answer could not be read.
    fn save(&self, folder: &str, name: &str, kind: FileKind) -> Result<Picked, Unshown>;

    /// Shows the folder dialog at `at`.
    ///
    /// # Errors
    /// As `save`.
    fn folder(&self, at: &str) -> Result<Picked, Unshown>;
}

/// The file type a Save dialog offers for `kind`, which also names it in the type list: no word to translate.
#[must_use]
pub fn pattern(kind: FileKind) -> String {
    format!("*.{}", kind.ext())
}

#[cfg(test)]
mod tests;
