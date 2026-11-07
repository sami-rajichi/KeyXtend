//! The places table: the shipped one parses into Windows' own numbers, a broken one is refused, and temp names, drive
//! types, drive names, attributes and error codes read as the port wants.

use super::*;
use crate::places::TABLE;
use crate::redirect::one_name;

/// A small good table, whose lines `with` swaps.
const GOOD: &str = r#"
max_listed = 3
max_subfolders = 2
temp_start = "~"
temp_gap = "-"
temp_end = ".tmp"
temp_stale_s = 60
labelled = "{label} ({drive})"
hidden = [2]
[drives]
removable = [2]
fixed = [3]
remote = [4]
[errors]
missing = [3]
full = [112]
read_only = [5]
[[cloud]]
name = "Cloud"
env = "CLOUD"
parts = ["Box"]
"#;

/// `GOOD` with the line `line` swapped for `new`.
fn with(line: &str, new: &str) -> String {
    assert!(GOOD.contains(line), "{line}");
    GOOD.replace(line, new)
}

/// The shipped table.
fn shipped() -> Table {
    Table::parse(TABLE).unwrap()
}

#[test]
fn the_shipped_table_parses_with_its_limits_set() {
    let t = shipped();
    assert!(t.max_listed > 0 && t.max_subfolders > 0 && t.temp_stale_s > 0);
    assert!(!t.cloud.is_empty());
}

#[test]
fn each_write_gets_its_own_temp_name_which_names_its_process_and_is_no_box() {
    let t = shipped();
    let names = [t.temp_name(7, 0), t.temp_name(7, 1), t.temp_name(8, 0)];
    assert!(names[0] != names[1] && names[0] != names[2] && names[1] != names[2]);
    for name in &names {
        assert!(
            one_name(name) && !is_box_name(name) && *name != box_file(),
            "{name}"
        );
    }
    assert_eq!(t.temp_owner(&names[0]), Some(7));
    assert_eq!(t.temp_owner(&names[2]), Some(8));
    let widest = t.temp_name(u32::MAX, u64::MAX);
    assert_eq!(t.temp_owner(&widest), Some(u32::MAX));
    let others = [
        box_file(),
        format!("~{}.tmp", box_file()),
        names[0].replace('7', "x"),
        names[0].replacen(&t.temp_end, "", 1),
        format!("a{}", names[0]),
    ];
    for other in others {
        assert_eq!(t.temp_owner(&other), None, "{other}");
    }
}

#[test]
fn a_table_with_an_empty_value_a_path_or_no_single_base_is_refused() {
    assert!(Table::parse(GOOD).is_ok());
    let label = with("env = \"CLOUD\"", "label = \"Cloud\"");
    assert!(Table::parse(&label).is_ok(), "a labelled drive is a base");
    let bad = [
        with("max_listed = 3", "max_listed = 0"),
        with("max_subfolders = 2", "max_subfolders = 0"),
        with("temp_stale_s = 60", "temp_stale_s = 0"),
        with("temp_gap = \"-\"", "temp_gap = \"\""),
        with("temp_gap = \"-\"", "temp_gap = \"1\""),
        with("temp_end = \".tmp\"", "temp_end = \".kxbox\""),
        with("temp_end = \".tmp\"", "temp_end = \"/x\""),
        with("labelled = \"{label} ({drive})\"", "labelled = \"{label}\""),
        with("hidden = [2]", "hidden = []"),
        with("removable = [2]", "removable = []"),
        with("missing = [3]", "missing = []"),
        with("name = \"Cloud\"", "name = \"\""),
        with("env = \"CLOUD\"", "env = \"\""),
        with("env = \"CLOUD\"", ""),
        with("env = \"CLOUD\"", "env = \"CLOUD\"\nlabel = \"Cloud\""),
        with("parts = [\"Box\"]", "parts = [\"..\"]"),
        with("parts = [\"Box\"]", "parts = [\"a\\\\b\"]"),
    ];
    for text in bad {
        assert_eq!(Table::parse(&text).err(), Some(TableError::Bad), "{text}");
    }
    let unknown = format!("extra = 1\n{GOOD}");
    assert!(matches!(Table::parse(&unknown), Err(TableError::Parse(_))));
}

