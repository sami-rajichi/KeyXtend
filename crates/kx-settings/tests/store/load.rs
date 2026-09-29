//! Loading: missing, bad, older and newer sections, per-value repair, kept entries and broken specs.

use crate::spec::{
    DEFAULTS, HOLD, Hold, SPEC, VERSION, defaults_with, folder, newer, open, reset, table,
    with_file,
};
use kx_module_api::{ModuleId, SettingsSpec, validate_as};
use kx_settings::Store;
use kx_test_support::golden::assert_golden;
use std::fs;
use std::path::PathBuf;

/// The golden file of the migrated sample file, from the crate folder.
const MIGRATED_GOLDEN: [&str; 3] = ["tests", "golden", "migrated.toml"];

/// A version 1 file with a top-level key and a section of a module that is not built in.
const V1_FILE: &str = "theme = \"dark\"\n\n[hold]\nversion = 1\ndelay_s = 2\nmax_ms = 5000\nsound = false\n\n[later]\ncolour = \"teal\"\n";

#[test]
fn a_missing_section_gives_the_defaults_and_no_notice() {
    let (_dir, files) = folder("missing");
    let (store, report) = open(&files);
    assert_eq!(store.get(HOLD), Some(&table(DEFAULTS)));
    assert!(report.notices.is_empty() && report.broken.is_empty());
    assert!(!store.needs_save());
    assert_eq!(store.to_text().unwrap(), "");
}

#[test]
fn one_bad_value_resets_alone_and_is_named() {
    let text = "[hold]\nversion = 3\nmax_ms = 900\nsound = \"loud\"\n";
    let (_dir, files) = with_file("onebad", text);
    let (store, report) = open(&files);
    assert_eq!(store.get(HOLD), Some(&defaults_with("max_ms = 900")));
    assert_eq!(report.notices, vec![reset("sound")]);
    assert!(store.needs_save());
    assert_eq!(
        store.to_text().unwrap(),
        "[hold]\nversion = 3\nmax_ms = 900\n"
    );
}

#[test]
fn values_valid_only_together_are_both_kept() {
    // Alone, each breaks a rule: 6000 is above the default maximum, and 9000 is too far above the default minimum.
    let (_dir, files) = with_file("together", "[hold]\nmin_ms = 6000\nmax_ms = 9000\n");
    let (store, report) = open(&files);
    assert_eq!(
        store.get(HOLD),
        Some(&defaults_with("min_ms = 6000\nmax_ms = 9000"))
    );
    assert!(report.notices.is_empty());
    assert!(!store.needs_save());
}

#[test]
fn a_pair_valid_only_together_survives_one_bad_value() {
    let text = "[hold]\nmin_ms = 6000\nmax_ms = 9000\nsound = \"loud\"\n";
    let (_dir, files) = with_file("pairtypo", text);
    let (mut store, report) = open(&files);
    let pair = defaults_with("min_ms = 6000\nmax_ms = 9000");
    assert_eq!(store.get(HOLD), Some(&pair));
    assert_eq!(report.notices, vec![reset("sound")]);
    store.save().unwrap();
    let saved = fs::read_to_string(&files.settings).unwrap();
    assert_eq!(saved, "[hold]\nversion = 3\nmax_ms = 9000\nmin_ms = 6000\n");
}

#[test]
fn a_value_that_fits_only_beside_another_survives_a_bad_value() {
    // `max_ms` is below the default minimum, so it fits only with `min_ms` lowered.
    let text = "[hold]\nmax_ms = 50\nmin_ms = 10\nsound = \"loud\"\n";
    let (_dir, files) = with_file("secondpass", text);
    let (store, report) = open(&files);
    assert_eq!(
        store.get(HOLD),
        Some(&defaults_with("min_ms = 10\nmax_ms = 50"))
    );
    assert_eq!(report.notices, vec![reset("sound")]);
    assert_eq!(
        store.to_text().unwrap(),
        "[hold]\nversion = 3\nmax_ms = 50\nmin_ms = 10\n"
    );
}

