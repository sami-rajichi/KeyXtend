//! The `[vault]` settings: the agreed defaults, every range's ends, the tops the worker caps, the check days, the
//! pairs, the limits the notebook takes and the vault job's config.

use super::*;
use kx_module_api::ports::{
    MAX_BLOB_BYTES, MAX_BOX_BYTES, MAX_BOX_WORK, MAX_FIELD_CHARS, MAX_VAULT_ENTRIES,
    MAX_VAULT_FIELDS, MB, MIN_BOX_WORK,
};

/// The defaults as a table.
fn defaults() -> toml::Table {
    SPEC.defaults.parse().unwrap()
}

/// The defaults with `key` set to `value`.
fn with(key: &str, value: impl Into<toml::Value>) -> toml::Table {
    set(defaults(), key, value)
}

/// `t` with `key` set to `value`.
fn set(mut t: toml::Table, key: &str, value: impl Into<toml::Value>) -> toml::Table {
    t.insert(key.to_owned(), value.into());
    t
}

/// The defaults with the passwords at their shortest, so each range's ends stand alone of the pairs.
fn loose() -> toml::Table {
    let (make, box_min) = (ends(&MAKE_CHARS).0, ends(&BOX_MIN_CHARS).0);
    set(
        set(defaults(), "make_chars", make),
        "box_min_chars",
        box_min,
    )
}

/// True when the table is refused by a rule that names `key`.
fn refused(t: &toml::Table, key: &str) -> bool {
    matches!(validate_as::<VaultSettings>(t), Err(SettingsError::Invalid(why)) if why.contains(key))
}

/// True when the table is taken.
fn taken(t: &toml::Table) -> bool {
    validate_as::<VaultSettings>(t).is_ok()
}

/// The ends of `range`, widened for the table.
fn ends<T: Copy + TryInto<i64>>(range: &RangeInclusive<T>) -> (i64, i64) {
    let wide = |n: T| n.try_into().unwrap_or_else(|_| panic!("fits an i64"));
    (wide(*range.start()), wide(*range.end()))
}

/// Each whole-number key and its range's ends.
fn bounds() -> Vec<(&'static str, (i64, i64))> {
    vec![
        ("field_chars", ends(&FIELD_CHARS)),
        ("max_entries", ends(&MAX_ENTRIES)),
        ("max_fields", ends(&MAX_FIELDS)),
        ("make_chars", ends(&MAKE_CHARS)),
        ("box_delay_ms", ends(&BOX_DELAY_MS)),
        ("box_min_chars", ends(&BOX_MIN_CHARS)),
        ("check_tries", ends(&CHECK_TRIES)),
        ("tab_gap_ms", ends(&TAB_GAP_MS)),
        ("box_mb", ends(&BOX_MB)),
        ("box_work", ends(&BOX_WORK)),
        ("vault_ms", ends(&VAULT_MS)),
        ("worker_idle_s", ends(&WORKER_IDLE_S)),
        ("restart_ms", ends(&RESTART_MS)),
        ("restart_max_ms", ends(&RESTART_MAX_MS)),
        ("paste_wait_ms", ends(&PASTE_WAIT_MS)),
        ("place_check_s", ends(&PLACE_CHECK_S)),
        ("later_h", ends(&LATER_H)),
    ]
}

#[test]
fn the_defaults_validate_and_hold_the_agreed_values() {
    assert!(SPEC.is_consistent());
    let s = VaultSettings::from_defaults().unwrap();
    assert_eq!(
        (s.field_chars, s.max_entries, s.max_fields),
        (1000, 2000, 30)
    );
    assert_eq!(
        (s.make_chars, s.box_delay_ms, s.box_min_chars),
        (20, 2000, 8)
    );
    assert_eq!(s.check_days, [1, 3, 7, 14, 30]);
    assert_eq!((s.check_tries, s.memory_checks, s.later_h), (5, true, 24));
    assert_eq!((s.tab_gap_ms, s.box_mb, s.box_work), (150, 16, 18));
    assert_eq!(
        (s.vault_ms, s.paste_wait_ms, s.place_check_s),
        (20_000, 3000, 60)
    );
    assert_eq!(
        (s.worker_idle_s, s.restart_ms, s.restart_max_ms),
        (30, 1000, 30_000)
    );
}

#[test]
fn each_range_takes_its_ends_and_refuses_past_them() {
    for (key, (lo, hi)) in bounds() {
        assert!(taken(&set(loose(), key, lo)), "{key} = {lo}");
        assert!(taken(&set(loose(), key, hi)), "{key} = {hi}");
        assert!(
            refused(&set(loose(), key, lo - 1), key),
            "{key} = {}",
            lo - 1
        );
        assert!(
            refused(&set(loose(), key, hi + 1), key),
            "{key} = {}",
            hi + 1
        );
    }
}

#[test]
fn the_plans_ranges_hold() {
    assert_eq!(
        (FIELD_CHARS, MAX_ENTRIES, MAX_FIELDS),
        (8..=4000, 1..=10_000, 1..=100)
    );
    assert_eq!(
        (MAKE_CHARS, BOX_DELAY_MS, BOX_MIN_CHARS),
        (8..=64, 250..=60_000, 8..=64)
    );
    assert_eq!(
        (CHECK_TRIES, TAB_GAP_MS, BOX_MB),
        (1..=10, 20..=2000, 1..=64)
    );
    assert_eq!((BOX_WORK, VAULT_MS), (16..=20, 2000..=120_000));
    assert_eq!(
        (WORKER_IDLE_S, RESTART_MS, RESTART_MAX_MS),
        (5..=3600, 100..=30_000, 1000..=600_000)
    );
    assert_eq!((PASTE_WAIT_MS, PLACE_CHECK_S), (2500..=10_000, 10..=3600));
    assert_eq!(LATER_H, 1..=168);
}

