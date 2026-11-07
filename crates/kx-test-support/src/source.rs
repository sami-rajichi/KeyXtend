//! Walks a crate's source, for tests that pin what its code may name.

use std::io;
use std::path::{Path, PathBuf};

/// Every Rust file under `dir`, at any depth.
///
/// # Errors
///
/// When a folder cannot be read.
pub fn rust_files(dir: &Path) -> io::Result<Vec<PathBuf>> {
    let mut found = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            found.extend(rust_files(&path)?);
        } else if path.extension().is_some_and(|e| e == "rs") {
            found.push(path);
        }
    }
    Ok(found)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tempdir::TempDir;

    #[test]
    fn it_finds_rust_files_at_any_depth_and_nothing_else() {
        let dir = TempDir::new("source").unwrap();
        let deep = dir.path().join("a").join("b");
        std::fs::create_dir_all(&deep).unwrap();
        std::fs::write(dir.path().join("top.rs"), "").unwrap();
        std::fs::write(deep.join("low.rs"), "").unwrap();
        std::fs::write(deep.join("note.md"), "").unwrap();
        let mut found = rust_files(dir.path()).unwrap();
        found.sort();
        assert_eq!(found, [deep.join("low.rs"), dir.path().join("top.rs")]);
    }

    #[test]
    fn a_folder_that_cannot_be_read_is_an_error() {
        let dir = TempDir::new("source-gone").unwrap();
        assert!(rust_files(&dir.path().join("gone")).is_err());
    }
}
