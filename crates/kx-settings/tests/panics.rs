//! Module settings code that panics fails only its own module; the store and its file stay intact.
//! This binary installs a panic hook that hides only the sample panics, so the output stays clean.

#![allow(
    clippy::unwrap_used,
    reason = "these fixture helpers run only in tests, where a failed setup should fail the test"
)]

use kx_module_api::{ModuleId, Settings, SettingsError, SettingsSpec, validate_as};
use kx_settings::{Files, LoadReport, Store, StoreError};
use kx_test_support::tempdir::TempDir;
use serde::{Deserialize, Serialize};
use std::panic;
use std::sync::Once;
use toml::{Table, Value};

/// The module whose settings code panics.
const BOOM: ModuleId = ModuleId::new("boom");
/// A sound module beside it.
const CALM: ModuleId = ModuleId::new("calm");
/// The level that makes the sample validator panic.
const TRIGGER: i64 = 13;

/// The payload of every sample panic, so the hook can tell them from real test failures.
struct SamplePanic;

/// Installs, once, a hook that hides sample panics and reports every other one as usual.
fn hide_sample_panics() {
    static HOOK: Once = Once::new();
    HOOK.call_once(|| {
        let usual = panic::take_hook();
        panic::set_hook(Box::new(move |info| {
            if !info.payload().is::<SamplePanic>() {
                usual(info);
            }
        }));
    });
}

/// The sample settings: one level.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Level {
    level: i64,
}

impl Settings for Level {
    fn check(&self) -> Result<(), SettingsError> {
        if self.level == TRIGGER {
            panic::panic_any(SamplePanic);
        }
        Ok(())
    }
}

/// 1 to 2: always panics.
fn explode(_: &mut Table) -> Result<(), SettingsError> {
    panic::panic_any(SamplePanic)
}

/// Version 2 settings whose only migration panics.
static LEVEL: SettingsSpec = SettingsSpec {
    version: 2,
    defaults: "level = 1",
    migrations: &[explode],
    validate: validate_as::<Level>,
};

/// Settings whose defaults make the validator panic.
static TRIGGERED: SettingsSpec = SettingsSpec {
    defaults: "level = 13",
    ..LEVEL
};

/// Loads `text` from a temp folder with `boom` on `spec` and `calm` on `LEVEL`.
fn open(text: &str, spec: &'static SettingsSpec) -> (TempDir, Files, Store, LoadReport) {
    hide_sample_panics();
    let dir = TempDir::new("panics").unwrap();
    let files = Files::new(dir.path());
    std::fs::write(&files.settings, text).unwrap();
    let (store, report) = Store::load(files.clone(), &[(BOOM, spec), (CALM, &LEVEL)]);
    (dir, files, store, report)
}

fn level(value: i64) -> Table {
    Table::from_iter([("level".to_owned(), Value::Integer(value))])
}

#[test]
fn a_panicking_validator_fails_only_its_module_and_its_section_survives() {
    let text = "[boom]\nlevel = 13\n\n[calm]\nlevel = 5\n";
    let (_dir, files, mut store, report) = open(text, &LEVEL);
    assert_eq!(report.broken, [BOOM]);
    assert!(report.notices.is_empty());
    assert_eq!(store.get(BOOM), None);
    assert_eq!(store.get(CALM), Some(&level(5)));
    store.save().unwrap();
    let saved = std::fs::read_to_string(&files.settings).unwrap();
    assert_eq!(
        saved,
        "[boom]\nlevel = 13\n\n[calm]\nversion = 2\nlevel = 5\n"
    );
}

#[test]
fn a_panicking_migration_fails_only_its_module_and_its_section_survives() {
    let text = "[boom]\nlevel = 2\nversion = 1\n\n[calm]\nlevel = 5\n";
    let (_dir, _files, store, report) = open(text, &LEVEL);
    assert_eq!(report.broken, [BOOM]);
    assert_eq!(store.get(CALM), Some(&level(5)));
    let kept = "[boom]\nlevel = 2\nversion = 1\n\n[calm]\nversion = 2\nlevel = 5\n";
    assert_eq!(store.to_text().unwrap(), kept);
}

#[test]
fn a_validator_that_panics_on_its_defaults_fails_only_its_module() {
    let (_dir, _files, store, report) = open("[calm]\nlevel = 5\n", &TRIGGERED);
    assert_eq!(report.broken, [BOOM]);
    assert_eq!(store.get(CALM), Some(&level(5)));
}

#[test]
fn a_panicking_validator_in_set_changes_nothing() {
    let (_dir, _files, mut store, _) = open("[calm]\nlevel = 5\n", &LEVEL);
    let hit = store.set(CALM, "level", Value::Integer(TRIGGER));
    assert!(matches!(hit, Err(StoreError::Panicked(id)) if id == CALM));
    assert_eq!(store.get(CALM), Some(&level(5)));
    assert!(!store.needs_save());
    store.set(CALM, "level", Value::Integer(7)).unwrap();
    assert_eq!(store.get(CALM), Some(&level(7)));
}
