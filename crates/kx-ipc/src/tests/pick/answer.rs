//! The answer check the pick job and its host both run (ADR-0056 decision 6, ADR-0062), in both path styles on every
//! system: a Save answers an absolute path of its kind and a folder ask a folder; UNC, device, relative and climbing
//! paths, the other style's paths, parts Win32 trims, another type, an over-long name and a path the reply frame
//! cannot carry fail.

use super::*;
use crate::rules::PathRoot::{self, Drive, Slash};

/// The most UTF-16 units Windows allows in one name.
const LONGEST: usize = crate::limits::snip::NAME_UNITS;
/// A character Windows keeps in two UTF-16 units.
const PAIRED: char = '😀';

/// Answers off a drive, relative, climbing, empty-parted, with a part Win32 trims to another name (a trailing dot or
/// space), or naming a stream, a device or a control character: no ask takes them, whatever their type.
const OFF_DRIVE: [&str; 28] = [
    r"\\server\share\snip.png",
    r"\\?\C:\snip.png",
    r"\\.\C:\snip.png",
    r"//?/C:/snip.png",
    r"\??\C:\snip.png",
    r"snip.png",
    r"snips\snip.png",
    r"C:snip.png",
    r"\snip.png",
    r"/snips/snip.png",
    r"1:\snip.png",
    r"CC:\snip.png",
    r"C:\snips\..\snip.png",
    r"C:\snips\.\snip.png",
    r"C:\..",
    r"C:\snips\\snip.png",
    r"C:\snips\snip.png:hidden.png",
    r"C:\con\snip.png",
    "C:\\a\tb\\snip.png",
    r"C:\snips\",
    "",
    r"C:\snips.\snip.png",
    r"C:\snips \snip.png",
    r"C:\...\snip.png",
    r"C:\. .\snip.png",
    r"C:\.. \snip.png",
    r"C:\a\.. ",
    r"C:\snips\snip.png ",
];

/// The same refusals for Unix paths, with a drive path, a backslash and a leading `//`, which may name a share.
const OFF_ROOT: [&str; 20] = [
    r"//server/share/snip.png",
    r"//snip.png",
    r"snip.png",
    r"snips/snip.png",
    r"C:\snips\snip.png",
    r"C:/snips/snip.png",
    r"/snips\snip.png",
    r"\snips\snip.png",
    r"/snips/../snip.png",
    r"/snips/./snip.png",
    r"/..",
    r"/snips//snip.png",
    r"/snips/snip.png:hidden.png",
    r"/con/snip.png",
    "/a\tb/snip.png",
    r"/snips/",
    "",
    r"/snips./snip.png",
    r"/snips /snip.png",
    r"/snips/snip.png ",
];

/// Save answers of another type than PNG, or with no name before the extension.
const NOT_PNG: [&str; 7] = [
    r"C:\snips\snip.jpg",
    r"C:\snips\snip",
    r"C:\snips\snip.png.jpg",
    r"C:\snips\snip.png.",
    r"C:\snips\.png",
    r"C:\snips\snip.mp4",
    r"D:\",
];

/// A PNG Save ask at a folder of its own.
fn png_ask() -> PickAsk<'static> {
    save(SNIPS, "snip.png", FileKind::Png)
}

/// A folder path of exactly `len` bytes from `start`, in parts of `LONGEST` units but the last, each after `sep`.
fn deep(start: &str, sep: char, len: usize) -> String {
    let mut path = String::from(start);
    while path.len() < len {
        let room = len - path.len() - 1;
        path.push(sep);
        path.push_str(&"d".repeat(room.min(LONGEST)));
    }
    assert!(path.len() == len && !path.ends_with(sep), "{}", path.len());
    path
}

/// A name of `units` UTF-16 units, with `ext` after it.
fn long_name(units: usize, ext: &str) -> String {
    let stem = units - ext.len() - 1;
    format!("{}.{ext}", "a".repeat(stem))
}

/// Checks that every ask in `root`'s style refuses each of `answers`.
fn all_refuse(root: PathRoot, answers: &[&str]) {
    for answer in answers {
        let png = png_ask().check_in(root, answer);
        assert_eq!(png, Err(IpcError::BadField), "{root:?} {answer:?}");
        let as_folder = answer.trim_end_matches(".png");
        let refused = folder(SNIPS).check_in(root, as_folder);
        assert_eq!(refused, Err(IpcError::BadField), "{root:?} {as_folder:?}");
    }
}

