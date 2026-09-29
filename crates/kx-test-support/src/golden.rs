//! Golden files: compare text with a file, or rewrite the file when blessing.

use std::io;
use std::path::{Path, PathBuf};

/// Name of the env var that turns blessing on.
pub const BLESS_VAR: &str = "KX_BLESS";
/// The value of the env var that turns blessing on.
pub const BLESS_ON: &str = "1";
/// Stands in for a line that the shorter text does not have.
const NO_LINE: &str = "<no line>";

/// Why a golden check failed.
#[derive(Debug, thiserror::Error)]
pub enum GoldenError {
    /// The golden file does not exist yet.
    #[error("golden file {} is missing; run the test with {}={} to create it", .0.display(), BLESS_VAR, BLESS_ON)]
    Missing(PathBuf),
    /// The text differs from the golden file.
    #[error("{} differs at line {line}: expected {expected:?}, got {actual:?}", path.display())]
    Differs {
        /// The golden file.
        path: PathBuf,
        /// The first differing line, counted from 1.
        line: usize,
        /// That line in the golden file.
        expected: String,
        /// That line in the text.
        actual: String,
    },
    /// The golden file could not be read or written.
    #[error("{}: {source}", path.display())]
    Io {
        /// The golden file.
        path: PathBuf,
        /// What the system said.
        source: io::Error,
    },
}

/// Compares `actual` with the golden file `path`; when `bless` is set, writes it there instead.
///
/// Line endings are compared as `\n`, since git may check the file out with `\r\n`.
///
/// # Errors
///
/// When the file is missing or differs, or cannot be read or written.
pub fn check(path: &Path, actual: &str, bless: bool) -> Result<(), GoldenError> {
    if bless {
        return write(path, actual);
    }
    let expected = std::fs::read_to_string(path).map_err(|source| match source.kind() {
        io::ErrorKind::NotFound => GoldenError::Missing(path.to_path_buf()),
        _ => GoldenError::Io {
            path: path.to_path_buf(),
            source,
        },
    })?;
    diff(path, &expected, actual)
}

/// Checks `actual` against the golden file `path`, or rewrites the file when blessing is on.
///
/// # Panics
///
/// When the check fails, with the reason as the message.
pub fn assert_golden(path: impl AsRef<Path>, actual: &str) {
    if let Err(e) = check(path.as_ref(), actual, blessing()) {
        panic!("{e}");
    }
}

/// True when the env var asks to rewrite golden files.
fn blessing() -> bool {
    std::env::var(BLESS_VAR).is_ok_and(|v| v == BLESS_ON)
}

/// Writes `text` to `path`, making its folders.
fn write(path: &Path, text: &str) -> Result<(), GoldenError> {
    let fail = |source| GoldenError::Io {
        path: path.to_path_buf(),
        source,
    };
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(fail)?;
    }
    std::fs::write(path, text).map_err(fail)
}

/// The first line where the texts differ, if any; line endings do not count.
fn diff(path: &Path, expected: &str, actual: &str) -> Result<(), GoldenError> {
    let (expected, actual) = (unix(expected), unix(actual));
    let (mut exp, mut act) = (expected.split('\n'), actual.split('\n'));
    let differing = (1..)
        .map_while(|line| match (exp.next(), act.next()) {
            (None, None) => None,
            (e, a) => Some((line, e, a)),
        })
        .find(|(_, e, a)| e != a);
    differing.map_or(Ok(()), |(line, e, a)| {
        Err(GoldenError::Differs {
            path: path.to_path_buf(),
            line,
            expected: e.unwrap_or(NO_LINE).to_string(),
            actual: a.unwrap_or(NO_LINE).to_string(),
        })
    })
}

/// The text with `\r\n` as `\n`.
fn unix(text: &str) -> String {
    text.replace("\r\n", "\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tempdir::TempDir;

    fn golden(text: &str) -> (TempDir, PathBuf) {
        let dir = TempDir::new("golden").unwrap();
        let file = dir.path().join("g.txt");
        std::fs::write(&file, text).unwrap();
        (dir, file)
    }

    #[test]
    fn matching_text_passes() {
        let (_dir, file) = golden("a\nb\n");
        assert!(check(&file, "a\nb\n", false).is_ok());
    }

    #[test]
    fn a_mismatch_names_the_first_differing_line() {
        let (_dir, file) = golden("a\nb\nc\n");
        let err = check(&file, "a\nX\nY\n", false).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("line 2"), "{msg}");
        assert!(msg.contains("\"b\"") && msg.contains("\"X\""), "{msg}");
    }

    #[test]
    fn a_shorter_text_differs_where_it_ends() {
        let (_dir, file) = golden("a\nb\nc\n");
        let msg = check(&file, "a\nb", false).unwrap_err().to_string();
        assert!(msg.contains("line 3"), "{msg}");
        assert!(msg.contains("\"c\"") && msg.contains(NO_LINE), "{msg}");
    }

    #[test]
    fn a_missing_file_says_how_to_bless() {
        let dir = TempDir::new("golden-missing").unwrap();
        let err = check(&dir.path().join("none.txt"), "a", false).unwrap_err();
        assert!(matches!(err, GoldenError::Missing(_)), "{err:?}");
        let hint = format!("{BLESS_VAR}={BLESS_ON}");
        assert!(err.to_string().contains(&hint), "{err}");
    }

    #[test]
    fn blessing_writes_then_a_normal_check_passes() {
        let dir = TempDir::new("golden-bless").unwrap();
        let file = dir.path().join("deep").join("er").join("g.txt");
        check(&file, "new\n", true).unwrap();
        assert_eq!(std::fs::read_to_string(&file).unwrap(), "new\n");
        assert!(check(&file, "new\n", false).is_ok());
    }

    #[test]
    fn blessing_replaces_a_different_file() {
        let (_dir, file) = golden("old\n");
        check(&file, "new\n", true).unwrap();
        assert!(check(&file, "new\n", false).is_ok());
    }

    #[test]
    fn crlf_and_lf_compare_equal_on_both_sides() {
        let (_dir, file) = golden("a\r\nb\r\n");
        assert!(check(&file, "a\nb\n", false).is_ok());
        assert!(check(&file, "a\r\nb\r\n", false).is_ok());
        let (_dir, lf) = golden("a\nb\n");
        assert!(check(&lf, "a\r\nb\r\n", false).is_ok());
    }

    #[test]
    fn a_folder_in_place_of_the_file_is_an_io_error() {
        let dir = TempDir::new("golden-io").unwrap();
        let err = check(dir.path(), "a", false).unwrap_err();
        assert!(matches!(err, GoldenError::Io { .. }), "{err:?}");
    }

    #[test]
    fn a_final_newline_counts() {
        let (_dir, file) = golden("a\n");
        let err = check(&file, "a", false).unwrap_err();
        assert!(
            matches!(err, GoldenError::Differs { line: 2, .. }),
            "{err:?}"
        );
    }

    #[test]
    fn the_assert_passes_on_a_match() {
        let (_dir, file) = golden("a\nb\n");
        assert_golden(&file, "a\nb\n");
    }

    #[test]
    fn the_assert_panics_with_the_reason() {
        if blessing() {
            return;
        }
        let (_dir, file) = golden("a\nb\n");
        let hit = std::panic::catch_unwind(|| assert_golden(&file, "a\nX\n")).unwrap_err();
        let msg = hit.downcast_ref::<String>().cloned().unwrap_or_default();
        assert!(msg.contains("line 2"), "{msg}");
    }
}
