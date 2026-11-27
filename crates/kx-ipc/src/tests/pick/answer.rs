//! The answer check the pick job and its host both run (ADR-0056 decision 6): a Save answers a drive path of its kind
//! and a folder ask a drive folder; UNC, device, relative and climbing paths, parts Win32 trims, another type, an
//! over-long name and a path the reply frame cannot carry fail.

use super::*;

/// The most UTF-16 units Windows allows in one name.
const LONGEST: usize = crate::limits::snip::NAME_UNITS;
/// A character Windows keeps in two UTF-16 units.
const PAIRED: char = '😀';

/// Answers off a drive, relative, climbing, empty-parted, with a part Win32 trims to another name (a trailing dot or
/// space), or naming a stream, a device or a control character: no ask takes them, whatever their type.
const OFF_DRIVE: [&str; 27] = [
    r"\\server\share\snip.png",
    r"\\?\C:\snip.png",
    r"\\.\C:\snip.png",
    r"//?/C:/snip.png",
    r"\??\C:\snip.png",
    r"snip.png",
    r"snips\snip.png",
    r"C:snip.png",
    r"\snip.png",
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

/// A drive folder path of exactly `len` bytes, in parts of `LONGEST` units but the last.
fn deep(len: usize) -> String {
    let mut path = String::from("C:");
    while path.len() < len {
        let room = len - path.len() - 1;
        path.push('\\');
        path.push_str(&"d".repeat(room.min(LONGEST)));
    }
    assert!(path.len() == len && !path.ends_with('\\'), "{}", path.len());
    path
}

/// A name of `units` UTF-16 units, with `ext` after it.
fn long_name(units: usize, ext: &str) -> String {
    let stem = units - ext.len() - 1;
    format!("{}.{ext}", "a".repeat(stem))
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
        assert_eq!(png.check(answer), Ok(()), "{answer}");
    }
    let mp4 = save(SNIPS, "snip.mp4", FileKind::Mp4);
    assert_eq!(mp4.check(r"C:\snips\snip.mp4"), Ok(()));
    assert_eq!(mp4.check(r"C:\snips\snip.png"), Err(IpcError::BadField));
    for answer in [r"D:\", "D:/", SNIPS, ARABIC, r"C:\snips\snip.png"] {
        assert_eq!(folder(SNIPS).check(answer), Ok(()), "{answer}");
    }
}

#[test]
fn an_answer_off_a_drive_is_refused_for_every_ask() {
    for answer in OFF_DRIVE {
        assert_eq!(
            png_ask().check(answer),
            Err(IpcError::BadField),
            "{answer:?}"
        );
        let as_folder = answer.trim_end_matches(".png");
        let refused = folder(SNIPS).check(as_folder);
        assert_eq!(refused, Err(IpcError::BadField), "{as_folder:?}");
    }
}

#[test]
fn a_save_answer_of_another_type_is_refused() {
    for answer in NOT_PNG {
        assert_eq!(png_ask().check(answer), Err(IpcError::BadField), "{answer}");
    }
}

#[test]
fn a_name_past_windows_longest_is_refused() {
    let at = |name: &str| format!(r"C:\snips\{name}");
    let png = png_ask();
    assert_eq!(png.check(&at(&long_name(LONGEST, PNG_EXT))), Ok(()));
    let over = at(&long_name(LONGEST + 1, PNG_EXT));
    assert_eq!(png.check(&over), Err(IpcError::BadField));
    let arabic = format!("{}.{PNG_EXT}", "ل".repeat(LONGEST - PNG_EXT.len() - 1));
    assert_eq!(png.check(&at(&arabic)), Ok(()), "units, not bytes");
    let paired = format!("{PAIRED}{}", long_name(LONGEST - 1, PNG_EXT));
    assert_eq!(
        png.check(&at(&paired)),
        Err(IpcError::BadField),
        "a pair is two units"
    );
    let folder_part = format!(r"C:\{}", "b".repeat(LONGEST + 1));
    assert_eq!(folder(SNIPS).check(&folder_part), Err(IpcError::BadField));
}

#[test]
fn an_answer_the_reply_frame_cannot_carry_is_refused_and_one_it_can_is_told() {
    let longest = deep(MAX_PATH_BYTES);
    assert_eq!(folder(SNIPS).check(&longest), Ok(()));
    assert!(
        picked(&longest).write(&mut Vec::new()).is_ok(),
        "what passes is told"
    );
    let over = deep(MAX_PATH_BYTES + 1);
    assert_eq!(folder(SNIPS).check(&over), Err(IpcError::BadField));
    assert!(picked(&over).write(&mut Vec::new()).is_err());
}

#[test]
fn goodbye_takes_no_answer() {
    for answer in [CHOSEN, SNIPS] {
        assert_eq!(PickAsk::Bye.check(answer), Err(IpcError::BadField));
    }
}
