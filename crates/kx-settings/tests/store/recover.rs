//! Recovery at the store level: a restored or defaulted start needs a save, and its notices come first.

use crate::spec::{DEFAULTS, HOLD, defaults_with, folder, open, reset, table, with_file};
use kx_module_api::{Notice, keys};
use std::fs;

/// A notice about the file as a whole.
fn file_notice(key: &'static str) -> Notice {
    Notice {
        key,
        module: None,
        args: Vec::new(),
    }
}

#[test]
fn a_restored_file_needs_a_save_and_a_clean_one_does_not() {
    let (_dir, files) = folder("restored");
    fs::write(&files.previous, "[hold]\nmax_ms = 900\n").unwrap();
    let (mut store, report) = open(&files);
    assert_eq!(report.notices, vec![file_notice(keys::SETTINGS_RESTORED)]);
    assert!(store.needs_save());
    store.save().unwrap();
    assert!(!store.needs_save());
    let (again, report) = open(&files);
    assert!(report.notices.is_empty());
    assert!(!again.needs_save());
    assert_eq!(again.get(HOLD), Some(&defaults_with("max_ms = 900")));
}

#[test]
fn a_defaults_start_needs_a_save_and_keeps_the_damage() {
    let (_dir, files) = with_file("defaults", "[hold\nsound = = 1\n");
    fs::write(&files.previous, [0xff, 0xfe]).unwrap();
    let (mut store, report) = open(&files);
    assert_eq!(report.notices, vec![file_notice(keys::SETTINGS_DEFAULTS)]);
    assert_eq!(store.get(HOLD), Some(&table(DEFAULTS)));
    assert!(store.needs_save());
    store.save().unwrap();
    assert!(!store.needs_save());
    assert!(files.broken.exists());
}

#[test]
fn the_file_notice_comes_before_the_section_notices() {
    let (_dir, files) = folder("order");
    fs::write(&files.previous, "[hold]\nsound = \"loud\"\n").unwrap();
    let (_, report) = open(&files);
    let expected = vec![file_notice(keys::SETTINGS_RESTORED), reset("sound")];
    assert_eq!(report.notices, expected);
}
