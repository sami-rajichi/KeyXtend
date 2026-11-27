//! The clipboard ports: their gates, `Debug` that never shows a copied part or the app, the errors, and the path
//! forms an adapter may probe, one per system.

use super::*;
use crate::REDACTED_MARKER;
use std::collections::HashSet;

/// Texts that appear nowhere else in the output: a Latin one and an Arabic one ("hello").
const SECRETS: [&str; 2] = ["zebra-quartz", "مرحبا"];
/// The app the copies come from, by id and by name.
const APP: [&str; 2] = ["keepass.exe", "KeePass"];
/// Paths no adapter may probe: shares, devices, NT paths, relative paths and a drive with no root.
const FAR: [&str; 14] = [
    r"\\host\share\a.txt",
    r"\\?\C:\a.txt",
    r"\\?\UNC\host\share\a.txt",
    r"\\.\PhysicalDrive0",
    r"\??\UNC\host\share\a.txt",
    r"\??\C:\a.txt",
    "//host/share/a.txt",
    r"\a.txt",
    "a.txt",
    r"docs\تقرير.pdf",
    "C:a.txt",
    "C:",
    r"1:\a.txt",
    "",
];
/// Absolute drive paths, the one form an adapter may probe on Windows.
const NEAR: [&str; 4] = [r"C:\a.txt", "c:/a.txt", r"Z:\docs\تقرير.pdf", r"C:\"];
/// Paths no adapter may probe off Windows: a leading `//`, a backslash, a relative path and a drive path.
const UNIX_FAR: [&str; 7] = [
    "//host/share/a.txt",
    r"/docs\a.txt",
    r"\a.txt",
    "a.txt",
    "docs/تقرير.pdf",
    r"C:\a.txt",
    "",
];
/// Absolute Unix paths, the one form an adapter may probe off Windows.
const UNIX_NEAR: [&str; 3] = ["/home/a.txt", "/docs/تقرير.pdf", "/"];

/// Secret `n` as text.
fn text(n: usize) -> String {
    SECRETS[n].to_owned()
}

/// One part of each kind, each holding a secret.
fn parts() -> Vec<ClipPart> {
    let bytes = || Redacted::new(text(0).into_bytes());
    vec![
        ClipPart::Text(Redacted::new(text(1))),
        ClipPart::Html(bytes()),
        ClipPart::Rtf(bytes()),
        ClipPart::Image {
            codec: ImageCodec::Png,
            bytes: bytes(),
        },
        ClipPart::Files(Redacted::new(vec![PathBuf::from(text(0))])),
    ]
}

/// The app, by its secret names.
fn app() -> AppRef {
    AppRef {
        id: APP[0].to_owned(),
        name: APP[1].to_owned(),
    }
}

/// What a leak would print: the secrets, and the secret's bytes as `Vec<u8>` prints them.
fn leaks() -> Vec<String> {
    let raw = format!("{:?}", text(0).into_bytes());
    SECRETS
        .iter()
        .chain(&APP)
        .map(|s| (*s).to_owned())
        .chain([raw])
        .collect()
}

#[test]
fn the_watch_needs_read_clipboard_and_the_writer_write_clipboard() {
    assert_eq!(ClipboardKey::ID.as_str(), "clipboard");
    assert_eq!(ClipboardKey::CAPABILITY, Some(Capability::ReadClipboard));
    assert_eq!(ClipboardWriterKey::ID.as_str(), "clipboard-writer");
    assert_eq!(
        ClipboardWriterKey::CAPABILITY,
        Some(Capability::WriteClipboard)
    );
}

#[test]
fn debug_never_shows_a_part_or_the_app() {
    let notes = [
        ClipNote::Copied {
            parts: parts(),
            app: Some(app()),
        },
        ClipNote::Marked { app: Some(app()) },
        ClipNote::Cleared { app: Some(app()) },
        ClipNote::TooBig {
            app: Some(app()),
            bytes: 9,
        },
    ];
    let parts = parts().iter().map(|p| format!("{p:?}")).collect::<Vec<_>>();
    let notes = notes.iter().map(|n| format!("{n:?}"));
    for shown in parts.into_iter().chain(notes) {
        assert!(shown.contains(REDACTED_MARKER), "{shown}");
        for leak in leaks() {
            assert!(!shown.contains(&leak), "{shown} shows {leak}");
        }
    }
}

#[test]
fn a_note_still_shows_its_kind_size_and_codec() {
    let big = ClipNote::TooBig {
        app: None,
        bytes: 9,
    };
    assert_eq!(format!("{big:?}"), "TooBig { app: None, bytes: 9 }");
    assert!(format!("{:?}", parts()[3]).contains("Png"));
}

#[test]
fn each_error_says_something_else() {
    let errors = [
        ClipError::Busy,
        ClipError::Refused,
        ClipError::Io,
        ClipError::TooLong,
        ClipError::Marked,
    ];
    let texts: HashSet<_> = errors.iter().map(ToString::to_string).collect();
    assert_eq!(texts.len(), errors.len());
}

#[test]
fn only_an_absolute_path_of_each_style_may_be_probed() {
    let styles = [
        (PathRoot::Drive, &FAR[..], &NEAR[..]),
        (PathRoot::Slash, &UNIX_FAR[..], &UNIX_NEAR[..]),
    ];
    for (root, far, near) in styles {
        for path in far {
            assert!(!probe_in(root, Path::new(path)), "{root:?} {path}");
        }
        for path in near {
            assert!(probe_in(root, Path::new(path)), "{root:?} {path}");
        }
    }
    let ours = if PathRoot::NATIVE == PathRoot::Drive {
        NEAR[0]
    } else {
        UNIX_NEAR[0]
    };
    assert!(probe_ok(Path::new(ours)), "the native style");
}

#[cfg(windows)]
#[test]
fn the_drive_rule_matches_a_drive_prefix_then_a_root_as_windows_parses_them() {
    use std::path::{Component, Prefix};
    for path in FAR.iter().chain(&NEAR) {
        let mut parts = Path::new(path).components();
        let disk = matches!(parts.next(), Some(Component::Prefix(p)) if matches!(p.kind(), Prefix::Disk(_)));
        let rooted = parts.next() == Some(Component::RootDir);
        let probed = probe_in(PathRoot::Drive, Path::new(path));
        assert_eq!(probed, disk && rooted, "{path}");
    }
}
