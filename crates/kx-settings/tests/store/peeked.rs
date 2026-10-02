//! `peek_section`: a later copy reads one module's section over its defaults, without the store or any file.

#![allow(clippy::unwrap_used, reason = "a failed setup should fail the test")]

use kx_module_api::{SettingsError, SettingsSpec, validate_as};
use kx_settings::peek_section;
use toml::Table;

use crate::spec::{DEFAULTS, HOLD, Hold, SPEC, VERSION};

/// The sample spec with its migrations lost, so the version no longer matches them.
static GAPPED: SettingsSpec = SettingsSpec {
    version: VERSION,
    defaults: DEFAULTS,
    migrations: &[],
    validate: validate_as::<Hold>,
};

/// A spec whose own defaults do not read as `Hold`.
static UNREADABLE: SettingsSpec = SettingsSpec {
    version: 1,
    defaults: "min_ms = \"x\"\n",
    migrations: &[],
    validate: validate_as::<Hold>,
};

/// `text` as a settings table.
fn table(text: &str) -> Table {
    text.parse().unwrap()
}

/// The hold section `file` gives, as its minimum and maximum.
fn read(file: Option<&Table>) -> (u32, u32) {
    let h: Hold = peek_section(file, HOLD, &SPEC).unwrap();
    (h.min_ms, h.max_ms)
}

#[test]
fn the_section_is_read_over_the_defaults_at_its_version() {
    assert_eq!(
        read(Some(&table("[hold]\nversion = 3\nmin_ms = 200"))),
        (200, 500)
    );
    assert_eq!(read(Some(&table("[hold]\nmin_ms = 200"))), (200, 500));
}

#[test]
fn one_bad_value_takes_its_default_and_the_others_stay() {
    let text = "[hold]\nversion = 3\nmin_ms = 200\nmax_ms = \"x\"";
    assert_eq!(read(Some(&table(text))), (200, 500));
}

#[test]
fn an_older_section_is_migrated_as_the_store_does() {
    let text = "[hold]\nversion = 1\ndelay_s = 1\nmax_ms = 2000";
    assert_eq!(read(Some(&table(text))), (1000, 2000));
}

#[test]
fn a_newer_section_gives_the_defaults() {
    assert_eq!(
        read(Some(&table("[hold]\nversion = 4\nmin_ms = 200"))),
        (100, 500)
    );
}

#[test]
fn no_section_or_one_that_does_not_read_gives_the_defaults() {
    let defaults = (100, 500);
    assert_eq!(read(None), defaults);
    for text in [
        "[other]\nmin_ms = 200",
        "[hold]\nmin_ms = 900",
        "[hold]\nmin_ms = \"x\"",
        "[hold]\nno_such = 1",
        "hold = 3",
    ] {
        assert_eq!(read(Some(&table(text))), defaults, "{text}");
    }
}

#[test]
fn an_inconsistent_spec_reads_no_section_and_gives_its_defaults() {
    assert!(!GAPPED.is_consistent());
    let file = table("[hold]\nversion = 3\nmin_ms = 200");
    let h: Hold = peek_section(Some(&file), HOLD, &GAPPED).unwrap();
    assert_eq!((h.min_ms, h.max_ms), (100, 500));
}

#[test]
fn defaults_that_do_not_read_are_an_error() {
    let file = table("[hold]\nmin_ms = 200");
    let read = peek_section::<Hold>(Some(&file), HOLD, &UNREADABLE);
    assert!(matches!(read, Err(SettingsError::Parse(_))));
}