#[test]
fn a_save_takes_a_drive_path_of_its_kind_and_a_folder_ask_a_drive_folder() {
    let png = png_ask();
    for answer in [
        r"C:\Users\Someone\Pictures\Snip 1.png",
        r"C:\snips\SNIP.PNG",
        r"C:/snips/snip.png",
        r"z:\snip.png",
        CHOSEN,
    ] {
        assert_eq!(png.check_in(Drive, answer), Ok(()), "{answer}");
    }
    let mp4 = save(SNIPS, "snip.mp4", FileKind::Mp4);
    assert_eq!(mp4.check_in(Drive, r"C:\snips\snip.mp4"), Ok(()));
    let png_file = r"C:\snips\snip.png";
    assert_eq!(mp4.check_in(Drive, png_file), Err(IpcError::BadField));
    for answer in [r"D:\", "D:/", SNIPS, ARABIC, png_file] {
        assert_eq!(folder(SNIPS).check_in(Drive, answer), Ok(()), "{answer}");
    }
}

#[test]
fn a_save_takes_a_unix_path_of_its_kind_and_a_folder_ask_a_unix_folder() {
    for answer in [
        "/home/someone/Pictures/Snip 1.png",
        "/snips/SNIP.PNG",
        "/snip.png",
    ] {
        assert_eq!(png_ask().check_in(Slash, answer), Ok(()), "{answer}");
    }
    let arabic = "/المستندات/لقطاتي";
    for answer in ["/", "/home/someone/Pictures", arabic] {
        assert_eq!(folder(SNIPS).check_in(Slash, answer), Ok(()), "{answer}");
    }
}

#[test]
fn a_root_is_a_drives_three_bytes_or_one_slash() {
    assert_eq!(Drive.len_of(br"c:/x"), Some(3));
    assert_eq!(Slash.len_of(b"/x"), Some(1));
    assert_eq!(Slash.len_of(b"/"), Some(1));
    for odd in [&b"C:"[..], b"/x", br"\\x"] {
        assert_eq!(Drive.len_of(odd), None, "{odd:?}");
    }
    for odd in [&b"//x"[..], br"/a\b", br"C:\x", b""] {
        assert_eq!(Slash.len_of(odd), None, "{odd:?}");
    }
}

#[test]
fn the_native_check_is_this_systems_style() {
    let (ours, theirs) = match PathRoot::NATIVE {
        Drive => (r"C:\snips\snip.png", "/snips/snip.png"),
        Slash => ("/snips/snip.png", r"C:\snips\snip.png"),
    };
    assert_eq!(png_ask().check(ours), Ok(()));
    assert_eq!(png_ask().check(theirs), Err(IpcError::BadField));
}

#[test]
fn an_answer_off_a_drive_is_refused_for_every_ask() {
    all_refuse(Drive, &OFF_DRIVE);
}

#[test]
fn an_answer_off_the_unix_root_is_refused_for_every_ask() {
    all_refuse(Slash, &OFF_ROOT);
}

#[test]
fn a_save_answer_of_another_type_is_refused() {
    for answer in NOT_PNG {
        let got = png_ask().check_in(Drive, answer);
        assert_eq!(got, Err(IpcError::BadField), "{answer}");
    }
}

#[test]
fn a_name_past_windows_longest_is_refused_in_both_styles() {
    for (root, dir) in [(Drive, r"C:\snips\"), (Slash, "/snips/")] {
        let at = |name: &str| format!("{dir}{name}");
        let png = png_ask();
        assert_eq!(
            png.check_in(root, &at(&long_name(LONGEST, PNG_EXT))),
            Ok(())
        );
        let over = at(&long_name(LONGEST + 1, PNG_EXT));
        assert_eq!(png.check_in(root, &over), Err(IpcError::BadField));
        let arabic = format!("{}.{PNG_EXT}", "ل".repeat(LONGEST - PNG_EXT.len() - 1));
        assert_eq!(png.check_in(root, &at(&arabic)), Ok(()), "units, not bytes");
        let paired = format!("{PAIRED}{}", long_name(LONGEST - 1, PNG_EXT));
        let got = png.check_in(root, &at(&paired));
        assert_eq!(got, Err(IpcError::BadField), "a pair is two units");
        let folder_part = at(&"b".repeat(LONGEST + 1));
        assert_eq!(
            folder(SNIPS).check_in(root, &folder_part),
            Err(IpcError::BadField)
        );
    }
}

#[test]
fn an_answer_the_reply_frame_cannot_carry_is_refused_and_one_it_can_is_told() {
    for (root, start, sep) in [(Drive, "C:", '\\'), (Slash, "/d", '/')] {
        let longest = deep(start, sep, MAX_PATH_BYTES);
        assert_eq!(folder(SNIPS).check_in(root, &longest), Ok(()));
        let told = picked(&longest).write(&mut Vec::new());
        assert!(told.is_ok(), "what passes is told");
        let over = deep(start, sep, MAX_PATH_BYTES + 1);
        assert_eq!(folder(SNIPS).check_in(root, &over), Err(IpcError::BadField));
        assert!(picked(&over).write(&mut Vec::new()).is_err());
    }
}

#[test]
fn goodbye_takes_no_answer() {
    for answer in [CHOSEN, SNIPS, "/snips"] {
        assert_eq!(PickAsk::Bye.check(answer), Err(IpcError::BadField));
    }
}
