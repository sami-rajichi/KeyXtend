//! Tests of `g6judge`, on fixtures of pictures' shares and percents; no input goes out.

use super::*;

/// The shipped `[g6] min_changed`.
const MIN: f64 = 0.02;
/// A document that overflows both ways, near its top left.
const TOP: Pair = Some((0.0, 0.0));
/// One that UI Automation cannot scroll sideways (-1).
const NO_SIDEWAYS: Pair = Some((-1.0, 40.0));

fn look(share: Option<f64>, before: Pair, after: Pair) -> Look {
    Look {
        share,
        before,
        after,
    }
}

#[test]
fn a_percent_that_reads_judges_the_way_and_a_picture_judges_without_one() {
    let down = look(Some(0.5), Some((0.0, 10.0)), Some((0.0, 14.0)));
    assert!(down.moved(Dir::Down, MIN));
    assert!(!down.moved(Dir::Up, MIN), "the percent says it went down");
    let unread = look(Some(0.5), None, None);
    assert!(unread.moved(Dir::Down, MIN) && unread.moved(Dir::Up, MIN));
    assert!(!look(Some(0.01), None, None).moved(Dir::Down, MIN), "noise");
    assert!(
        !look(None, None, None).moved(Dir::Down, MIN),
        "the window moved"
    );
}

#[test]
fn sideways_is_judged_by_the_horizontal_percent() {
    let right = look(Some(0.3), Some((5.0, 40.0)), Some((9.0, 40.0)));
    assert!(right.moved(Dir::Right, MIN) && !right.moved(Dir::Left, MIN));
    assert!(
        !right.moved(Dir::Down, MIN),
        "the vertical percent stood still"
    );
    let flat = look(Some(0.3), NO_SIDEWAYS, NO_SIDEWAYS);
    assert!(
        flat.moved(Dir::Right, MIN),
        "no sideways percent, so the picture"
    );
}

#[test]
fn still_reads_the_percents_and_the_picture_only_without_them() {
    assert!(look(Some(0.0), TOP, TOP).still(MIN));
    assert!(
        look(Some(0.3), TOP, TOP).still(MIN),
        "a tip over the picture"
    );
    assert!(!look(Some(0.0), Some((0.0, 10.0)), Some((0.0, 10.5))).still(MIN));
    assert!(
        !look(Some(0.0), Some((3.0, 10.0)), Some((2.0, 10.0))).still(MIN),
        "it went left"
    );
    assert!(look(Some(MIN), None, None).still(MIN), "a blinking caret");
    assert!(!look(Some(0.3), None, None).still(MIN));
    assert!(!look(None, None, None).still(MIN), "the window moved");
}

#[test]
fn a_rate_is_the_slope_of_the_reads_short_of_the_ends_in_percent_a_second() {
    let steady = [(0, 10.0), (200, 11.0), (400, 12.0), (600, 13.0)];
    let r = rate(&steady).expect("a rate");
    assert!((r - 5.0).abs() < 1e-9, "{r}");
    let pinned = [(0, 0.0), (200, 0.0), (400, 2.0), (600, 4.0), (800, 100.0)];
    let r = rate(&pinned).expect("from the two inner reads");
    assert!((r - 10.0).abs() < 1e-9, "{r}");
    let up = [(0, 50.0), (500, 45.0)];
    assert!(rate(&up).expect("a rate") < 0.0, "upward reads fall");
    assert_eq!(rate(&[(0, 0.0), (200, 3.0)]), None, "one inner read");
    assert_eq!(rate(&[(5, 3.0), (5, 4.0)]), None, "no time between them");
}

#[test]
fn the_ratio_must_lie_near_the_test_builds_within_the_slack() {
    let want = 10.0 / 3.0;
    assert!(ratio_ok(3.0, 10.0, want, 0.3));
    assert!(ratio_ok(-3.0, -12.0, want, 0.3), "upward rates");
    assert!(!ratio_ok(3.0, 3.0, want, 0.3), "never went fast");
    assert!(!ratio_ok(3.0, 20.0, want, 0.3));
    assert!(!ratio_ok(0.0, 10.0, want, 0.3), "slow never moved");
}

#[test]
fn sideways_cases_run_only_where_the_document_overflows_sideways() {
    assert!(sideways(TOP));
    assert!(!sideways(NO_SIDEWAYS));
    assert!(!sideways(None));
}

#[test]
fn an_end_reads_at_its_own_way_only() {
    assert!(ended(Dir::Down, Some((0.0, 100.0))));
    assert!(ended(Dir::Up, TOP) && ended(Dir::Left, TOP));
    assert!(!ended(Dir::Down, Some((0.0, 99.5))), "short of it");
    assert!(!ended(Dir::Down, None) && !ended(Dir::Right, NO_SIDEWAYS));
}

#[test]
fn each_outside_click_stops_it_once_and_the_end_stops_it_too() {
    let line = ScrollLine {
        press: 1,
        end: 1,
        user: 3,
        ..ScrollLine::default()
    };
    assert!(stops_ok(&line, 1));
    assert!(
        !stops_ok(&line, 0),
        "a tap on the pad stopped it as a press"
    );
    let no_end = ScrollLine { end: 0, ..line };
    assert!(!stops_ok(&no_end, 1));
}

#[test]
fn an_unjudged_case_fails_nothing_and_a_failed_one_fails_the_app() {
    assert!(pass(&[Some(true), None, Some(true)]));
    assert!(!pass(&[Some(true), Some(false), None]));
    assert!(pass(&[]));
}
