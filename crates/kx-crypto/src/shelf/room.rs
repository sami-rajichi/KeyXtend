//! The shelf's drive: its free space and name come from the platform, so this crate stays OS-neutral.

use std::io;
use std::path::Path;

/// Reads the free space and the name of a folder's drive; the platform implements it.
pub trait DiskRoom: Send + Sync {
    /// Bytes free on the drive that holds `path`, an existing folder.
    ///
    /// # Errors
    /// When the drive cannot be read.
    fn free(&self, path: &Path) -> io::Result<u64>;

    /// The name to show for the drive that holds `path`, such as `D:`.
    ///
    /// # Errors
    /// When the drive cannot be read.
    fn drive(&self, path: &Path) -> io::Result<String>;
}

/// The nearest folder at or above `path` that exists, for asking about its drive; `path` when none does.
pub(super) fn nearest(path: &Path) -> &Path {
    path.ancestors().find(|p| p.is_dir()).unwrap_or(path)
}
