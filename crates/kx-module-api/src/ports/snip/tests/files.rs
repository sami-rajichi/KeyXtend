//! A snip's file in both path styles (ADR-0062): only a plain absolute path of its own kind, a `Debug` that hides it,
//! and dialog and play errors that never name it.

use super::keeps;
use crate::REDACTED_MARKER;
use crate::ports::snip::{AskError, FileKind, PlayError, SnipFile};
use crate::ports::{FolderPath, PathRoot};
use PathRoot::{Drive, Slash};
use std::collections::HashSet;
use std::path::{MAIN_SEPARATOR, PathBuf};

/// A file in a folder holding the user's name, as the Snips folder does.
const SNIP: &str = r"C:\Users\Alex\AppData\Local\KeyXtend\snips\Snip 2026-11-23 14-30-05";
/// The same file off Windows.
const UNIX_SNIP: &str = "/home/Alex/.local/share/KeyXtend/snips/Snip 2026-11-23 14-30-05";

/// `snip` with `ext`.
fn named(snip: &str, ext: &str) -> PathBuf {
    PathBuf::from(format!("{snip}.{ext}"))
}

/// `parts` on `drive`, spelt with `\`, with `/`, and joined with this system's own separator.
fn spellings(drive: &str, parts: &[&str]) -> [PathBuf; 3] {
    let spelt = |sep: char| PathBuf::from(format!("{drive}:{sep}{}", parts.join(&sep.to_string())));
    let root = PathBuf::from(format!("{drive}:{MAIN_SEPARATOR}"));
    let joined = parts.iter().fold(root, |path, part| path.join(part));
    [spelt('\\'), spelt('/'), joined]
}

/// The video at `path` in `root`'s style.
fn video(root: PathRoot, path: impl Into<PathBuf>) -> SnipFile {
    SnipFile::new_in(root, path.into(), FileKind::Mp4).unwrap()
}

/// The folder at `path` in `root`'s style.
fn folder(root: PathRoot, path: impl Into<PathBuf>) -> FolderPath {
    FolderPath::new_in(root, path.into()).unwrap()
}

#[test]
fn each_kind_names_its_extension_as_the_frames_do() {
    assert_eq!(FileKind::ALL, [FileKind::Png, FileKind::Mp4]);
    assert_eq!(FileKind::Png.ext(), kx_ipc::PNG_EXT);
    assert_eq!(FileKind::Mp4.ext(), kx_ipc::MP4_EXT);
    assert_eq!((kx_ipc::PNG_EXT, kx_ipc::MP4_EXT), ("png", "mp4"));
}

#[test]
fn a_plain_absolute_path_of_its_kind_is_taken_in_any_case() {
    for (root, snip) in [(Drive, SNIP), (Slash, UNIX_SNIP)] {
        for (ext, kind) in [("png", FileKind::Png), ("MP4", FileKind::Mp4)] {
            let file = SnipFile::new_in(root, named(snip, ext), kind);
            let facts = file.as_ref().map(|f| (f.expose().to_path_buf(), f.kind()));
            assert_eq!(facts, Some((named(snip, ext), kind)), "{root:?}");
        }
    }
    let ours = if PathRoot::NATIVE == Drive {
        SNIP
    } else {
        UNIX_SNIP
    };
    assert!(SnipFile::new(named(ours, "png"), FileKind::Png).is_some());
}

#[test]
fn another_kind_a_bare_extension_or_no_plain_drive_path_is_refused() {
    let bad = [
        (named(SNIP, "png"), FileKind::Mp4),
        (named(SNIP, "png.exe"), FileKind::Png),
        (PathBuf::from(SNIP), FileKind::Png),
        (PathBuf::from(r"C:\Snips\NUL.png"), FileKind::Png),
        (PathBuf::from(r"C:\Snips.\a.png"), FileKind::Png),
        (PathBuf::from(r"\\host\share\a.png"), FileKind::Png),
        (PathBuf::from(r"C:\Snips\a.png:hidden"), FileKind::Png),
        (PathBuf::from("a.png"), FileKind::Png),
        (named(UNIX_SNIP, "png"), FileKind::Png),
    ];
    let bare = spellings("C", &["Snips", ".png"]).map(|p| (p, FileKind::Png));
    for (path, kind) in bad.into_iter().chain(bare) {
        assert!(
            SnipFile::new_in(Drive, path.clone(), kind).is_none(),
            "{path:?}"
        );
    }
}

