//! What the snip fakes' tests share: the Snips folder's paths, a clock and a folder that reads it, and a picture.

use super::{FakeSnipFolder, Stored};
use crate::FakeClock;
use kx_module_api::Mono;
use kx_module_api::Redacted;
use kx_module_api::ports::{FileKind, FolderPath, SnipFile};
use kx_test_support::paths::native_path;
use std::sync::Arc;
use std::time::Duration;

/// How long a test waits for another thread; it never sleeps.
pub(super) const WITHIN: Duration = Duration::from_secs(10);

/// The Snips folder, under a folder holding the user's name; `dir` and `file` read it through `native_path`.
pub(super) const SNIPS: &str = "C:/Users/Alex/KeyXtend/snips";
/// Its unsaved part.
pub(super) const UNSAVED: &str = "C:/Users/Alex/KeyXtend/snips/unsaved";
/// A place the user chose, outside the Snips folder.
pub(super) const CHOSEN: &str = "C:/Users/Alex/Pictures";
/// Picture bytes no output may show.
pub(super) const PNG: [u8; 4] = [0x89, 0x7e, 0x3d, 0x51];

/// `path`, written for Windows, as a folder.
pub(super) fn dir(path: &str) -> FolderPath {
    FolderPath::new(native_path(path)).unwrap()
}

/// The file `name` of `kind` in the folder at `path`, spelled as one path written for Windows.
pub(super) fn file(path: &str, name: &str, kind: FileKind) -> SnipFile {
    SnipFile::new(native_path(&format!("{path}/{name}")), kind).unwrap()
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