#[test]
fn a_value_that_fits_only_beside_another_survives_two_bad_values() {
    // The unknown key is set aside first, so leaving out `sound` keeps the pair.
    let text = "[hold]\nextra = 1\nmax_ms = 50\nmin_ms = 10\nsound = \"loud\"\n";
    let (_dir, files) = with_file("twobad", text);
    let (store, report) = open(&files);
    let pair = defaults_with("min_ms = 10\nmax_ms = 50");
    assert_eq!(store.get(HOLD), Some(&pair));
    assert_eq!(report.notices, vec![reset("extra, sound")]);
}

#[test]
fn several_bad_values_share_one_notice_in_key_order() {
    let text = "[hold]\nsound = 3\nmin_ms = -1\nextra = 1\nmax_ms = 700\n";
    let (_dir, files) = with_file("several", text);
    let (store, report) = open(&files);
    assert_eq!(store.get(HOLD), Some(&defaults_with("max_ms = 700")));
    assert_eq!(report.notices, vec![reset("extra, min_ms, sound")]);
}

#[test]
fn a_section_without_a_version_counts_as_current() {
    let (_dir, files) = with_file("noversion", "[hold]\nmax_ms = 900\n");
    let (store, report) = open(&files);
    assert_eq!(store.get(HOLD), Some(&defaults_with("max_ms = 900")));
    assert!(report.notices.is_empty());
    assert_eq!(
        store.to_text().unwrap(),
        "[hold]\nversion = 3\nmax_ms = 900\n"
    );
}

#[test]
fn a_section_that_is_not_a_table_is_a_bad_section() {
    let (_dir, files) = with_file("notable", "hold = 5\n");
    let (store, report) = open(&files);
    assert_eq!(store.get(HOLD), Some(&table(DEFAULTS)));
    assert_eq!(report.notices, vec![reset("hold")]);
    assert!(store.needs_save());
    assert_eq!(store.to_text().unwrap(), "");
}

#[test]
fn a_version_that_is_not_a_positive_integer_is_a_bad_section() {
    for version in ["0", "-2", "\"3\"", "1.5"] {
        let text = format!("[hold]\nversion = {version}\nsound = false\nmax_ms = 900\n");
        let (_dir, files) = with_file("badversion", &text);
        let (store, report) = open(&files);
        assert_eq!(store.get(HOLD), Some(&table(DEFAULTS)), "{version}");
        assert_eq!(report.notices, vec![reset("max_ms, sound")], "{version}");
        assert_eq!(store.to_text().unwrap(), "", "{version}");
    }
}

#[test]
fn a_bad_section_with_only_a_version_names_the_section() {
    let (_dir, files) = with_file("onlyversion", "[hold]\nversion = 0\n");
    let (_, report) = open(&files);
    assert_eq!(report.notices, vec![reset("hold")]);
}

#[test]
fn a_v1_section_runs_both_migrations_in_order() {
    let (_dir, files) = with_file("golden", V1_FILE);
    let (mut store, report) = open(&files);
    let expected = defaults_with("min_ms = 2000\nmax_ms = 5000\nsound = false");
    assert_eq!(store.get(HOLD), Some(&expected));
    assert!(report.notices.is_empty());
    store.save().unwrap();
    let saved = fs::read_to_string(&files.settings).unwrap();
    let golden: PathBuf = [env!("CARGO_MANIFEST_DIR")]
        .into_iter()
        .chain(MIGRATED_GOLDEN)
        .collect();
    assert_golden(golden, &saved);
}

#[test]
fn a_v2_section_runs_only_the_last_migration() {
    let text = "[hold]\nversion = 2\ndelay_ms = 300\ndelay_s = 1\n";
    let (_dir, files) = with_file("v2", text);
    let (store, report) = open(&files);
    assert_eq!(store.get(HOLD), Some(&defaults_with("min_ms = 300")));
    assert_eq!(report.notices, vec![reset("delay_s")]);
}

