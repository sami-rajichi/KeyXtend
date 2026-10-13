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
    /// The file is a folder, unreadable, too big, not UTF-8 or not TOML.
    Damaged,
}

/// The Windows error when another program opened the file without sharing it.
#[cfg(windows)]
const ERROR_SHARING_VIOLATION: i32 = 32;
/// The Windows error when another program locked a region of the file.
#[cfg(windows)]
const ERROR_LOCK_VIOLATION: i32 = 33;

/// The OS error codes that mean another program holds the file; Windows only.
#[cfg(windows)]
const LOCK_CODES: &[i32] = &[ERROR_SHARING_VIOLATION, ERROR_LOCK_VIOLATION];
#[cfg(not(windows))]
const LOCK_CODES: &[i32] = &[];

/// Sorts an I/O error: missing, a lock to wait out, or else damage as before.
/// Linux reports "not a directory" where Windows reports "path not found".
fn classify(error: &io::Error) -> Problem {
    if error
        .raw_os_error()
        .is_some_and(|code| LOCK_CODES.contains(&code))
    {
        return Problem::Locked;
    }
    match error.kind() {
        io::ErrorKind::NotFound | io::ErrorKind::NotADirectory => Problem::Missing,
        io::ErrorKind::PermissionDenied | io::ErrorKind::ResourceBusy => Problem::Locked,
        _ => Problem::Damaged,
    }
}

/// Sorts an error on `path`; Windows refuses to open a folder as a file, so a folder is checked first.
fn problem(path: &Path, error: &io::Error) -> Problem {
    if path.is_dir() {
        Problem::Damaged
    } else {
        classify(error)
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
    use io::{Error, ErrorKind};
    use kx_test_support::tempdir::TempDir;
    use std::fs;

    fn of(kind: ErrorKind) -> Problem {
        classify(&Error::from(kind))
    }

    /// Linux says "not a directory" where Windows says "path not found", as when a file sits in
    /// place of the data folder.
    #[test]
    fn not_found_and_not_a_folder_are_missing() {
        for kind in [ErrorKind::NotFound, ErrorKind::NotADirectory] {
            assert_eq!(of(kind), Problem::Missing, "{kind:?}");
        }
    }

    #[test]
    fn a_refusal_or_a_busy_file_is_a_lock_to_wait_out() {
        for kind in [ErrorKind::PermissionDenied, ErrorKind::ResourceBusy] {
            assert_eq!(of(kind), Problem::Locked, "{kind:?}");
        }
    }

    #[cfg(windows)]
    #[test]
    fn a_sharing_or_lock_violation_is_a_lock() {
        for code in [ERROR_SHARING_VIOLATION, ERROR_LOCK_VIOLATION] {
            let error = Error::from_raw_os_error(code);
            assert_eq!(classify(&error), Problem::Locked, "{code}");
        }
    }

    /// Error 32 is a broken pipe outside Windows, which is no lock.
    #[cfg(not(windows))]
    #[test]
    fn the_windows_lock_codes_mean_no_lock_elsewhere() {
        const BROKEN_PIPE: i32 = 32;
        let error = Error::from_raw_os_error(BROKEN_PIPE);
        assert_eq!(classify(&error), Problem::Damaged);
    }

    #[test]
    fn every_other_error_keeps_meaning_damaged() {
        let kinds = [
            ErrorKind::IsADirectory,
            ErrorKind::Interrupted,
            ErrorKind::TimedOut,
            ErrorKind::WouldBlock,
            ErrorKind::InvalidData,
            ErrorKind::UnexpectedEof,
            ErrorKind::Other,
        ];
        for kind in kinds {
            assert_eq!(of(kind), Problem::Damaged, "{kind:?}");
        }
    }

    #[test]
    fn a_folder_in_place_of_the_file_is_damaged() {
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
        let dir = TempDir::new("readlock").unwrap();
        let path = dir.path().join("held.toml");
        fs::write(&path, "[a]\nsize = 3\n").unwrap();
        let held = kx_test_support::lock::hold(&path).unwrap();
        let got = read_table(&path);
        drop(held);
        assert_eq!(got, Err(Problem::Locked));
    }
}
