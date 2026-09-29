//! Changing and saving: checked changes, changes-only text and the unsaved notice.

use crate::spec::{DEFAULTS, HOLD, defaults_with, folder, open, table, with_file};
use kx_module_api::{ModuleId, Notice, SettingsError, keys};
use kx_settings::{Files, SaveError, StoreError, VERSION_KEY};
use std::fs;
use toml::Value;

/// The notice for a failed save.
fn unsaved() -> Notice {
    Notice {
        key: keys::SETTINGS_UNSAVED,
        module: None,
        args: Vec::new(),
    }
}

#[test]
fn set_to_a_default_removes_the_key_from_the_text() {
    let (_dir, files) = with_file("todefault", "[hold]\nmax_ms = 900\nsound = false\n");
    let (mut store, _) = open(&files);
    store.set(HOLD, "max_ms", Value::Integer(500)).unwrap();
    assert_eq!(store.get(HOLD), Some(&defaults_with("sound = false")));
    assert_eq!(
        store.to_text().unwrap(),
        "[hold]\nversion = 3\nsound = false\n"
    );
    store.set(HOLD, "sound", Value::Boolean(true)).unwrap();
    assert_eq!(store.to_text().unwrap(), "");
    assert!(store.needs_save());
}

#[test]
fn an_invalid_value_returns_the_error_and_changes_nothing() {
    let (_dir, files) = with_file("invalid", "[hold]\nmax_ms = 900\n");
    let (mut store, _) = open(&files);
    let before = store.to_text().unwrap();
    let above = store.set(HOLD, "min_ms", Value::Integer(901));
    assert!(matches!(
        above,
        Err(StoreError::Settings(SettingsError::Invalid(_)))
    ));
    let typed = store.set(HOLD, "sound", Value::String("loud".into()));
    assert!(matches!(
        typed,
        Err(StoreError::Settings(SettingsError::Parse(_)))
    ));
    assert_eq!(store.get(HOLD), Some(&defaults_with("max_ms = 900")));
    assert_eq!(store.to_text().unwrap(), before);
    assert!(!store.needs_save());
}

#[test]
fn set_refuses_unknown_modules_newer_sections_and_the_version_key() {
    let other = ModuleId::new("other");
    let (_dir, files) = folder("refuse");
    let (mut store, _) = open(&files);
    let unknown = store.set(other, "max_ms", Value::Integer(1));
    assert!(matches!(unknown, Err(StoreError::Unknown(id)) if id == other));
    let reserved = store.set(HOLD, VERSION_KEY, Value::Integer(3));
    assert!(matches!(reserved, Err(StoreError::Reserved)));
    let (_dir, files) = with_file("refusenewer", "[hold]\nversion = 4\n");
    let (mut store, _) = open(&files);
    let newer = store.set(HOLD, "max_ms", Value::Integer(900));
    assert!(matches!(newer, Err(StoreError::Newer(id)) if id == HOLD));
    assert_eq!(store.to_text().unwrap(), "[hold]\nversion = 4\n");
}

#[test]
fn setting_the_current_value_changes_nothing() {
    let (_dir, files) = folder("same");
    let (mut store, _) = open(&files);
    store.set(HOLD, "max_ms", Value::Integer(500)).unwrap();
    assert!(!store.needs_save());
    assert_eq!(store.get(HOLD), Some(&table(DEFAULTS)));
}

#[test]
fn the_text_puts_the_version_first_then_the_changes() {
    let (_dir, files) = folder("order");
    let (mut store, _) = open(&files);
    store.set(HOLD, "sound", Value::Boolean(false)).unwrap();
    store.set(HOLD, "max_ms", Value::Integer(900)).unwrap();
    let text = "[hold]\nversion = 3\nmax_ms = 900\nsound = false\n";
    assert_eq!(store.to_text().unwrap(), text);
}

#[test]
fn a_change_saved_then_loaded_gives_the_same_settings() {
    let (_dir, files) = folder("roundtrip");
    let (mut store, _) = open(&files);
    store.set(HOLD, "max_ms", Value::Integer(900)).unwrap();
    store.set(HOLD, "min_ms", Value::Integer(800)).unwrap();
    assert!(store.needs_save());
    store.save().unwrap();
    assert!(!store.needs_save());
    let (again, report) = open(&files);
    assert!(report.notices.is_empty());
    assert_eq!(again.get(HOLD), store.get(HOLD));
    assert_eq!(again.to_text().unwrap(), store.to_text().unwrap());
    assert_eq!(
        fs::read_to_string(&files.settings).unwrap(),
        store.to_text().unwrap()
    );
}

#[test]
fn a_failed_save_warns_once_until_a_save_succeeds() {
    let (dir, _) = folder("unsaved");
    let files = Files::new(&dir.path().join("data"));
    let (mut store, _) = open(&files);
    store.set(HOLD, "max_ms", Value::Integer(900)).unwrap();
    fs::write(&files.dir, "a file where the folder goes").unwrap();
    let first = store.save().unwrap_err();
    assert!(matches!(
        first.error,
        StoreError::File(SaveError::CreateDir { .. })
    ));
    assert_eq!(first.notice, Some(unsaved()));
    assert_eq!(store.save().unwrap_err().notice, None);
    assert!(store.needs_save());
    fs::remove_file(&files.dir).unwrap();
    store.save().unwrap();
    fs::create_dir(&files.temp).unwrap();
    assert_eq!(store.save().unwrap_err().notice, Some(unsaved()));
}
