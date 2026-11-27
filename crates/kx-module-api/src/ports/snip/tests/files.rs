//! A snip's file: only a plain drive path of its own kind, read the same on every system, a `Debug` that hides it, and
//! dialog and play errors that never name it.

use super::keeps;
use crate::REDACTED_MARKER;
use crate::ports::FolderPath;
use crate::ports::snip::{AskError, FileKind, PlayError, SnipFile};
use std::collections::HashSet;
use std::path::{MAIN_SEPARATOR, PathBuf};

/// A file in a folder holding the user's name, as the Snips folder does.
const SNIP: &str = r"C:\Users\Alex\AppData\Local\KeyXtend\snips\Snip 2026-11-23 14-30-05";

/// `SNIP` with `ext`.
fn named(ext: &str) -> PathBuf {
    PathBuf::from(format!("{SNIP}.{ext}"))
}

/// `parts` on `drive`, spelt with `\`, with `/`, and joined with this system's own separator.
fn spellings(drive: &str, parts: &[&str]) -> [PathBuf; 3] {
    let spelt = |sep: char| PathBuf::from(format!("{drive}:{sep}{}", parts.join(&sep.to_string())));
    let root = PathBuf::from(format!("{drive}:{MAIN_SEPARATOR}"));
    let joined = parts.iter().fold(root, |path, part| path.join(part));
    [spelt('\\'), spelt('/'), joined]
}

/// The video at `path`.
fn video(path: PathBuf) -> SnipFile {
    SnipFile::new(path, FileKind::Mp4).unwrap()
}

#[test]
fn each_kind_names_its_extension_as_the_frames_do() {
    assert_eq!(FileKind::ALL, [FileKind::Png, FileKind::Mp4]);
    assert_eq!(FileKind::Png.ext(), kx_ipc::PNG_EXT);
    assert_eq!(FileKind::Mp4.ext(), kx_ipc::MP4_EXT);
    assert_eq!((kx_ipc::PNG_EXT, kx_ipc::MP4_EXT), ("png", "mp4"));
}

#[test]
fn a_plain_drive_path_of_its_kind_is_taken_in_any_case() {
    for (ext, kind) in [("png", FileKind::Png), ("MP4", FileKind::Mp4)] {
        let file = SnipFile::new(named(ext), kind);
        let facts = file.as_ref().map(|f| (f.expose().to_path_buf(), f.kind()));
        assert_eq!(facts, Some((named(ext), kind)));
    }
}

#[test]
fn another_kind_a_bare_extension_or_no_plain_drive_path_is_refused() {
    let bad = [
        (named("png"), FileKind::Mp4),
        (named("png.exe"), FileKind::Png),
        (PathBuf::from(SNIP), FileKind::Png),
        (PathBuf::from(r"C:\Snips\NUL.png"), FileKind::Png),
        (PathBuf::from(r"C:\Snips.\a.png"), FileKind::Png),
        (PathBuf::from(r"\\host\share\a.png"), FileKind::Png),
        (PathBuf::from(r"C:\Snips\a.png:hidden"), FileKind::Png),
        (PathBuf::from("a.png"), FileKind::Png),
    ];
    let bare = spellings("C", &["Snips", ".png"]).map(|p| (p, FileKind::Png));
    for (path, kind) in bad.into_iter().chain(bare) {
        assert!(SnipFile::new(path.clone(), kind).is_none(), "{path:?}");
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
    for folder in spellings("C", &["Snips", "unsaved"]) {
        let unsaved = FolderPath::new(folder).unwrap();
        for path in spellings("C", &["Snips", "unsaved", "a.mp4"]) {
            assert!(video(path.clone()).lies_in(&unsaved), "{path:?}");
        }
        for path in elsewhere.iter().flatten() {
            assert!(!video(path.clone()).lies_in(&unsaved), "{path:?}");
        }
    }
}

#[test]
fn either_separator_splits_a_path_on_every_system_as_windows_reads_it() {
    let folder = |path: &str| FolderPath::new(PathBuf::from(path)).unwrap();
    let unsaved = folder(r"C:/Snips\unsaved");
    assert!(video(PathBuf::from(r"C:\Snips/unsaved\a.mp4")).lies_in(&unsaved));
    assert!(
        video(PathBuf::from(r"c:\Snips\unsaved\a.mp4")).lies_in(&unsaved),
        "a drive in either case"
    );
    assert!(video(PathBuf::from(r"C:\Snips\unsaved\a.mp4")).lies_in(&folder(r"C:\Snips\unsaved\")));
    let below = video(PathBuf::from(r"C:/Snips/unsaved/old\a.mp4"));
    assert!(!below.lies_in(&unsaved), "a part below, past a backslash");
    let bare = SnipFile::new(PathBuf::from(r"C:/Snips/a.png\.png"), FileKind::Png);
    assert!(bare.is_none(), "a bare extension past a backslash");
}

#[test]
fn a_file_prints_its_kind_and_never_its_path() {
    let shown = format!("{:?}", SnipFile::new(named("mp4"), FileKind::Mp4).unwrap());
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
