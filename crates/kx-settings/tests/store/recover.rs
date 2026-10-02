//! Recovery at the store level: a restored or defaulted start needs a save, file notices come first,
//! and a locked file is never written.

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
    let expected = vec![file_notice(keys::SETTINGS_RESTORED), reset(&["sound"])];
    assert_eq!(report.notices, expected);
}

/// A file nested far past toml's limit is damaged: a later copy peeks nothing and a load restores the previous copy.
#[test]
fn a_file_nested_too_deep_counts_as_damaged() {
    const DEPTH: usize = 10_000;
    let deep = format!(
        "[hold]\nmax_ms = {}1{}\n",
        "[".repeat(DEPTH),
        "]".repeat(DEPTH)
    );
    let (_dir, files) = with_file("deep", &deep);
    fs::write(&files.previous, "[hold]\nmax_ms = 900\n").unwrap();
    assert_eq!(kx_settings::peek(&files), None);
    let (store, report) = open(&files);
    assert_eq!(report.notices, vec![file_notice(keys::SETTINGS_RESTORED)]);
    assert_eq!(store.get(HOLD), Some(&defaults_with("max_ms = 900")));
    assert_eq!(fs::read_to_string(&files.broken).unwrap(), deep);
}

/// Another program holds the file with no sharing: defaults in memory, one notice, and no write.
#[cfg(windows)]
#[test]
fn a_locked_file_runs_on_defaults_and_is_never_written() {
    use kx_settings::StoreError;
    use toml::Value;
    let (main, previous) = ("[hold]\nmax_ms = 900\n", "[hold]\nmax_ms = 700\n");
    let (_dir, files) = with_file("locked", main);
    fs::write(&files.previous, previous).unwrap();
    let held = kx_test_support::lock::hold(&files.settings).unwrap();
    let (mut store, report) = open(&files);
    assert_eq!(report.notices, vec![file_notice(keys::SETTINGS_LOCKED)]);
    assert_eq!(store.get(HOLD), Some(&table(DEFAULTS)), "no restore");
    store.set(HOLD, "max_ms", Value::Integer(800)).unwrap();
    assert!(!store.needs_save());
    let refused = store.save().unwrap_err();
    drop(held);
    assert!(matches!(refused.error, StoreError::Locked));
    assert_eq!(refused.notice, None, "the load already told the user");
    assert_eq!(fs::read_to_string(&files.settings).unwrap(), main);
    assert_eq!(fs::read_to_string(&files.previous).unwrap(), previous);
    assert!(!files.broken.exists() && !files.temp.exists());
}

/// A file locked only after the load fails that save, stays whole, and a later save catches up.
#[cfg(windows)]
#[test]
fn a_file_locked_after_the_load_fails_the_save_and_stays_whole() {
    use toml::Value;
    let main = "[hold]\nmax_ms = 900\n";
    let (_dir, files) = with_file("lockedlater", main);
    let (mut store, _) = open(&files);
    store.set(HOLD, "max_ms", Value::Integer(800)).unwrap();
    let held = kx_test_support::lock::hold(&files.settings).unwrap();
    let unsaved = store.save().unwrap_err();
    drop(held);
    assert_eq!(unsaved.notice, Some(file_notice(keys::SETTINGS_UNSAVED)));
    assert_eq!(fs::read_to_string(&files.settings).unwrap(), main);
    assert!(!files.broken.exists(), "a locked file is not damaged");
    store.save().unwrap();
    let saved = fs::read_to_string(&files.settings).unwrap();
    assert!(saved.contains("max_ms = 800"), "{saved}");
}
