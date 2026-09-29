//! Holds a file open the way another program's lock does, so tests can meet a locked file.

use std::fs::{File, OpenOptions};
use std::io;
use std::os::windows::fs::OpenOptionsExt;
use std::path::Path;

/// The Windows share mode that lets no other open succeed.
const NO_SHARING: u32 = 0;

/// Opens `path` with no sharing, so no other open succeeds until the returned file drops.
///
/// # Errors
/// The open error, for example when the file is missing.
pub fn hold(path: &Path) -> io::Result<File> {
    OpenOptions::new()
        .read(true)
        .share_mode(NO_SHARING)
        .open(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tempdir::TempDir;
    use std::fs;

    #[test]
    fn a_held_file_opens_again_only_after_it_drops() {
        let dir = TempDir::new("hold").unwrap();
        let path = dir.path().join("held.txt");
        fs::write(&path, "x").unwrap();
        let held = hold(&path).unwrap();
        assert!(File::open(&path).is_err());
        drop(held);
        assert!(File::open(&path).is_ok());
    }

    #[test]
    fn a_missing_file_is_an_error() {
        let dir = TempDir::new("holdnone").unwrap();
        assert!(hold(&dir.path().join("none.txt")).is_err());
    }
}