#[test]
fn a_failing_migration_gives_a_bad_section() {
    let text = "[hold]\nversion = 1\ndelay_s = \"soon\"\nmax_ms = 900\n";
    let (_dir, files) = with_file("migfail", text);
    let (store, report) = open(&files);
    assert_eq!(store.get(HOLD), Some(&table(DEFAULTS)));
    assert_eq!(report.notices, vec![reset("delay_s, max_ms")]);
    assert!(store.needs_save());
    assert_eq!(store.to_text().unwrap(), "");
}

#[test]
fn newer_unknown_and_top_level_entries_survive_word_for_word() {
    let text = "theme = \"dark\"\n\n[hold]\nfuture = [1, 2]\nmin_ms = 1\nversion = 9\n\n[later]\ncolour = \"teal\"\n\n[later.deep]\non = true\n";
    let (_dir, files) = with_file("kept", text);
    let (store, report) = open(&files);
    assert_eq!(store.get(HOLD), Some(&table(DEFAULTS)));
    assert_eq!(report.notices, vec![newer()]);
    assert!(!store.needs_save());
    assert_eq!(store.to_text().unwrap(), text);
}

#[test]
fn a_version_too_big_for_the_spec_counts_as_newer() {
    let text = "[hold]\nversion = 99999999999\n";
    let (_dir, files) = with_file("huge", text);
    let (store, report) = open(&files);
    assert_eq!(report.notices, vec![newer()]);
    assert_eq!(store.to_text().unwrap(), text);
}

#[test]
fn a_broken_spec_is_reported_and_the_others_load() {
    let broken = [
        ModuleId::new("a"),
        ModuleId::new("b"),
        ModuleId::new("c"),
        ModuleId::new("d"),
    ];
    let text = "[a]\nkeep = \"me\"\n\n[hold]\nmax_ms = 900\n";
    let (_dir, files) = with_file("broken", text);
    let specs = [
        (broken[0], &BAD_TEXT),
        (broken[1], &GAPS),
        (HOLD, &SPEC),
        (broken[2], &INVALID),
        (broken[3], &VERSIONED),
    ];
    let (store, report) = Store::load(files.clone(), &specs);
    assert_eq!(report.broken, broken);
    assert!(broken.iter().all(|id| store.get(*id).is_none()));
    assert_eq!(store.get(HOLD), Some(&defaults_with("max_ms = 900")));
    assert_eq!(
        store.to_text().unwrap(),
        "[a]\nkeep = \"me\"\n\n[hold]\nversion = 3\nmax_ms = 900\n"
    );
}

/// A sound version 1 spec of the sample settings, for building broken ones.
const HOLD_V1: SettingsSpec = SettingsSpec {
    version: 1,
    defaults: DEFAULTS,
    migrations: &[],
    validate: validate_as::<Hold>,
};

/// Broken: its defaults are not TOML.
static BAD_TEXT: SettingsSpec = SettingsSpec {
    defaults: "min_ms = = 1",
    ..HOLD_V1
};
/// Broken: version 3 with no migrations.
static GAPS: SettingsSpec = SettingsSpec {
    version: VERSION,
    ..HOLD_V1
};
/// Broken: its defaults fail the check.
static INVALID: SettingsSpec = SettingsSpec {
    defaults: "min_ms = 9\nmax_ms = 1\nsound = true",
    ..HOLD_V1
};
/// Broken: its defaults hold the version key.
static VERSIONED: SettingsSpec = SettingsSpec {
    defaults: "version = 1\nmin_ms = 1\nmax_ms = 2\nsound = true",
    ..HOLD_V1
};

#[test]
fn a_module_registered_twice_is_broken_the_second_time() {
    let (_dir, files) = with_file("twice", "[hold]\nmax_ms = 900\n");
    let (store, report) = Store::load(files.clone(), &[(HOLD, &SPEC), (HOLD, &SPEC)]);
    assert_eq!(report.broken, [HOLD]);
    assert_eq!(store.get(HOLD), Some(&defaults_with("max_ms = 900")));
    assert_eq!(
        store.to_text().unwrap(),
        "[hold]\nversion = 3\nmax_ms = 900\n"
    );
}