#[test]
fn drive_types_become_kinds_network_drives_are_marked_and_the_rest_are_left_out() {
    let t = shipped();
    assert_eq!(t.kind(2), Some(PlaceKind::Removable));
    for code in [3, 4, 6] {
        assert_eq!(t.kind(code), Some(PlaceKind::Drive), "{code}");
    }
    assert!(t.remote(4) && !t.remote(2) && !t.remote(3) && !t.remote(6));
    // Unknown, no root, an optical drive and a type Windows does not have.
    for code in [0, 1, 5, 7] {
        assert_eq!(t.kind(code), None, "{code}");
    }
}
#[test]
fn a_drive_shows_its_letter_and_its_label_when_it_has_one() {
    let t = shipped();
    let root = Path::new("E:\\");
    assert_eq!(t.drive_name(root, ""), "E:");
    assert_eq!(t.drive_name(root, "SANDISK"), "SANDISK (E:)");
    assert_eq!(t.drive_name(root, "ملفاتي"), "ملفاتي (E:)");
}

#[test]
fn hidden_and_system_entries_hide_and_a_cloud_file_never_does() {
    let t = shipped();
    for attributes in [0x2, 0x4, 0x6, 0x12] {
        assert!(t.hides(attributes), "{attributes:#x}");
    }
    // A folder, an archive, a reparse point, and OneDrive's pinned, online-only and recall-on-read files.
    for attributes in [0, 0x10, 0x20, 0x400, 0x8_0000, 0x10_0000, 0x40_0000] {
        assert!(!t.hides(attributes), "{attributes:#x}");
    }
}

#[test]
fn windows_errors_become_the_port_errors_and_any_other_is_io() {
    let t = shipped();
    let of = |code| t.error(&io::Error::from_raw_os_error(code));
    let cases = [
        (3, PlaceError::Missing),
        (21, PlaceError::Missing),
        (1167, PlaceError::Missing),
        (112, PlaceError::Full),
        (5, PlaceError::ReadOnly),
        (19, PlaceError::ReadOnly),
        (32, PlaceError::Io),
    ];
    for (code, want) in cases {
        assert_eq!(of(code), want, "{code}");
    }
    assert_eq!(t.error(&io::Error::other("no code")), PlaceError::Io);
}

#[test]
fn a_call_keeps_only_the_errors_it_may_return() {
    let read = [PlaceError::Missing];
    assert_eq!(only(PlaceError::Missing, &read), PlaceError::Missing);
    assert_eq!(only(PlaceError::ReadOnly, &read), PlaceError::Io);
    assert_eq!(only(PlaceError::Full, &read), PlaceError::Io);
}

#[cfg(windows)]
#[test]
fn the_table_numbers_are_windows_own() {
    use windows::Win32::Foundation as e;
    use windows::Win32::Storage::FileSystem::{FILE_ATTRIBUTE_HIDDEN, FILE_ATTRIBUTE_SYSTEM};
    use windows::Win32::System::WindowsProgramming as d;
    let t = shipped();
    assert_eq!(t.hidden, [FILE_ATTRIBUTE_HIDDEN.0, FILE_ATTRIBUTE_SYSTEM.0]);
    assert_eq!(t.drives.removable, [d::DRIVE_REMOVABLE]);
    assert_eq!(t.drives.fixed, [d::DRIVE_FIXED, d::DRIVE_RAMDISK]);
    assert_eq!(t.drives.remote, [d::DRIVE_REMOTE]);
    let codes = |list: &[e::WIN32_ERROR]| -> Vec<i32> {
        list.iter().map(|c| i32::try_from(c.0).unwrap()).collect()
    };
    let missing = [
        e::ERROR_FILE_NOT_FOUND,
        e::ERROR_PATH_NOT_FOUND,
        e::ERROR_INVALID_DRIVE,
        e::ERROR_NOT_READY,
        e::ERROR_BAD_NETPATH,
        e::ERROR_DEV_NOT_EXIST,
        e::ERROR_NETNAME_DELETED,
        e::ERROR_BAD_NET_NAME,
        e::ERROR_DIRECTORY,
        e::ERROR_NO_MEDIA_IN_DRIVE,
        e::ERROR_DEVICE_NOT_CONNECTED,
    ];
    assert_eq!(t.errors.missing, codes(&missing));
    let full = [
        e::ERROR_HANDLE_DISK_FULL,
        e::ERROR_DISK_FULL,
        e::ERROR_DISK_QUOTA_EXCEEDED,
    ];
    assert_eq!(t.errors.full, codes(&full));
    let read_only = [
        e::ERROR_ACCESS_DENIED,
        e::ERROR_WRITE_PROTECT,
        e::ERROR_FILE_READ_ONLY,
    ];
    assert_eq!(t.errors.read_only, codes(&read_only));
}
