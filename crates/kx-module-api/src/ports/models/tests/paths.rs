//! The path rule in both styles (ADR-0062): an absolute path within the frames' cap, with no climbing part, stream,
//! control character or device name; Windows takes drive paths and every other system `/` paths.

use super::*;
use PathRoot::{Drive, Slash};

/// Checks that `root`'s style takes each of `good` and refuses each of `bad`.
fn judged(root: PathRoot, good: &[&str], bad: &[&str]) {
    for path in good {
        assert!(path_ok_in(root, Path::new(path)), "{root:?} {path:?}");
    }
    for path in bad {
        assert!(!path_ok_in(root, Path::new(path)), "{root:?} {path:?}");
    }
}

#[test]
fn only_an_absolute_drive_path_is_ok_on_windows() {
    let good = [
        r"D:\KeyXtend\models",
        "C:/models/x.gguf",
        r"E:\a b\c..d.gguf",
        r"D:\",
    ];
    let bad = [
        "",
        "models",
        r"\models",
        "/models",
        r"\\server\share\m",
        r"\\?\D:\m",
        r"\\.\pipe\x",
        "D:models",
    ];
    judged(Drive, &good, &bad);
}

#[test]
fn only_an_absolute_unix_path_is_ok_elsewhere() {
    let good = ["/home/someone/models", "/a b/c..d.gguf", "/"];
    let bad = [
        "",
        "models",
        r"D:\models",
        "D:/models",
        "//server/share/m",
        r"/home\models",
    ];
    judged(Slash, &good, &bad);
}

#[test]
fn the_native_rule_is_this_systems_style() {
    let (ours, theirs) = match PathRoot::NATIVE {
        Drive => (r"D:\models", "/models"),
        Slash => ("/models", r"D:\models"),
    };
    assert!(path_ok(Path::new(ours)));
    assert!(!path_ok(Path::new(theirs)));
}

#[test]
fn a_path_that_climbs_or_names_a_stream_is_refused() {
    let drive = [
        r"D:\a\..\b",
        "D:/a/../b",
        r"D:\a\.\b",
        r"D:\a\.. \b",
        r"D:\a\...",
        r"D:\m\x.gguf:zone",
    ];
    judged(Drive, &[], &drive);
    let unix = [
        "/a/../b",
        "/a/./b",
        "/a/.. /b",
        "/a/...",
        "/m/x.gguf:zone",
        "/m:x",
    ];
    judged(Slash, &[], &unix);
}

#[test]
fn a_path_with_a_control_character_or_a_device_name_is_refused() {
    let bad = [
        "D:\\m\\x\0.gguf",
        "D:\\m\\x\n.gguf",
        "D:\\m\u{7f}\\x.gguf",
        r"D:\m\NUL.gguf",
        r"D:\com1.gguf",
        r"D:\Lpt9\x.gguf",
        r"D:\m\aux",
        r"D:\m\con.tar.gz",
        r"D:\m\prn .gguf",
        r"D:\m\COM¹.gguf",
        r"D:\Lpt³\x.gguf",
        r"D:\m\CONIN$",
        r"D:\m\conout$.gguf",
    ];
    let near = [
        r"D:\m\com10.gguf",
        r"D:\m\nullable.gguf",
        r"D:\console\x.gguf",
    ];
    judged(Drive, &near, &bad);
    let unix_bad = ["/m/x\0.gguf", "/m/x\n.gguf", "/m/NUL.gguf", "/Lpt9/x.gguf"];
    judged(Slash, &["/m/com10.gguf", "/console/x.gguf"], &unix_bad);
}

#[test]
fn a_path_past_the_frames_cap_is_refused() {
    for (root, start) in [(Drive, r"D:\"), (Slash, "/")] {
        let at = |len: usize| format!("{start}{}", "a".repeat(len - start.len()));
        assert!(path_ok_in(root, Path::new(&at(kx_ipc::MAX_PATH_BYTES))));
        assert!(!path_ok_in(
            root,
            Path::new(&at(kx_ipc::MAX_PATH_BYTES + 1))
        ));
    }
}
