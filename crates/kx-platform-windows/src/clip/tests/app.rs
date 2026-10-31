//! The app behind a copy: its exe name in lower case as the id, and its file's description, else the stem, as the name.

use super::*;
use crate::clip::app;

#[test]
fn our_own_process_is_named_after_its_file() {
    let exe = std::env::current_exe().unwrap();
    let id = exe.file_name().unwrap().to_string_lossy().to_lowercase();
    let stem = exe.file_stem().unwrap().to_string_lossy().into_owned();
    let got = app::of(Some(std::process::id())).unwrap();
    assert_eq!(got.id, id);
    assert_eq!(got.name, stem, "a test program has no description");
}

#[test]
fn a_system_program_is_named_by_its_description() {
    let root = std::env::var("SystemRoot").unwrap();
    let cap = table().unwrap().app.version_bytes;
    let name = app::describe(&format!(r"{root}\System32\cmd.exe"), cap).unwrap();
    assert!(!name.trim().is_empty() && !name.eq_ignore_ascii_case("cmd"));
}

#[test]
fn version_information_past_the_cap_is_not_read() {
    let root = std::env::var("SystemRoot").unwrap();
    assert_eq!(app::describe(&format!(r"{root}\System32\cmd.exe"), 1), None);
}

#[test]
fn no_process_has_no_app() {
    assert_eq!(app::of(None), None);
    assert_eq!(app::of(Some(0)), None);
}

#[test]
fn the_name_is_the_description_cut_to_the_cap_else_the_stem() {
    assert_eq!(app::name(Some("Notepad".into()), "notepad", 80), "Notepad");
    assert_eq!(app::name(Some("مفكرة طويلة".into()), "x", 5), "مفكرة");
    assert_eq!(app::name(Some("  ".into()), "notepad", 80), "notepad");
    assert_eq!(app::name(None, "notepad", 80), "notepad");
    assert_eq!(app::name(None, "notepad", 3), "not");
}

#[test]
fn a_program_off_a_drive_is_named_by_its_stem_and_its_file_is_never_read() {
    for path in [r"\server\share\Tool.exe", r"\?\UNC\server\share\Tool.exe"] {
        let read = std::cell::Cell::new(false);
        let got = app::named(path, 80, |_| {
            read.set(true);
            Some("Tool Pro".into())
        });
        assert_eq!(got.unwrap().name, "Tool", "{path}");
        assert!(!read.get(), "{path} would be read over the network");
    }
    let local = app::named(r"C:\Tools\Tool.exe", 80, |_| Some("Tool Pro".into()));
    assert_eq!(local.unwrap().name, "Tool Pro", "a drive path is read");
}