#[test]
fn another_kind_a_bare_extension_or_no_plain_unix_path_is_refused() {
    let bad = [
        (named(UNIX_SNIP, "png"), FileKind::Mp4),
        (PathBuf::from("/Snips/NUL.png"), FileKind::Png),
        (PathBuf::from("/Snips./a.png"), FileKind::Png),
        (PathBuf::from("//host/share/a.png"), FileKind::Png),
        (PathBuf::from("/Snips/a.png:hidden"), FileKind::Png),
        (PathBuf::from(r"/Snips\a.png"), FileKind::Png),
        (PathBuf::from("/Snips/.png"), FileKind::Png),
        (PathBuf::from("a.png"), FileKind::Png),
        (named(SNIP, "png"), FileKind::Png),
    ];
    for (path, kind) in bad {
        assert!(
            SnipFile::new_in(Slash, path.clone(), kind).is_none(),
            "{path:?}"
        );
    }
}

#[test]
fn a_file_lies_only_directly_in_its_folder_however_either_is_spelt() {
    let elsewhere = [
        spellings("C", &["Snips", "a.mp4"]),
        spellings("C", &["Snips", "unsaved", "old", "a.mp4"]),
        spellings("C", &["Snips", "unsaved2", "a.mp4"]),
        spellings("D", &["Snips", "unsaved", "a.mp4"]),
    ];
    for at in spellings("C", &["Snips", "unsaved"]) {
        let unsaved = folder(Drive, at);
        for path in spellings("C", &["Snips", "unsaved", "a.mp4"]) {
            assert!(
                video(Drive, path.clone()).lies_in_at(Drive, &unsaved),
                "{path:?}"
            );
        }
        for path in elsewhere.iter().flatten() {
            assert!(
                !video(Drive, path.clone()).lies_in_at(Drive, &unsaved),
                "{path:?}"
            );
        }
    }
}

#[test]
fn a_unix_file_lies_only_directly_in_its_folder_with_every_letters_case() {
    let elsewhere = [
        "/Snips/a.mp4",
        "/Snips/unsaved/old/a.mp4",
        "/Snips/unsaved2/a.mp4",
        "/snips/unsaved/a.mp4",
        "/media/Snips/unsaved/a.mp4",
    ];
    for at in ["/Snips/unsaved", "/Snips/unsaved/"] {
        let unsaved = folder(Slash, at);
        let inside = video(Slash, "/Snips/unsaved/a.mp4");
        assert!(inside.lies_in_at(Slash, &unsaved), "{at}");
        for path in elsewhere {
            assert!(!video(Slash, path).lies_in_at(Slash, &unsaved), "{path}");
        }
    }
}

#[test]
fn either_separator_splits_a_drive_path_as_windows_reads_it() {
    let unsaved = folder(Drive, r"C:/Snips\unsaved");
    let inside = |path: &str, around: &FolderPath| video(Drive, path).lies_in_at(Drive, around);
    assert!(inside(r"C:\Snips/unsaved\a.mp4", &unsaved));
    assert!(
        inside(r"c:\Snips\unsaved\a.mp4", &unsaved),
        "a drive in either case"
    );
    let slashed = folder(Drive, r"C:\Snips\unsaved\");
    assert!(inside(r"C:\Snips\unsaved\a.mp4", &slashed));
    let below = r"C:/Snips/unsaved/old\a.mp4";
    assert!(!inside(below, &unsaved), "a part below, past a backslash");
    let bare = SnipFile::new_in(Drive, PathBuf::from(r"C:/Snips/a.png\.png"), FileKind::Png);
    assert!(bare.is_none(), "a bare extension past a backslash");
}

#[test]
fn a_file_prints_its_kind_and_never_its_path() {
    let shown = format!("{:?}", video(Drive, named(SNIP, "mp4")));
    assert!(keeps(&shown), "{shown}");
    assert_eq!(
        shown,
        format!("SnipFile {{ path: {REDACTED_MARKER}, kind: Mp4 }}")
    );
}

#[test]
fn dialog_and_play_errors_say_something_else_and_name_no_path() {
    let asks = [
        AskError::BadPath,
        AskError::Failed,
        AskError::Gone,
        AskError::Range("job_mb"),
    ];
    let texts: Vec<String> = asks
        .iter()
        .map(ToString::to_string)
        .chain(
            [
                PlayError::BadPath,
                PlayError::Missing,
                PlayError::Refused(5),
            ]
            .iter()
            .map(ToString::to_string),
        )
        .collect();
    let unique: HashSet<_> = texts.iter().collect();
    assert_eq!(unique.len(), texts.len(), "{texts:?}");
    assert!(texts.iter().all(|t| !t.contains('\\')), "{texts:?}");
    assert!(PlayError::Refused(5).to_string().contains("code 5"));
}
