//! Where snips are written: errors that never name a path, and an unsaved file that prints its kind and ages only.

use super::{holds, keeps};
use crate::REDACTED_MARKER;
use crate::ports::snip::{FileKind, SaveError, SnipFile, UnsavedFile};
use kx_test_support::paths::native_path;
use std::collections::HashSet;
use std::time::Duration;

#[test]
fn save_errors_each_say_something_else_and_name_no_path() {
    let errors = [
        SaveError::BadPath,
        SaveError::NoFolder,
        SaveError::Missing,
        SaveError::ReadOnly,
        SaveError::Full,
        SaveError::Io,
    ];
    let texts: HashSet<_> = errors.iter().map(ToString::to_string).collect();
    assert_eq!(texts.len(), errors.len(), "{texts:?}");
    assert!(texts.iter().all(|t| !t.contains('\\')), "{texts:?}");
}

#[test]
fn an_unsaved_file_prints_its_kind_and_age_and_never_its_path() {
    let path = native_path(r"C:\Users\Alex\snips\unsaved\a.mp4");
    let aged = UnsavedFile {
        file: SnipFile::new(path, FileKind::Mp4).unwrap(),
        age: Duration::from_hours(25),
        copied: Some(Duration::from_hours(2)),
    };
    let shown = format!("{aged:?}");
    assert!(keeps(&shown), "{shown}");
    holds(&shown, &["Mp4", "90000s", "7200s", REDACTED_MARKER]);
}
