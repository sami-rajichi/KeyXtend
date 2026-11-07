//! The fill bound: how late a fill's part after a Tab may still go, within its range and at least
//! twice the relabel time.

use super::*;

/// The agreed bound, in ms.
const LATE_MS: u64 = 2_000;
const RANGE: (i64, i64) = (500, 20_000);

/// The defaults with the bound at `late` and the relabel time at `relabel`, both in ms.
fn both(late: i64, relabel: i64) -> toml::Table {
    let mut t = with("fill_late_ms", late);
    t.insert("relabel_ms".to_owned(), relabel.into());
    t
}

#[test]
fn a_fill_part_may_go_up_to_2_s_late_by_default() {
    let s = KeyboardSettings::from_defaults().unwrap();
    assert_eq!(s.fill_late_ms, LATE_MS);
    assert!(s.fill_late_ms >= LATE_FACTOR * s.relabel_ms);
}

#[test]
fn the_bound_is_refused_outside_its_range() {
    let (low, high) = RANGE;
    let late = |ms: i64| validate_as::<KeyboardSettings>(&both(ms, 50));
    assert_eq!((late(low), late(high)), (Ok(()), Ok(())));
    assert!(invalid(&both(low - 1, 50)), "just under");
    assert!(invalid(&both(high + 1, 50)), "just over");
    assert!(unparsed(&with("fill_late_ms", -1)), "below zero");
    let bound = |ms: &u64| i64::try_from(*ms).unwrap();
    let named = (bound(FILL_LATE_MS.start()), bound(FILL_LATE_MS.end()));
    assert_eq!(named, RANGE, "one range, named once");
}

#[test]
fn the_bound_is_at_least_twice_the_relabel_time_so_a_slow_beat_still_makes_it() {
    let ok =
        |late: i64, relabel: i64| validate_as::<KeyboardSettings>(&both(late, relabel)) == Ok(());
    assert_eq!(LATE_FACTOR, 2, "a coarse timer needs this margin");
    assert!(ok(2_000, 1_000), "the edge");
    assert!(ok(2_001, 1_000), "above");
    assert!(invalid(&both(1_999, 1_000)), "just under");
    assert!(invalid(&both(1_000, 1_000)), "equal");
    assert!(
        invalid(&both(5_000, 10_000)),
        "a slow relabel needs a later bound"
    );
    assert!(ok(20_000, 10_000), "the slowest relabel still has room");
}
