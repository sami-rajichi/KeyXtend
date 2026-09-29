//! Reads one settings file as a table, or says why it gave none: missing, locked or damaged.

use crate::names::MAX_FILE_BYTES;
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

/// Why a file gave no table.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(super) enum Problem {
    /// The file does not exist.
    Missing,
    /// Another program holds the file or it refused access, so try again next run.
    Locked,
    /// The file is a folder, too big, not UTF-8 or not TOML.
    Damaged,
}

/// Sorts an I/O error: not found is missing, a folder is damaged, anything else is a lock.
fn classify(kind: io::ErrorKind) -> Problem {
    match kind {
        io::ErrorKind::NotFound => Problem::Missing,
        io::ErrorKind::IsADirectory => Problem::Damaged,
        _ => Problem::Locked,
    }
}

/// Sorts an error on `path`; Windows refuses to open a folder as a file, so a folder is checked first.
fn problem(path: &Path, error: &io::Error) -> Problem {
    if path.is_dir() {
        Problem::Damaged
    } else {
        classify(error.kind())
    }
}

/// Reads one file as a table, never reading more than `MAX_FILE_BYTES` and one byte.
pub(super) fn read_table(path: &Path) -> Result<toml::Table, Problem> {
    let file = File::open(path).map_err(|e| problem(path, &e))?;
    let mut bytes = Vec::new();
    file.take(MAX_FILE_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| problem(path, &e))?;
    if u64::try_from(bytes.len()).map_or(true, |len| len > MAX_FILE_BYTES) {
        return Err(Problem::Damaged);
    }
    let text = String::from_utf8(bytes).map_err(|_| Problem::Damaged)?;
    text.parse().map_err(|_| Problem::Damaged)
}

#[cfg(test)]
mod tests {
    use super::*;
    use io::ErrorKind;
    use kx_test_support::tempdir::TempDir;
    use std::fs;

    #[test]
    fn only_not_found_is_missing() {
        assert_eq!(classify(ErrorKind::NotFound), Problem::Missing);
    }

    #[test]
    fn a_refusal_or_a_busy_file_is_a_lock_to_wait_out() {
        let kinds = [
            ErrorKind::PermissionDenied,
            ErrorKind::ResourceBusy,
            ErrorKind::WouldBlock,
            ErrorKind::Interrupted,
            ErrorKind::TimedOut,
            ErrorKind::Other,
        ];
        for kind in kinds {
            assert_eq!(classify(kind), Problem::Locked, "{kind:?}");
        }
    }

    #[test]
    fn a_folder_in_place_of_the_file_is_damaged() {
        assert_eq!(classify(ErrorKind::IsADirectory), Problem::Damaged);
        let dir = TempDir::new("readdir").unwrap();
        assert_eq!(read_table(dir.path()), Err(Problem::Damaged));
    }

    #[test]
    fn bad_text_is_damaged() {
        let dir = TempDir::new("readbad").unwrap();
        let path = dir.path().join("bad.toml");
        fs::write(&path, "[a\n").unwrap();
        assert_eq!(read_table(&path), Err(Problem::Damaged));
    }

    /// Another program holding the file with no sharing makes it locked, not damaged.
    #[cfg(windows)]
    #[test]
    fn a_file_held_with_no_sharing_is_locked() {
        use std::os::windows::fs::OpenOptionsExt;
        let dir = TempDir::new("readlock").unwrap();
        let path = dir.path().join("held.toml");
        fs::write(&path, "[a]\nsize = 3\n").unwrap();
        let held = fs::OpenOptions::new()
            .read(true)
            .share_mode(0)
            .open(&path)
            .unwrap();
        let got = read_table(&path);
        drop(held);
        assert_eq!(got, Err(Problem::Locked));
    }
}
