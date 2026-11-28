//! Where snips are written (ADR-0056 decisions 3 and 6): a place the Save dialog answered, reached only through that
//! answer, and the Snips folder, whose `unsaved` part holds recordings until they go. Only `ScreenCapture` reaches the
//! folder, and no path prints.

use super::file::SnipFile;
use crate::ports::FolderPath;
use crate::{Capability, Redacted, ServiceId, ServiceKey};
use std::time::Duration;
use thiserror::Error;

/// Why a snip was not written, listed or deleted; no variant names a path.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Error)]
pub enum SaveError {
    /// A file of another kind, or a video outside the `unsaved` part.
    #[error("the file is not of the asked kind or place")]
    BadPath,
    /// A hidden run made no Snips folder (ADR-0037).
    #[error("there is no Snips folder")]
    NoFolder,
    /// The file, its folder or its drive is gone.
    #[error("the file or its drive is gone")]
    Missing,
    /// The place cannot be written.
    #[error("the place cannot be written")]
    ReadOnly,
    /// The drive is full.
    #[error("the drive is full")]
    Full,
    /// Another error of the file system.
    #[error("the file could not be read or written")]
    Io,
}

/// The file the Save dialog answered; a snip reaches the user's chosen place only through it (edge 6).
pub trait SaveTo: Send {
    /// The answered file, of the kind asked.
    fn file(&self) -> &SnipFile;

    /// Writes the picture `png` there whole, replacing a file the user chose to replace; a failed write leaves the
    /// place as it was.
    ///
    /// # Errors
    /// `BadPath` when the answer is no picture, `Missing`, `ReadOnly` or `Full` for its place, else `Io`.
    fn png(&self, png: &Redacted<Vec<u8>>) -> Result<(), SaveError>;

    /// Copies `video`, a video of the `unsaved` part, there; it may take seconds, so call it off the module's thread.
    ///
    /// # Errors
    /// `BadPath` when the answer is no video or `video` lies elsewhere, `Missing` also for a `video` gone, else as
    /// `png`.
    fn copy(&self, video: &SnipFile) -> Result<(), SaveError>;
}

/// A file of the `unsaved` part, how long ago it was last written, and how long ago it was marked copied.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnsavedFile {
    /// The file.
    pub file: SnipFile,
    /// Its age; zero when the system's clock reads it from the future.
    pub age: Duration,
    /// The age of its copy mark (`SnipFolder::mark_copied`); `None` when it was never copied.
    pub copied: Option<Duration>,
}

/// The Snips folder and its `unsaved` part, where a recording is written while it records; the platform provides it.
pub trait SnipFolder: Send + Sync {
    /// The Snips folder; `None` in a hidden run, which makes none (ADR-0037).
    fn snips(&self) -> Option<FolderPath>;

    /// The pictures and videos directly in the `unsaved` part, with their ages and copy marks, for the start's sweep
    /// (edge 15); links and other files are left alone.
    ///
    /// # Errors
    /// `NoFolder` in a hidden run, `Missing` when the part is gone, else `Io`.
    fn unsaved(&self) -> Result<Vec<UnsavedFile>, SaveError>;

    /// Marks `video`, a video directly in the `unsaved` part, as copied from now, with a small file beside it, so the
    /// mark outlives the app; the video itself is never renamed, as a player may hold it open.
    ///
    /// # Errors
    /// `BadPath` for a picture or a file elsewhere, `NoFolder` in a hidden run, `Missing` when `video` is gone, else as
    /// a write fails.
    fn mark_copied(&self, video: &SnipFile) -> Result<(), SaveError>;

    /// Deletes each copy mark in the `unsaved` part whose video is gone.
    ///
    /// # Errors
    /// `NoFolder` in a hidden run, `Missing` when the part is gone, else `Io`.
    fn prune_marks(&self) -> Result<(), SaveError>;

    /// Deletes `file`, which lies directly in the `unsaved` part (`SnipFile::lies_in`), its copy mark first; one
    /// already gone is no error.
    ///
    /// # Errors
    /// `BadPath` for a file elsewhere, `NoFolder` in a hidden run, `ReadOnly` when it cannot be deleted, else `Io`.
    fn remove(&self, file: &SnipFile) -> Result<(), SaveError>;

    /// Bytes free to us on the Snips folder's drive, for `min_free_mb`, which counts `MB` bytes to the MB (edge 8).
    ///
    /// # Errors
    /// `NoFolder` in a hidden run, `Missing` without its drive, else `Io`.
    fn free(&self) -> Result<u64, SaveError>;
}

/// The Snips folder service; it needs `ScreenCapture`, as its `unsaved` part holds recordings.
pub struct SnipFolderKey;

impl ServiceKey for SnipFolderKey {
    type Api = dyn SnipFolder;
    const ID: ServiceId = ServiceId::new("snip-folder");
    const CAPABILITY: Option<Capability> = Some(Capability::ScreenCapture);
}
