//! The `[clipboard]` settings: the agreed defaults, every range's ends, the checked lists and pairs, and the configs
//! the parts take.

use super::*;
use std::time::Duration;

use kx_module_api::ports::{MAX_BLOB_BYTES, MB};

/// The defaults as a table.
fn defaults() -> toml::Table {
    SPEC.defaults.parse().unwrap()
}

/// The defaults with `key` set to `value`.
fn with(key: &str, value: impl Into<toml::Value>) -> toml::Table {
    let mut t = defaults();
    t.insert(key.to_owned(), value.into());
    t
}

/// True when the table is refused by a rule that names `key`.
fn refused(t: &toml::Table, key: &str) -> bool {
    matches!(validate_as::<ClipSettings>(t), Err(SettingsError::Invalid(why)) if why.contains(key))
}

/// True when the table is taken.
fn taken(t: &toml::Table) -> bool {
    validate_as::<ClipSettings>(t).is_ok()
}

/// The ends of `range`, widened for the table.
fn ends<T: Copy + TryInto<i64>>(range: &RangeInclusive<T>) -> (i64, i64) {
    let wide = |n: T| n.try_into().unwrap_or_else(|_| panic!("fits an i64"));
    (wide(*range.start()), wide(*range.end()))
}

/// Each whole-number key and its range's ends.
fn bounds() -> Vec<(&'static str, (i64, i64))> {
    vec![
        ("keep_days", ends(&KEEP_DAYS)),
        ("media_days", ends(&KEEP_DAYS)),
        ("max_entries", ends(&MAX_ENTRIES)),
        ("max_mb", ends(&MAX_MB)),
        ("min_free_mb", ends(&MIN_FREE_MB)),
        ("max_entry_mb", ends(&MAX_ENTRY_MB)),
        ("clear_follow_s", ends(&CLEAR_FOLLOW_S)),
        ("join_breaks", ends(&JOIN_BREAKS)),
        ("paste_gap_ms", ends(&PASTE_GAP_MS)),
        ("paste_wait_ms", ends(&PASTE_WAIT_MS)),
        ("search_chars", ends(&SEARCH_CHARS)),
        ("title_chars", ends(&TITLE_CHARS)),
        ("preview_chars", ends(&PREVIEW_CHARS)),
        ("thumb_px", ends(&THUMB_PX)),
        ("preview_px", ends(&PREVIEW_PX)),
        ("max_pixels", ends(&MAX_PIXELS)),
        ("media_ms", ends(&MEDIA_MS)),
        ("worker_idle_s", ends(&WORKER_IDLE_S)),
        ("worker_mb", ends(&WORKER_MB)),
        ("restart_ms", ends(&RESTART_MS)),
        ("restart_max_ms", ends(&RESTART_MAX_MS)),
        ("open_ms", ends(&OPEN_MS)),
        ("poll_ms", ends(&POLL_MS)),
        ("save_ms", ends(&SAVE_MS)),
    ]
}

#[test]
fn the_defaults_validate_and_hold_the_agreed_values() {
    assert!(SPEC.is_consistent());
    let s = ClipSettings::from_defaults().unwrap();
    assert_eq!((s.keep_days, s.media_days, s.max_entries), (7, 1, 1000));
    assert_eq!((s.max_mb, s.min_free_mb, s.max_entry_mb), (1024, 2048, 32));
    assert_eq!(s.clear_follow_s, 120);
    assert!(s.skip_apps.is_empty() && s.join_rule.is_empty() && s.history_key);
    assert_eq!(
        (s.join_breaks, s.paste_gap_ms, s.paste_wait_ms),
        (2, 150, 1000)
    );
    assert_eq!(
        (s.search_chars, s.title_chars, s.preview_chars),
        (1000, 300, 20_000)
    );
    assert_eq!(
        (s.thumb_px, s.preview_px, s.max_pixels),
        (128, 960, 100_000_000)
    );
    assert_eq!((s.media_ms, s.worker_idle_s, s.worker_mb), (3000, 30, 512));
    assert_eq!((s.restart_ms, s.restart_max_ms), (1000, 30_000));
    assert_eq!((s.open_ms, s.poll_ms, s.save_ms), (250, 10, 500));
}

#[test]
fn each_range_takes_its_ends_and_refuses_past_them() {
    for (key, (lo, hi)) in bounds() {
        assert!(taken(&with(key, lo)), "{key} = {lo}");
        assert!(taken(&with(key, hi)), "{key} = {hi}");
        assert!(refused(&with(key, lo - 1), key), "{key} = {}", lo - 1);
        assert!(refused(&with(key, hi + 1), key), "{key} = {}", hi + 1);
    }
}

