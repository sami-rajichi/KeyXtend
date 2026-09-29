//! A temp folder that is deleted when dropped, even if the test fails.

use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};

/// Start of every folder name, so leftovers are easy to spot.
const PREFIX: &str = "kx";

/// Makes names unique across the tests of one process.
static UNIQUE: AtomicU32 = AtomicU32::new(0);

/// A fresh folder under the OS temp dir.
#[derive(Debug)]
pub struct TempDir(PathBuf);

impl TempDir {
    /// Creates an empty folder named after `label`, the process id and a counter.
    ///
    /// # Errors
    ///
    /// When the folder cannot be created.
    pub fn new(label: &str) -> io::Result<Self> {
        let n = UNIQUE.fetch_add(1, Ordering::Relaxed);
        let name = format!("{PREFIX}-{label}-{}-{n}", std::process::id());
        let path = std::env::temp_dir().join(name);
        std::fs::create_dir(&path)?;
        Ok(Self(path))
    }

    /// The folder's path.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        // A leftover temp folder is harmless, so a failed delete is ignored.
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_folders_with_one_label_differ() {
        let (a, b) = (TempDir::new("same").unwrap(), TempDir::new("same").unwrap());
        assert_ne!(a.path(), b.path());
    }

    #[test]
    fn the_folder_exists_while_held_and_is_gone_after_drop() {
        let dir = TempDir::new("held").unwrap();
        let path = dir.path().to_path_buf();
        assert!(path.is_dir());
        drop(dir);
        assert!(!path.exists());
    }

    #[test]
    fn the_label_is_part_of_the_name() {
        let dir = TempDir::new("marker").unwrap();
        let name = dir
            .path()
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        assert!(name.contains("marker"), "{name}");
    }

    #[test]
    fn the_folder_starts_empty() {
        let dir = TempDir::new("empty").unwrap();
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 0);
    }

    #[test]
    fn a_folder_with_files_in_it_is_removed_whole() {
        let dir = TempDir::new("full").unwrap();
        let path = dir.path().to_path_buf();
        std::fs::create_dir_all(path.join("a").join("b")).unwrap();
        std::fs::write(path.join("a").join("b").join("f.txt"), "x").unwrap();
        drop(dir);
        assert!(!path.exists());
    }
}
