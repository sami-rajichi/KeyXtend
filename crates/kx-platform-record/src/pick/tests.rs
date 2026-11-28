//! The dialogs' neutral half: each kind offers only its own type, an answer prints no path, and a dialog that did not
//! show says so in one line with the system's code and no path.

use super::*;

/// A system code a failed dialog keeps: `E_ACCESSDENIED`.
const DENIED: u32 = 0x8007_0005;

#[test]
fn each_kind_offers_only_its_own_type() {
    assert_eq!(pattern(FileKind::Png), "*.png");
    assert_eq!(pattern(FileKind::Mp4), "*.mp4");
}

#[test]
fn a_dialog_that_did_not_show_says_so_in_one_short_line_with_its_code() {
    let line = Unshown { code: DENIED }.to_string();
    assert!(line.contains("0x80070005"), "{line}");
    assert!(!line.contains('\n'), "{line}");
    assert!(!line.contains('\\') && !line.contains('/'), "{line}");
}

#[test]
fn an_answer_shows_its_paths_length_never_the_path() {
    let path = r"D:\المستندات\private\snip.png";
    let shown = format!("{:?}", Picked::Path(path.to_owned()));
    assert_eq!(shown, format!("Path({})", path.len()));
    assert_eq!(format!("{:?}", Picked::Cancelled), "Cancelled");
}

#[cfg(windows)]
#[test]
fn a_system_error_keeps_its_code() {
    use windows::Win32::Foundation::E_ACCESSDENIED;
    let unshown = Unshown::from(windows::core::Error::from(E_ACCESSDENIED));
    assert_eq!(unshown, Unshown { code: DENIED });
}

#[cfg(not(windows))]
#[test]
fn a_system_without_dialogs_shows_none() {
    let dialogs = LiveDialogs;
    let none = Err(Unshown {
        code: super::none::NOT_BUILT,
    });
    assert_eq!(dialogs.save("/home", "snip.png", FileKind::Png), none);
    assert_eq!(dialogs.folder("/home"), none);
}
