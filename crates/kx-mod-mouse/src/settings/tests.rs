use super::*;

/// Each bounded key and the range it takes.
const BOUNDS: [(&str, RangeInclusive<u32>); 6] = [
    ("hold_ms", HOLD_MS),
    ("still_px", STILL_PX),
    ("ring_after_ms", RING_AFTER_MS),
    ("health_ms", HEALTH_MS),
    ("health_slack_ms", HEALTH_SLACK_MS),
    ("echo_ms", ECHO_MS),
];

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

fn check(t: &toml::Table) -> Result<(), SettingsError> {
    validate_as::<MouseSettings>(t)
}

/// The table is refused by a rule that names `key`.
fn refused(t: &toml::Table, key: &str) -> bool {
    matches!(check(t), Err(SettingsError::Invalid(why)) if why.contains(key))
}

fn unparsed(t: &toml::Table) -> bool {
    matches!(check(t), Err(SettingsError::Parse(_)))
}

#[test]
fn the_defaults_validate_and_hold_the_agreed_values() {
    assert!(SPEC.is_consistent());
    assert_eq!((SPEC.validate)(&defaults()), Ok(()));
    let want = MouseSettings {
        rclick: false,
        hold_ms: 1500,
        still_px: 7,
        ring_after_ms: 150,
        health_ms: 1000,
        health_slack_ms: 500,
        echo_ms: 250,
    };
    assert_eq!(MouseSettings::from_defaults(), Ok(want));
}

#[test]
fn the_hold_spans_half_a_second_to_three_as_the_spec_says() {
    assert_eq!(HOLD_MS, 500..=3_000);
}

#[test]
fn each_bound_takes_its_ends_and_refuses_a_value_just_outside() {
    for (key, range) in BOUNDS {
        let (low, top) = (i64::from(*range.start()), i64::from(*range.end()));
        assert_eq!(check(&with(key, low)), Ok(()), "{key} = {low}");
        assert_eq!(check(&with(key, top)), Ok(()), "{key} = {top}");
        assert!(refused(&with(key, top + 1), key), "{key} = {}", top + 1);
        let under = with(key, low - 1);
        let out = if low == 0 {
            unparsed(&under)
        } else {
            refused(&under, key)
        };
        assert!(out, "{key} = {}", low - 1);
    }
}

#[test]
fn the_ring_shows_before_the_shortest_hold_can_fire() {
    let shortest = i64::from(*HOLD_MS.start());
    let late = Err(SettingsError::Invalid(RING_LATE.to_owned()));
    assert_eq!(check(&with("ring_after_ms", shortest)), late, "equal");
    assert_eq!(check(&with("ring_after_ms", shortest + 1)), late, "above");
    assert!(
        RING_AFTER_MS.end() < HOLD_MS.start(),
        "every ring its range takes shows before the hold"
    );
}

#[test]
fn the_hook_config_carries_the_three_host_values() {
    let mut s = MouseSettings::from_defaults().unwrap();
    (s.health_ms, s.health_slack_ms, s.echo_ms) = (2_000, 300, 40);
    let want = HookConfig {
        health_ms: 2_000,
        health_slack_ms: 300,
        echo_ms: 40,
    };
    assert_eq!(s.hook_config(), want);
}

#[test]
fn a_missing_or_unknown_key_does_not_parse() {
    let mut missing = defaults();
    missing.remove("hold_ms");
    assert!(unparsed(&missing), "hold_ms has no default in code");
    assert!(unparsed(&with("grab", true)), "Grab is never remembered");
    assert!(unparsed(&with("rclick", 1)), "a mode is on or off");
}

#[test]
fn the_key_the_module_writes_is_right_click_mode() {
    assert_eq!(
        defaults().get(RCLICK_KEY),
        Some(&toml::Value::Boolean(false))
    );
    assert!(check(&with(RCLICK_KEY, true)).is_ok());
}
