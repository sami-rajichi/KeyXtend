//! The folder reveal port (ADR-0046): a checked folder, shown in the system's file manager, which runs as the user's
//! own program, never with our rights. Only `OpenFolder` reaches it, and a path may hold the user's name, so it never
//! prints.

use super::clip::{PathRoot, SEPARATORS};
use super::models::path_ok_in;
use crate::{Capability, Redacted, ServiceId, ServiceKey};
use std::path::{Path, PathBuf};
use thiserror::Error;

/// The bytes Windows drops from the end of a path's part, so `models.` would name `models`.
const DROPPED: &[u8] = b". ";

/// Why a folder was refused or not shown; no variant names the folder.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Error)]
pub enum FolderError {
    /// It is not a plain absolute path, or `show` found it off a fixed local drive.
    #[error("the path is not a plain local path")]
    BadPath,
    /// No folder is there: nothing, or a file.
    #[error("no folder is there")]
    Missing,
    /// It, or a folder above it, is a link, a junction or another reparse point, which may lead anywhere.
    #[error("the folder is a link")]
    Linked,
    /// The system or its file manager refused, with its error code.
    #[error("the file manager refused to show the folder (code {0})")]
    Refused(i32),
}

/// A checked folder: a plain absolute path (`path_ok`) with no part ending in a dot or a space; it prints redacted.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FolderPath(Redacted<PathBuf>);

impl FolderPath {
    /// Checks and wraps `path`.
    ///
    /// # Errors
    /// `BadPath` for a share, a device, a stream, a dot part, a relative path or anything else `path_ok` refuses, and
    /// for a part ending in a dot or a space, which Windows would read as another name.
    pub fn new(path: PathBuf) -> Result<Self, FolderError> {
        Self::new_in(PathRoot::NATIVE, path)
    }

    /// `new` in `root`'s style, so both styles are tested on every system.
    pub(crate) fn new_in(root: PathRoot, path: PathBuf) -> Result<Self, FolderError> {
        let bytes = path.as_os_str().as_encoded_bytes();
        let mut parts = bytes.split(|b| SEPARATORS.contains(b));
        let dropped = parts.any(|p| p.last().is_some_and(|b| DROPPED.contains(b)));
        if dropped || !path_ok_in(root, &path) {
            return Err(FolderError::BadPath);
        }
        Ok(Self(Redacted::new(path)))
    }

    /// The path, for the adapter that hands it to the file manager.
    #[must_use]
    pub fn expose(&self) -> &Path {
        self.0.expose()
    }
}

/// Shows folders in the system's file manager; the platform provides it.
pub trait FolderReveal: Send + Sync {
    /// Shows `folder` when it is a folder on a fixed local drive with no link in or above it; it may wait on the file
    /// manager, so call it off the UI thread.
    ///
    /// # Errors
    /// `BadPath` off a fixed local drive, `Missing` when no folder is there, `Linked` for a reparse point on the way,
    /// `Refused` when the system says no.
    fn show(&self, folder: &FolderPath) -> Result<(), FolderError>;
}

/// The folder reveal service; it needs `OpenFolder`.
pub struct FolderRevealKey;

impl ServiceKey for FolderRevealKey {
    type Api = dyn FolderReveal;
    const ID: ServiceId = ServiceId::new("folder-reveal");
    const CAPABILITY: Option<Capability> = Some(Capability::OpenFolder);
}

#[cfg(test)]
mod tests;