#[test]
fn the_plans_ranges_and_the_safe_lower_bounds_hold() {
    assert_eq!(MAX_ENTRIES, 1..=5000);
    assert_eq!(
        (MAX_MB, MIN_FREE_MB, MAX_ENTRY_MB),
        (64..=16_384, 256..=65_536, 1..=64)
    );
    assert_eq!(
        (KEEP_DAYS, CLEAR_FOLLOW_S, JOIN_BREAKS),
        (1..=7, 10..=600, 1..=3)
    );
    assert_eq!((PASTE_GAP_MS, PASTE_WAIT_MS), (20..=2000, 100..=5000));
    assert!(*RESTART_MS.start() >= 100 && *WORKER_MB.start() >= 64);
    assert!(*MEDIA_MS.start() >= 500 && *WORKER_IDLE_S.start() >= 5);
}

#[test]
fn a_join_rule_takes_80_characters_on_one_line() {
    let arabic = "ـ".repeat(JOIN_RULE_CHARS);
    assert!(taken(&with("join_rule", arabic.as_str())));
    assert!(refused(
        &with("join_rule", format!("{arabic}ـ")),
        "join_rule"
    ));
    assert!(refused(&with("join_rule", "--\n--"), "join_rule"));
    assert!(taken(&with("join_rule", "———")));
}

#[test]
fn skipped_apps_are_named_and_counted() {
    let app = |id: &str| toml::Value::Array(vec![id.into()]);
    assert!(taken(&with("skip_apps", app("keepass.exe"))));
    assert!(refused(&with("skip_apps", app("")), "skip_apps"));
    assert!(refused(&with("skip_apps", app("a\tb")), "skip_apps"));
    let long = "a".repeat(*APP_CHARS.end() + 1);
    assert!(refused(&with("skip_apps", app(&long)), "skip_apps"));
    let many: Vec<toml::Value> = (0..=*SKIP_APPS.end())
        .map(|n| format!("app{n}").into())
        .collect();
    assert!(refused(&with("skip_apps", many), "skip_apps"));
}

#[test]
fn the_waits_and_the_restarts_keep_their_order() {
    let mut t = with("poll_ms", 100);
    t.insert("open_ms".to_owned(), 50.into());
    assert!(refused(&t, "poll_ms"));
    let mut t = with("restart_ms", 20_000);
    t.insert("restart_max_ms".to_owned(), 10_000.into());
    assert!(refused(&t, "restart_ms"));
}

#[test]
fn an_unknown_key_is_refused() {
    let t = with("max_items", 5);
    assert!(matches!(
        validate_as::<ClipSettings>(&t),
        Err(SettingsError::Parse(_))
    ));
}

#[test]
fn the_parts_get_the_settings_in_their_units() {
    let s = ClipSettings::from_defaults().unwrap();
    let limits = s.limits();
    assert_eq!(
        (limits.keep_days, limits.media_days, limits.max_entries),
        (7, 1, 1000)
    );
    assert_eq!(limits.max_bytes, 1024 * MB);
    assert_eq!(
        (limits.min_free_bytes, limits.max_entry_bytes),
        (2048 * MB, 32 * MB)
    );
    assert_eq!(s.entry_config().title_chars, 300);
    assert_eq!(s.entry_config().search_chars, 1000);
    assert_eq!(s.store_config().thumb_px, 128);
    assert_eq!(s.store_config().blob_bytes, MAX_BLOB_BYTES);
    assert_eq!(s.panel_config().query_chars, 1000);
    let clip = s.clip_config();
    assert_eq!((clip.max_bytes, clip.wait), (32 * MB, s.wait()));
    assert_eq!((s.wait().open_ms, s.wait().poll_ms), (250, 10));
    let pics = s.picture_config();
    assert_eq!(
        (pics.deadline_ms, pics.idle_s, pics.memory_mb),
        (3000, 30, 512)
    );
    assert_eq!(
        (pics.max_pixels, pics.restart_ms, pics.restart_max_ms),
        (100_000_000, 1000, 30_000)
    );
    assert_eq!(s.join_config().breaks, 2);
    assert_eq!(s.save(), Duration::from_millis(500));
    assert_eq!(s.paste_gap(), Duration::from_millis(150));
    assert_eq!(s.paste_wait(), Duration::from_millis(1000));
}

#[test]
fn the_rows_wait_for_the_ui() {
    let s = ClipSettings::from_defaults().unwrap();
    assert_eq!(s.panel_config().rows, 0, "no page size of its own");
}

#[test]
fn the_unicode_line_and_paragraph_breaks_are_refused_too() {
    for mark in ['\u{2028}', '\u{2029}'] {
        let broken = format!("--{mark}--");
        assert!(refused(&with("join_rule", broken.as_str()), "join_rule"));
        let app = toml::Value::Array(vec![format!("a{mark}b").into()]);
        assert!(refused(&with("skip_apps", app), "skip_apps"));
    }
}
