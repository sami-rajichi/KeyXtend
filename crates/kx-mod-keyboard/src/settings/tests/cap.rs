//! The text cap: the most characters a picked word may erase or type, within its range.

use super::*;

/// The agreed cap and range, in characters (P7 plan, 2026-10-24; the floor rose to fit `[predict] max_word`).
const CAP: u16 = 256;
const RANGE: (i64, i64) = (128, 1_024);

#[test]
fn a_picked_word_may_erase_or_type_256_characters_by_default() {
    assert_eq!(KeyboardSettings::from_defaults().unwrap().text_cap, CAP);
}

#[test]
fn the_cap_is_refused_outside_its_range() {
    let (low, high) = RANGE;
    let cap = |n: i64| validate_as::<KeyboardSettings>(&with("text_cap", n));
    assert_eq!((cap(low), cap(high)), (Ok(()), Ok(())));
    assert!(invalid(&with("text_cap", low - 1)), "just under");
    assert!(invalid(&with("text_cap", high + 1)), "just over");
    assert!(invalid(&with("text_cap", 0)));
    assert!(unparsed(&with("text_cap", -1)), "below zero");
    let named = (i64::from(*TEXT_CAP.start()), i64::from(*TEXT_CAP.end()));
    assert_eq!(named, RANGE, "one range, named once");
}
