//! The file ports (ADR-0056 decisions 6 and 10): a snip's checked file, the system's Save and folder dialogs, which
//! only `SaveFile` reaches, and a video played in the user's own player, which only `OpenFolder` reaches. A path may
//! hold the user's name, so it never prints.

use super::store::SaveTo;
use super::waits::PickLimits;
use crate::ports::FolderPath;
use crate::ports::clip::{PathRoot, SEPARATORS};
use crate::{Capability, Redacted, ServiceId, ServiceKey};
use std::path::{Path, PathBuf};
use thiserror::Error;

listed! {
    /// What a snip is saved as.
    pub enum FileKind ("Both kinds.") {
        /// A picture.
        Png,
        /// A video.
        Mp4,
    }
}

impl FileKind {
    /// Its extension in lower case, the frames' own.
    #[must_use]
    pub const fn ext(self) -> &'static str {
        match self {
            Self::Png => kx_ipc::PNG_EXT,
            Self::Mp4 => kx_ipc::MP4_EXT,
        }
    }
}

/// `path`'s non-empty parts between separators of either kind; a Unix path holds no `\` (`PathRoot`).
fn parts(path: &Path) -> impl Iterator<Item = &[u8]> {
    let bytes = path.as_os_str().as_encoded_bytes();
    bytes
        .split(|b| SEPARATORS.contains(b))
        .filter(|p| !p.is_empty())
}

/// What follows the last dot of `name`; `None` without a dot, or when the dot leads it, as `.png` has no extension.
fn extension(name: &[u8]) -> Option<&[u8]> {
    let dot = name.iter().rposition(|b| *b == b'.').filter(|d| *d > 0)?;
    name.get(dot + 1..)
}

/// A snip's file: a path `FolderPath` takes, whose name ends in its kind's extension in any case. A path is absolute
/// in this system's style (`PathRoot`), so it means the same to every check.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SnipFile {
    /// The path, which may hold the user's name.
    path: Redacted<PathBuf>,
    /// Its kind.
    kind: FileKind,
}

impl SnipFile {
    /// `path` as a file of `kind`; `None` unless `FolderPath::new` takes it and its extension is the kind's.
    #[must_use]
    pub fn new(path: PathBuf, kind: FileKind) -> Option<Self> {
        Self::new_in(PathRoot::NATIVE, path, kind)
    }

    /// `new` in `root`'s style, so both styles are tested on every system.
    pub(crate) fn new_in(root: PathRoot, path: PathBuf, kind: FileKind) -> Option<Self> {
        let ext = parts(&path).last().and_then(extension);
        let named = ext.is_some_and(|e| e.eq_ignore_ascii_case(kind.ext().as_bytes()));
        let plain = named && FolderPath::new_in(root, path.clone()).is_ok();
        plain.then(|| Self {
            path: Redacted::new(path),
            kind,
        })
    }

    /// The path, for the adapter that opens or writes it.
    #[must_use]
    pub fn expose(&self) -> &Path {
        self.path.expose()
    }

    /// Its kind.
    #[must_use]
    pub const fn kind(&self) -> FileKind {
        self.kind
    }

    /// True when it lies directly in `folder`, as a video of the `unsaved` part must before an adapter touches it; only
    /// a drive letter's case does not count, as on Windows.
    #[must_use]
    pub fn lies_in(&self, folder: &FolderPath) -> bool {
        self.lies_in_at(PathRoot::NATIVE, folder)
    }

    /// `lies_in` in `root`'s style, so both styles are tested on every system.
    pub(crate) fn lies_in_at(&self, root: PathRoot, folder: &FolderPath) -> bool {
        let mut inside: Vec<&[u8]> = parts(self.expose()).collect();
        inside.pop();
        let around: Vec<&[u8]> = parts(folder.expose()).collect();
        let same = |a: &[u8], b: &[u8]| match root {
            PathRoot::Drive => a.eq_ignore_ascii_case(b),
            PathRoot::Slash => a == b,
        };
        match (inside.split_first(), around.split_first()) {
            (Some((head, rest)), Some((their, theirs))) => same(head, their) && rest == theirs,
            _ => false,
        }
    }
}

/// Why a dialog gave no answer; no variant names a path.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Error)]
pub enum AskError {
    /// The answer is no plain absolute path of the asked kind, so nothing is written there.
    #[error("the dialog's answer is not a plain absolute path of the asked kind")]
    BadPath,
    /// The dialog could not show.
    #[error("the dialog could not show")]
    Failed,
    /// The worker ended, answered badly or gave no answer.
    #[error("the dialog's worker ended")]
    Gone,
    /// The limits' value of this name is outside its range.
    #[error("the dialog's {0} is outside its range")]
    Range(&'static str),
}

/// Shows the system's own Save and folder dialogs outside our process, in a job held to `PickLimits`; the platform
/// provides it.
pub trait FileAsk: Send + Sync {
    /// Shows the Save dialog at `proposed`'s folder with its name and kind, and answers the file chosen, of that kind,
    /// through which alone the snip is written there, or `None` on Cancel; it waits on the user, so call it off the
    /// module's thread.
    ///
    /// # Errors
    /// `Range` when `limits.check` refuses them, `BadPath` for an answer of another kind, no plain absolute path or one
    /// in the Snips folder's unsaved part, `Failed` or `Gone` when no dialog showed.
    fn save(
        &self,
        proposed: &SnipFile,
        limits: &PickLimits,
    ) -> Result<Option<Box<dyn SaveTo>>, AskError>;

    /// Shows the folder dialog at `at`, and answers the folder chosen, or `None` on Cancel.
    ///
    /// # Errors
    /// As `save`.
    fn folder(&self, at: &FolderPath, limits: &PickLimits) -> Result<Option<FolderPath>, AskError>;
}

/// The dialogs service; it needs `SaveFile`.
pub struct FileAskKey;

impl ServiceKey for FileAskKey {
    type Api = dyn FileAsk;
    const ID: ServiceId = ServiceId::new("file-ask");
    const CAPABILITY: Option<Capability> = Some(Capability::SaveFile);
}

/// Why a video did not play; no variant names it.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Error)]
pub enum PlayError {
    /// It is no video of the Snips folder.
    #[error("the file is not a video of the Snips folder")]
    BadPath,
    /// No video is there.
    #[error("no video is there")]
    Missing,
    /// The system or its file manager refused, with its error code.
    #[error("the file manager refused to play the video (code {0})")]
    Refused(i32),
}

/// Plays Snip's videos in the user's own player; the platform provides it.
pub trait MediaOpen: Send + Sync {
    /// Plays `video`, an MP4 directly in the Snips folder or its unsaved part, through the running file manager as
    /// `FolderReveal` shows a folder, so nothing starts as our child; call it off the UI thread.
    ///
    /// # Errors
    /// `BadPath` for a picture or a file elsewhere, also in a hidden run, `Missing` when it is gone, `Refused` when the
    /// system says no.
    fn play(&self, video: &SnipFile) -> Result<(), PlayError>;
}

/// The player service; it needs `OpenFolder` (ADR-0056 decision 10).
pub struct MediaOpenKey;

impl ServiceKey for MediaOpenKey {
    type Api = dyn MediaOpen;
    const ID: ServiceId = ServiceId::new("media-open");
    const CAPABILITY: Option<Capability> = Some(Capability::OpenFolder);
}
