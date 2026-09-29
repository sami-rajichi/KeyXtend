//! A temp folder that is deleted when dropped, even if the test fails.

use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};

/// Start of every folder name, so leftovers are easy to spot.
const PREFIX: &str = "kx";

/// How many names `new` tries before it gives up on folders that earlier runs left behind.
const MAX_TRIES: u32 = 8;

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
    /// When the folder cannot be created, or `MAX_TRIES` names are taken already.
    pub fn new(label: &str) -> io::Result<Self> {
        Self::make(label, &UNIQUE)
    }

    /// Creates the folder for the next value of `counter`; a name already taken by a leftover
    /// folder takes the next value, up to `MAX_TRIES` names.
    fn make(label: &str, counter: &AtomicU32) -> io::Result<Self> {
        let mut tries = 1;
        loop {
            let path = path_of(label, counter.fetch_add(1, Ordering::Relaxed));
            match std::fs::create_dir(&path) {
                Ok(()) => return Ok(Self(path)),
                Err(e) if e.kind() == io::ErrorKind::AlreadyExists && tries < MAX_TRIES => {
                    tries += 1;
                }
                Err(e) => return Err(e),
            }
        }
    }

    /// The folder's path.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.0
    }
}

/// The folder for `label` and counter value `n`.
fn path_of(label: &str, n: u32) -> PathBuf {
    let name = format!("{PREFIX}-{label}-{}-{n}", std::process::id());
    std::env::temp_dir().join(name)
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

    /// A folder an earlier crashed run could have left, removed when the test ends.
    fn leftover(label: &str, n: u32) -> TempDir {
        let path = path_of(label, n);
        std::fs::create_dir(&path).unwrap();
        TempDir(path)
    }

    #[test]
    fn a_leftover_folder_is_skipped_for_the_next_name() {
        let counter = AtomicU32::new(0);
        let left: Vec<_> = (0..MAX_TRIES - 1).map(|n| leftover("skip", n)).collect();
        let dir = TempDir::make("skip", &counter).unwrap();
        assert!(left.iter().all(|old| old.path() != dir.path()));
        assert_eq!(counter.load(Ordering::Relaxed), MAX_TRIES);
    }

    #[test]
    fn too_many_leftover_folders_give_already_exists() {
        let counter = AtomicU32::new(0);
        let _left: Vec<_> = (0..MAX_TRIES).map(|n| leftover("many", n)).collect();
        let err = TempDir::make("many", &counter).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::AlreadyExists);
        assert_eq!(counter.load(Ordering::Relaxed), MAX_TRIES);
    }

    #[test]
    fn any_other_error_returns_at_once() {
        let counter = AtomicU32::new(0);
        let err = TempDir::make("no/such/folder", &counter).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::NotFound);
        assert_eq!(counter.load(Ordering::Relaxed), 1, "no second try");
    }

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
