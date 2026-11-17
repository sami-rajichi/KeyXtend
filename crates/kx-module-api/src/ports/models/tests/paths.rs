//! The path rule: an absolute drive path within the frames' cap, with no climbing part, stream, control character or
//! device name.

use super::*;

#[test]
fn only_an_absolute_drive_path_is_ok() {
    for good in [
        r"D:\KeyXtend\models",
        "C:/models/x.gguf",
        r"E:\a b\c..d.gguf",
        r"D:\",
    ] {
        assert!(path_ok(Path::new(good)), "{good}");
    }
    for bad in [
        "",
        "models",
        r"\models",
        r"\\server\share\m",
        r"\\?\D:\m",
        r"\\.\pipe\x",
        "D:models",
    ] {
        assert!(!path_ok(Path::new(bad)), "{bad}");
    }
}

#[test]
fn a_path_that_climbs_or_names_a_stream_is_refused() {
    let bad = [
        r"D:\a\..\b",
        "D:/a/../b",
        r"D:\a\.\b",
        r"D:\a\.. \b",
        r"D:\a\...",
        r"D:\m\x.gguf:zone",
    ];
    for path in bad {
        assert!(!path_ok(Path::new(path)), "edge 38: {path}");
    }
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
    for path in bad {
        assert!(!path_ok(Path::new(path)), "{path:?}");
    }
    let near = [
        r"D:\m\com10.gguf",
        r"D:\m\nullable.gguf",
        r"D:\console\x.gguf",
    ];
    for path in near {
        assert!(path_ok(Path::new(path)), "{path}");
    }
}

#[test]
fn a_path_past_the_frames_cap_is_refused() {
    let at = |len: usize| format!("{DRIVE_ROOT}{}", "a".repeat(len - DRIVE_ROOT.len()));
    assert!(path_ok(Path::new(&at(kx_ipc::MAX_PATH_BYTES))));
    assert!(!path_ok(Path::new(&at(kx_ipc::MAX_PATH_BYTES + 1))));
}
