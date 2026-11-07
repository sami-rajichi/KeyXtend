//! Where Quick-fill's notebook lives (ADR-0038 decision 2): a folder of its own under the data folder, with its own
//! sealed key, as the learned words and the clipboard history keep theirs.

use kx_platform::AppDirs;
use std::path::PathBuf;

pub use kx_platform::VAULT_DIR;

/// The notebook's folder for the app's `dirs`.
#[must_use]
pub fn folder(dirs: &AppDirs) -> PathBuf {
    dirs.data_dir().path().join(VAULT_DIR)
}
