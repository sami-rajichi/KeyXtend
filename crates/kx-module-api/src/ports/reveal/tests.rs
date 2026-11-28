//! A folder to show, in both path styles (ADR-0062): only a plain absolute path with no part Windows would trim, a
//! `Debug` that hides it, errors that never name it, and the service's gate.

use super::*;
use crate::REDACTED_MARKER;
use PathRoot::{Drive, Slash};
use std::collections::HashSet;

/// A folder holding the user's name, as a models folder may.
const MODELS: &str = r"C:\Users\Alex\KeyXtend\models";
/// The same folder off Windows.
const UNIX_MODELS: &str = "/home/Alex/KeyXtend/models";

/// Checks that `root`'s style takes each of `good` as it is.
fn taken(root: PathRoot, good: &[&str]) {
    for path in good {
        let got = FolderPath::new_in(root, PathBuf::from(path)).map(|f| f.expose().to_path_buf());
        assert_eq!(got, Ok(PathBuf::from(path)), "{root:?}");
    }
}

/// Checks that `root`'s style refuses each of `bad`.
fn refused(root: PathRoot, bad: &[&str]) {
    for path in bad {
        let got = FolderPath::new_in(root, PathBuf::from(path));
        assert_eq!(got, Err(FolderError::BadPath), "{root:?} {path}");
    }
}

#[test]
fn a_plain_absolute_path_of_each_style_is_taken_as_it_is() {
    taken(Drive, &[MODELS, r"D:\KeyXtend\models", "E:/models", r"D:\"]);
    taken(Slash, &[UNIX_MODELS, "/media/usb/models", "/"]);
    let ours = if PathRoot::NATIVE == Drive {
        MODELS
    } else {
        UNIX_MODELS
    };
    assert!(
        FolderPath::new(PathBuf::from(ours)).is_ok(),
        "the native style"
    );
}

#[test]
fn a_share_a_device_a_stream_a_dot_part_or_a_relative_path_is_refused() {
    let drive = [
        r"\\server\share\models",
        r"\\?\C:\models",
        r"\\.\C:\models",
        r"C:\models:hidden",
        r"C:\KeyXtend\..\Windows",
        r"C:\KeyXtend\.\models",
        r"C:\NUL\models",
        "models",
        r"C:models",
        UNIX_MODELS,
        "",
    ];
    refused(Drive, &drive);
    let unix = [
        "//server/share/models",
        "/models:hidden",
        "/KeyXtend/../etc",
        "/KeyXtend/./models",
        "/NUL/models",
        r"/home\models",
        "models",
        MODELS,
        "",
    ];
    refused(Slash, &unix);
}

#[test]
fn a_part_ending_in_a_dot_or_a_space_is_refused_as_windows_would_drop_it() {
    let drive = [
        r"C:\KeyXtend.\models",
        r"C:\KeyXtend\models.",
        r"C:\KeyXtend \models",
        "C:/KeyXtend/models ",
    ];
    refused(Drive, &drive);
    refused(
        Slash,
        &[
            "/KeyXtend./models",
            "/KeyXtend/models ",
            "/KeyXtend /models",
        ],
    );
}

#[test]
fn debug_prints_only_the_marker() {
    let folder = FolderPath::new_in(Drive, PathBuf::from(MODELS)).unwrap();
    let text = format!("{folder:?}");
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
