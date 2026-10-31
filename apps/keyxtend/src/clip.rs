//! Where the clipboard history lives (ADR-0034): a folder of its own under the data folder, with its own sealed key, so
//! the shelf, its day folders and the panel's Open folder all name the same place.

use kx_platform::AppDirs;
use std::path::PathBuf;

pub use kx_platform::CLIP_DIR;

/// The history's folder for the app's `dirs`.
#[must_use]
pub fn folder(dirs: &AppDirs) -> PathBuf {
    dirs.data_dir().path().join(CLIP_DIR)
}