/// The worker refuses past its caps, so each top is the cap itself and a change there needs no edit here.
#[test]
fn the_tops_are_the_vault_workers_caps() {
    assert_eq!(*FIELD_CHARS.end(), MAX_FIELD_CHARS);
    assert_eq!(*MAX_ENTRIES.end(), MAX_VAULT_ENTRIES);
    assert_eq!(*MAX_FIELDS.end(), MAX_VAULT_FIELDS);
    let box_bytes = u64::from(*BOX_MB.end()) * MB;
    assert_eq!(box_bytes, u64::try_from(MAX_BOX_BYTES).unwrap());
    assert_eq!(BOX_WORK, MIN_BOX_WORK..=MAX_BOX_WORK);
}

#[test]
fn the_check_days_rise_within_their_range_and_count() {
    let days = |d: &[i64]| toml::Value::Array(d.iter().map(|&n| n.into()).collect());
    let (lo, hi) = ends(&CHECK_DAY);
    assert!(taken(&with("check_days", days(&[lo]))));
    assert!(taken(&with("check_days", days(&[lo, hi]))));
    for bad in [vec![], vec![lo - 1], vec![hi + 1], vec![3, 3], vec![7, 3]] {
        assert!(
            refused(&with("check_days", days(&bad)), "check_days"),
            "{bad:?}"
        );
    }
    let most = i64::try_from(*CHECK_STEPS.end()).unwrap();
    assert!(taken(&with(
        "check_days",
        days(&(1..=most).collect::<Vec<_>>())
    )));
    let many: Vec<i64> = (1..=most + 1).collect();
    assert!(refused(&with("check_days", days(&many)), "check_days"));
}

#[test]
fn an_unknown_key_is_refused() {
    let t = with("max_items", 5);
    assert!(matches!(
        validate_as::<VaultSettings>(&t),
        Err(SettingsError::Parse(_))
    ));
}

#[test]
fn the_notebook_takes_its_limits_and_the_blob_cap() {
    let limits = VaultSettings::from_defaults().unwrap().limits();
    assert_eq!(
        (limits.field_chars, limits.max_entries, limits.max_fields),
        (1000, 2000, 30)
    );
    assert_eq!(limits.max_bytes, MAX_BLOB_BYTES);
}

#[test]
fn the_limits_the_tests_share_are_the_defaults() {
    let limits = VaultSettings::from_defaults().unwrap().limits();
    assert_eq!(crate::fixture::LIMITS, limits);
}

#[test]
fn a_field_holds_a_made_password_and_a_backup_password() {
    let at = |key: &str, n: i64, field: i64| set(with(key, n), "field_chars", field);
    for (key, why) in [("make_chars", BAD_MAKE), ("box_min_chars", BAD_BOX_MIN)] {
        assert!(taken(&at(key, 20, 21)), "{key} below");
        assert!(taken(&at(key, 20, 20)), "{key} equal");
        assert!(refused(&at(key, 21, 20), why), "{key} above");
    }
    assert!(
        FIELD_CHARS.start() >= MAKE_CHARS.start() && FIELD_CHARS.start() >= BOX_MIN_CHARS.start()
    );
}

#[test]
fn the_first_restart_wait_never_passes_the_longest() {
    let at = |first: i64, most: i64| set(with("restart_ms", first), "restart_max_ms", most);
    assert!(taken(&at(1000, 2000)));
    assert!(taken(&at(2000, 2000)));
    assert!(refused(&at(2001, 2000), BAD_RESTART));
}

#[test]
fn the_vault_job_runs_with_the_worker_settings() {
    let s = VaultSettings::from_defaults().unwrap();
    let cfg = s.box_config();
    assert_eq!(cfg.deadline_ms, s.vault_ms);
    assert_eq!(cfg.idle_s, s.worker_idle_s);
    assert_eq!(
        (cfg.restart_ms, cfg.restart_max_ms),
        (s.restart_ms, s.restart_max_ms)
    );
}

#[test]
fn the_waits_are_their_settings_in_ms() {
    let s = VaultSettings::from_defaults().unwrap();
    let ms = |n: u64| std::time::Duration::from_millis(n);
    assert_eq!((s.tab_gap(), s.paste_wait()), (ms(150), ms(3000)));
}

#[test]
fn the_backup_waits_and_cap_are_their_settings() {
    let s = VaultSettings::from_defaults().unwrap();
    let ms = std::time::Duration::from_millis;
    assert_eq!((s.box_delay(), s.place_check()), (ms(2000), ms(60_000)));
    assert_eq!(s.box_cap(), 16 * MB);
}

#[test]
fn the_memory_checks_waits_are_their_settings_in_hours_and_days() {
    let s = VaultSettings::from_defaults().unwrap();
    let hours = |n: u64| std::time::Duration::from_secs(n * 3600);
    assert_eq!(s.later(), hours(24));
    assert_eq!(
        s.check_waits(),
        [1, 3, 7, 14, 30].map(|d| hours(d * 24)).to_vec()
    );
}

#[test]
fn the_memory_checks_key_names_its_setting() {
    assert!(defaults().contains_key(MEMORY_CHECKS));
    let off: VaultSettings = kx_module_api::parse_as(&with(MEMORY_CHECKS, false)).unwrap();
    assert!(!off.memory_checks);
}
