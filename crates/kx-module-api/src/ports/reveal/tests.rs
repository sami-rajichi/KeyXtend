//! A folder to show: only a plain drive path with no part Windows would trim, a `Debug` that hides it, errors that
//! never name it, and the service's gate.

use super::*;
use crate::REDACTED_MARKER;
use std::collections::HashSet;

/// A folder holding the user's name, as a models folder may.
const MODELS: &str = r"C:\Users\Alex\KeyXtend\models";

#[test]
fn a_plain_drive_path_is_taken_as_it_is() {
    for good in [MODELS, r"D:\KeyXtend\models", "E:/models"] {
        let taken = FolderPath::new(PathBuf::from(good)).map(|f| f.expose().to_path_buf());
        assert_eq!(taken, Ok(PathBuf::from(good)));
    }
}

#[test]
fn a_share_a_device_a_stream_a_dot_part_or_a_relative_path_is_refused() {
    let bad = [
        r"\\server\share\models",
        r"\\?\C:\models",
        r"\\.\C:\models",
        r"C:\models:hidden",
        r"C:\KeyXtend\..\Windows",
        r"C:\KeyXtend\.\models",
        r"C:\NUL\models",
        "models",
        r"C:models",
        "",
    ];
    for path in bad {
        let refused = FolderPath::new(PathBuf::from(path));
        assert_eq!(refused, Err(FolderError::BadPath), "{path}");
    }
}

#[test]
fn a_part_ending_in_a_dot_or_a_space_is_refused_as_windows_would_drop_it() {
    for path in [
        r"C:\KeyXtend.\models",
        r"C:\KeyXtend\models.",
        r"C:\KeyXtend \models",
        "C:/KeyXtend/models ",
    ] {
        let refused = FolderPath::new(PathBuf::from(path));
        assert_eq!(refused, Err(FolderError::BadPath), "{path}");
    }
    let root = FolderPath::new(PathBuf::from(r"D:\")).map(|f| f.expose().to_path_buf());
    assert_eq!(root, Ok(PathBuf::from(r"D:\")), "a drive's root is plain");
}

#[test]
fn debug_prints_only_the_marker() {
    let text = format!("{:?}", FolderPath::new(PathBuf::from(MODELS)).unwrap());
    assert_eq!(text, format!("FolderPath({REDACTED_MARKER})"));
    assert!(!text.contains("Alex"), "{text}");
}

#[test]
fn errors_never_name_a_folder_and_each_says_something_else() {
    let errors = [
        FolderError::BadPath,
        FolderError::Missing,
        FolderError::Linked,
        FolderError::Refused(5),
    ];
    let texts: HashSet<_> = errors.iter().map(ToString::to_string).collect();
    assert_eq!(texts.len(), errors.len());
    assert!(
        texts.iter().all(|t| !t.contains('\\') && !t.contains('/')),
        "{texts:?}"
    );
    assert!(FolderError::Refused(5).to_string().contains("code 5"));
}

#[test]
fn the_folder_reveal_is_named_and_gated_by_open_folder() {
    assert_eq!(FolderRevealKey::ID.as_str(), "folder-reveal");
    let gate = Some(Capability::OpenFolder);
    assert_eq!(FolderRevealKey::CAPABILITY, gate, "ADR-0046");
}
