//! The file steps: read with a cap, and write a whole file through a temp file beside it.

use crate::consts::TEMP_EXT;
use std::fs::{self, File};
use std::io::{self, Read as _, Write as _};
use std::path::{Path, PathBuf};

/// What reading a file gave.
#[derive(Debug, PartialEq, Eq)]
pub(super) enum Read {
    /// There is no such file.
    Missing,
    /// The file is longer than the cap; only the cap and one byte were read.
    TooBig,
    /// The whole file.
    Bytes(Vec<u8>),
}

/// Reads `path`, never more than `cap` bytes and one.
pub(super) fn read(path: &Path, cap: usize) -> io::Result<Read> {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Read::Missing),
        Err(e) => return Err(e),
    };
    let limit = u64::try_from(cap).map_or(u64::MAX, |cap| cap.saturating_add(1));
    let mut bytes = Vec::new();
    file.take(limit).read_to_end(&mut bytes)?;
    Ok(if bytes.len() > cap {
        Read::TooBig
    } else {
        Read::Bytes(bytes)
    })
}

/// The temp file a write to `path` goes through: its name plus the temp extension.
fn temp_of(path: &Path) -> PathBuf {
    let mut name = path.as_os_str().to_owned();
    name.push(".");
    name.push(TEMP_EXT);
    PathBuf::from(name)
}

/// Writes `bytes` to the temp file and syncs it to disk.
fn write_temp(temp: &Path, bytes: &[u8]) -> io::Result<()> {
    let mut file = File::create(temp)?;
    file.write_all(bytes)?;
    file.sync_all()
}

/// Puts `bytes` in `path` whole: a temp file in the same folder, then one rename over the old file.
pub(super) fn write_whole(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let temp = temp_of(path);
    let result = write_temp(&temp, bytes).and_then(|()| fs::rename(&temp, path));
    if result.is_err() {
        // A leftover temp file is harmless and the next write replaces it, so a failed delete is ignored.
        let _ = fs::remove_file(&temp);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use kx_test_support::tempdir::TempDir;

    #[test]
    fn a_file_at_the_cap_is_read_and_one_byte_more_is_too_big() {
        let dir = TempDir::new("crypto-disk").unwrap();
        let path = dir.path().join("blob");
        fs::write(&path, b"four").unwrap();
        assert_eq!(read(&path, 4).unwrap(), Read::Bytes(b"four".to_vec()));
        assert_eq!(read(&path, 3).unwrap(), Read::TooBig);
        assert_eq!(read(&dir.path().join("none"), 4).unwrap(), Read::Missing);
    }

    #[test]
    fn a_write_replaces_the_file_and_leaves_no_temp() {
        let dir = TempDir::new("crypto-disk").unwrap();
        let path = dir.path().join("blob");
        write_whole(&path, b"old").unwrap();
        write_whole(&path, b"new").unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"new");
        assert!(!temp_of(&path).exists());
    }

    #[test]
    fn a_failed_rename_keeps_the_old_file_and_drops_the_temp() {
        let dir = TempDir::new("crypto-disk").unwrap();
        let path = dir.path().join("blob");
        fs::create_dir_all(path.join("inside")).unwrap();
        assert!(write_whole(&path, b"new").is_err());
        assert!(path.join("inside").is_dir());
        assert!(!temp_of(&path).exists());
    }
}
