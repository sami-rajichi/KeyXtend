//! A run's pace over a rising clock: a start a minute old no longer counts, a full minute waits only until its oldest
//! start is a minute old, and a start booked ahead counts too. No 60 s window ever holds more than `per_min` starts.

use crate::net::pace::{MINUTE_MS, Pace};
use proptest::prelude::*;

/// A short while within a minute, in ms.
const STEP_MS: u64 = 10_000;

#[test]
fn a_start_a_minute_old_no_longer_counts() {
    let mut pace = Pace::new(1);
    assert_eq!(pace.turn(0), 0);
    assert_eq!(pace.turn(MINUTE_MS - 1), 1, "still within the minute");
    assert_eq!(pace.turn(3 * MINUTE_MS), 0, "both starts are gone");
}

#[test]
fn a_full_minute_waits_only_until_its_oldest_start_is_a_minute_old() {
    let mut pace = Pace::new(2);
    assert_eq!(pace.turn(0), 0);
    assert_eq!(pace.turn(STEP_MS), 0);
    assert_eq!(pace.turn(2 * STEP_MS), MINUTE_MS - 2 * STEP_MS);
}

#[test]
fn a_start_booked_ahead_counts_and_the_next_goes_after_it() {
    let mut pace = Pace::new(2);
    pace.turn(0);
    pace.turn(STEP_MS);
    let booked = pace.turn(2 * STEP_MS) + 2 * STEP_MS;
    let next = pace.turn(3 * STEP_MS) + 3 * STEP_MS;
    assert_eq!(booked, MINUTE_MS);
    assert_eq!(next, STEP_MS + MINUTE_MS, "a minute after the second start");
}

proptest! {
    /// For any `per_min` and any order of clock readings, the starts never go back, no minute holds more than
    /// `per_min` of them, and a turn with room in its minute and nothing booked ahead never waits.
    #[test]
    fn no_minute_holds_more_than_per_min_starts_and_room_never_waits(
        per_min in 1_usize..5,
        nows in prop::collection::vec(0..4 * MINUTE_MS, 1..40),
    ) {
        let mut pace = Pace::new(per_min);
        let mut starts: Vec<u64> = Vec::new();
        for now in nows {
            let in_minute = starts.iter().filter(|&&s| s + MINUTE_MS > now).count();
            let ahead = starts.last().is_some_and(|&s| s > now);
            let at = now + pace.turn(now);
            if in_minute < per_min && !ahead {
                prop_assert_eq!(at, now, "room never waits");
            }
            prop_assert!(starts.last().is_none_or(|&s| s <= at), "starts never go back");
            starts.push(at);
        }
        for pair in starts.windows(per_min + 1) {
            prop_assert!(pair[per_min] >= pair[0] + MINUTE_MS, "{starts:?}");
        }
    }
}
