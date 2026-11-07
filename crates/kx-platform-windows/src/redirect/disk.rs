//! The real disk behind the probe: plain file-system calls as this program sees them, a package's redirection
//! included, that never follow a link where a list or a delete could leave the copy.

use std::fs::{self, OpenOptions};
use std::io::{self, ErrorKind};
use std::path::Path;

use kx_platform::mirror::MirrorDisk;

/// The file system of the running program.
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct WinMirrorDisk;

impl MirrorDisk for WinMirrorDisk {
    fn folders(&self, dir: &Path) -> io::Result<Vec<String>> {
        // An entry's type comes from the listing itself, so a link or junction is never taken for a folder.
        let entries = fs::read_dir(dir)?.filter_map(Result::ok);
        let dirs = entries.filter(|e| e.file_type().is_ok_and(|t| t.is_dir()));
        Ok(dirs
            .filter_map(|e| e.file_name().into_string().ok())
            .collect())
    }

    fn is_dir(&self, path: &Path) -> bool {
        fs::symlink_metadata(path).is_ok_and(|m| m.is_dir())
    }

    fn exists(&self, path: &Path) -> bool {
        fs::symlink_metadata(path).is_ok()
    }

    fn is_empty(&self, dir: &Path) -> bool {
        fs::read_dir(dir).is_ok_and(|mut entries| entries.next().is_none())
    }

    fn make_dir(&self, dir: &Path) -> io::Result<bool> {
        match fs::create_dir(dir) {
            Ok(()) => Ok(true),
            Err(e) if e.kind() == ErrorKind::AlreadyExists && dir.is_dir() => Ok(false),
            Err(e) => Err(e),
        }
    }

    fn write_new(&self, path: &Path) -> io::Result<()> {
        let mut new = OpenOptions::new();
        new.write(true).create_new(true).open(path).map(drop)
    }

    fn remove_file(&self, path: &Path) -> io::Result<()> {
        fs::remove_file(path)
    }

    fn remove_empty(&self, dir: &Path) -> io::Result<()> {
        fs::remove_dir(dir)
    }

    fn remove_tree(&self, dir: &Path) -> io::Result<()> {
        // The standard delete removes a link or junction it meets as a link, and never follows it.
        fs::remove_dir_all(dir)
    }
}
