//! Key repeat's settings: on by default with the owner's calm times, off on request, and each time within its range.

use super::*;
use kx_module_api::parse_as;

/// The owner's calm defaults (2026-10-11): the first repeat after 0.8 s, then 10 a second.
const DELAY_MS: u64 = 800;
const EVERY_MS: u64 = 100;
/// The ranges the owner approved, in ms.
const DELAY_RANGE: (i64, i64) = (300, 3_000);
const EVERY_RANGE: (i64, i64) = (33, 1_000);

#[test]
fn repeat_is_on_by_default_with_the_owners_calm_times() {
    let s = KeyboardSettings::from_defaults().unwrap();
    assert!(s.repeat);
    assert_eq!((s.repeat_delay_ms, s.repeat_every_ms), (DELAY_MS, EVERY_MS));
}

#[test]
fn repeat_can_be_turned_off() {
    let s: KeyboardSettings = parse_as(&with("repeat", false)).unwrap();
    assert!(!s.repeat);
    assert!(unparsed(&with("repeat", "no")), "only true or false");
}

#[test]
fn each_time_is_refused_outside_its_range() {
    let times = [
        ("repeat_delay_ms", DELAY_RANGE),
        ("repeat_every_ms", EVERY_RANGE),
    ];
    for (key, (low, high)) in times {
        let time = |ms: i64| validate_as::<KeyboardSettings>(&with(key, ms));
        assert_eq!((time(low), time(high)), (Ok(()), Ok(())), "{key}");
        assert!(invalid(&with(key, low - 1)), "{key} just under");
        assert!(invalid(&with(key, high + 1)), "{key} just over");
        assert!(invalid(&with(key, 0)), "{key} = 0");
        assert!(
            invalid(&with(key, i64::MAX)),
            "{key}: a hand-edited huge value"
        );
        assert!(unparsed(&with(key, -1)), "{key} below zero");
    }
}

#[test]
fn a_refused_time_names_its_range() {
    let Err(SettingsError::Invalid(why)) =
        validate_as::<KeyboardSettings>(&with("repeat_every_ms", 1))
    else {
        panic!("refused");
    };
    let (low, high) = EVERY_RANGE;
    assert_eq!(why, format!("repeat_every_ms must be from {low} to {high}"));
}
