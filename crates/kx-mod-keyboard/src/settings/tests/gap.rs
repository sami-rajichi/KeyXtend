//! The erase gap: the pause after each Backspace of a picked word, within its range.

use super::*;

/// The owner's pick (2026-10-26): Notepad took every key with a pause of 2 ms or more after each.
const GAP_MS: u64 = 10;
const RANGE: (i64, i64) = (0, 50);

#[test]
fn a_picked_word_pauses_10_ms_after_each_backspace_by_default() {
    let s = KeyboardSettings::from_defaults().unwrap();
    assert_eq!(s.erase_gap_ms, GAP_MS);
}

#[test]
fn the_gap_is_refused_outside_its_range() {
    let (low, high) = RANGE;
    let gap = |ms: i64| validate_as::<KeyboardSettings>(&with("erase_gap_ms", ms));
    assert_eq!((gap(low), gap(high)), (Ok(()), Ok(())), "0 waits nothing");
    assert!(invalid(&with("erase_gap_ms", high + 1)), "just over");
    assert!(
        invalid(&with("erase_gap_ms", i64::MAX)),
        "a hand-edited huge value"
    );
    assert!(unparsed(&with("erase_gap_ms", -1)), "below zero");
    let bound = |ms: &u64| i64::try_from(*ms).unwrap();
    let named = (bound(ERASE_GAP_MS.start()), bound(ERASE_GAP_MS.end()));
    assert_eq!(named, RANGE, "one range, named once");
}
