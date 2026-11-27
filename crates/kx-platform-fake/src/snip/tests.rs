//! What the snip fakes' tests share: the Snips folder's paths, a clock and a folder that reads it, and a picture.

use super::{FakeSnipFolder, Stored};
use crate::FakeClock;
use kx_module_api::Mono;
use kx_module_api::Redacted;
use kx_module_api::ports::{FileKind, FolderPath, SnipFile};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

/// How long a test waits for another thread; it never sleeps.
pub(super) const WITHIN: Duration = Duration::from_secs(10);

/// The Snips folder, under a folder holding the user's name; forward slashes read as a drive path on every system.
pub(super) const SNIPS: &str = "C:/Users/Alex/KeyXtend/snips";
/// Its unsaved part.
pub(super) const UNSAVED: &str = "C:/Users/Alex/KeyXtend/snips/unsaved";
/// A place the user chose, outside the Snips folder.
pub(super) const CHOSEN: &str = "C:/Users/Alex/Pictures";
/// Picture bytes no output may show.
pub(super) const PNG: [u8; 4] = [0x89, 0x7e, 0x3d, 0x51];

/// `path` as a folder.
pub(super) fn dir(path: &str) -> FolderPath {
    FolderPath::new(PathBuf::from(path)).unwrap()
}

/// The file `name` of `kind` in the folder at `path`.
pub(super) fn file(path: &str, name: &str, kind: FileKind) -> SnipFile {
    SnipFile::new(PathBuf::from(path).join(name), kind).unwrap()
}

/// A clock at its start and a folder that reads it.
pub(super) fn rig() -> (FakeClock, FakeSnipFolder) {
    let clock = FakeClock::new(Mono::default());
    let folder = FakeSnipFolder::new(Arc::new(clock.clone()), dir(SNIPS), dir(UNSAVED));
    (clock, folder)
}

/// The picture `PNG`.
pub(super) fn png() -> Stored {
    Stored::Png(Redacted::new(PNG.to_vec()))
}
