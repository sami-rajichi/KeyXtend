use super::*;
use kx_module_api::within;

/// Each bounded key with no partner rule at its ends, and the range it takes.
const BOUNDS: [(&str, RangeInclusive<u32>); 5] = [
    ("limit_s", LIMIT_S),
    ("tick_ms", TICK_MS),
    ("check_ms", CHECK_MS),
    ("read_ms", READ_MS),
    ("edge_px", EDGE_PX),
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

/// The defaults with each key set to its number.
fn with_all(pairs: &[(&str, u32)]) -> toml::Table {
    let mut t = defaults();
    for &(key, value) in pairs {
        t.insert(key.to_owned(), i64::from(value).into());
    }
    t
}

fn check(t: &toml::Table) -> Result<(), SettingsError> {
    validate_as::<ScrollSettings>(t)
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
    let want = ScrollSettings {
        slow: 3,
        fast: 10,
        limit_s: 120,
        stop_at_end: true,
        stop_on_press: true,
        tick_ms: 16,
        check_ms: 250,
        read_ms: 300,
        edge_px: 8,
    };
    assert_eq!(ScrollSettings::from_defaults(), Ok(want));
}

#[test]
fn each_bound_takes_its_ends_and_refuses_a_value_just_outside() {
    for (key, range) in BOUNDS {
        let (low, top) = (*range.start(), *range.end());
        assert_eq!(check(&with_all(&[(key, low)])), Ok(()), "{key} = {low}");
        assert_eq!(check(&with_all(&[(key, top)])), Ok(()), "{key} = {top}");
        let (under, over) = (low - 1, top + 1);
        assert!(refused(&with_all(&[(key, over)]), key), "{key} = {over}");
        assert!(refused(&with_all(&[(key, under)]), key), "{key} = {under}");
    }
}

#[test]
fn the_ranges_are_the_agreed_ones() {
    assert_eq!(SPEED, 1..=60);
    assert_eq!((NO_LIMIT, LIMIT_S), (0, 10..=1_800));
    assert_eq!((TICK_MS, CHECK_MS), (8..=100, 100..=2_000));
    assert_eq!((READ_MS, EDGE_PX), (100..=2_000, 2..=64));
}

#[test]
fn both_speeds_take_their_range_and_fast_stays_above_slow() {
    let (low, top) = (*SPEED.start(), *SPEED.end());
    for slow in low - 1..=top + 1 {
        for fast in low - 1..=top + 1 {
            let ok = SPEED.contains(&slow) && SPEED.contains(&fast) && fast > slow;
            let t = with_all(&[("slow", slow), ("fast", fast)]);
            assert_eq!(check(&t).is_ok(), ok, "slow = {slow}, fast = {fast}");
        }
    }
}

#[test]
fn a_speed_outside_its_range_names_the_range_before_the_pair_rule() {
    for key in ["slow", "fast"] {
        for value in [*SPEED.start() - 1, *SPEED.end() + 1] {
            let want = within(key, value, &SPEED);
            assert_eq!(check(&with_all(&[(key, value)])), want, "{key} = {value}");
        }
    }
}

#[test]
fn a_fast_speed_at_or_below_slow_says_why() {
    let why = Err(SettingsError::Invalid(FAST_SLOW.to_owned()));
    let equal = with_all(&[("slow", 10), ("fast", 10)]);
    assert_eq!(check(&equal), why, "equal");
    let below = with_all(&[("slow", 10), ("fast", 3)]);
    assert_eq!(check(&below), why, "below");
}

#[test]
fn zero_means_no_limit_and_any_other_limit_stays_within_its_range() {
    for limit in NO_LIMIT..=*LIMIT_S.end() + 1 {
        let t = with_all(&[("limit_s", limit)]);
        if limit == NO_LIMIT || LIMIT_S.contains(&limit) {
            assert_eq!(check(&t), Ok(()), "limit_s = {limit}");
        } else {
            assert!(refused(&t, "limit_s"), "limit_s = {limit}");
        }
    }
}

#[test]
fn a_check_spans_more_than_one_tick() {
    let both = *CHECK_MS.start();
    assert!(
        TICK_MS.contains(&both),
        "the ranges meet, so the rule bites"
    );
    let why = Err(SettingsError::Invalid(TICK_LONG.to_owned()));
    let equal = with_all(&[("tick_ms", both), ("check_ms", both)]);
    assert_eq!(check(&equal), why);
    let below = with_all(&[("tick_ms", both - 1), ("check_ms", both)]);
    assert_eq!(check(&below), Ok(()));
}

#[test]
fn either_stop_can_be_switched_off() {
    let mut t = with("stop_at_end", false);
    t.insert("stop_on_press".to_owned(), false.into());
    assert_eq!(check(&t), Ok(()));
}

#[test]
fn the_scroll_config_carries_the_three_platform_values() {
    let mut s = ScrollSettings::from_defaults().unwrap();
    (s.check_ms, s.read_ms, s.edge_px) = (500, 700, 20);
    let want = ScrollConfig {
        check_ms: 500,
        read_ms: 700,
        edge_px: 20,
    };
    assert_eq!(s.scroll_config(), want);
}

#[test]
fn a_missing_unknown_or_mistyped_key_does_not_parse() {
    let mut missing = defaults();
    missing.remove("slow");
    assert!(unparsed(&missing), "slow has no default in code");
    assert!(unparsed(&with("speed", 3)), "an unknown key");
    assert!(unparsed(&with("slow", 2.5)), "speeds count whole lines");
    assert!(unparsed(&with("limit_s", -1)), "a limit is never negative");
    assert!(unparsed(&with("stop_at_end", 1)), "a stop is on or off");
}
