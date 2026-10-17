//! Tests of `g6judge`, on fixtures of pictures' shares and percents; no input goes out.

use super::*;

/// A fixture `min_changed`; the shipped `[g6]` one may differ.
const MIN: f64 = 0.02;
/// A document that overflows both ways, near its top left.
const TOP: Pair = Some((0.0, 0.0));
/// One that UI Automation cannot scroll sideways (-1).
const NO_SIDEWAYS: Pair = Some((-1.0, 40.0));
/// One that UI Automation can scroll neither way (-1, -1).
const NO_SCROLL: Pair = Some((-1.0, -1.0));
/// A slope from two reads, as the old floor took.
const TWO: u64 = 2;

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
    assert!(!look(None, None, None).moved(Dir::Down, MIN), "no picture");
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
    assert!(!look(None, None, None).still(MIN), "no picture");
}

#[test]
fn still_falls_back_to_the_picture_when_no_axis_reads_a_percent() {
    assert!(
        !look(Some(0.3), NO_SCROLL, NO_SCROLL).still(MIN),
        "the picture moved"
    );
    assert!(look(Some(0.0), NO_SCROLL, NO_SCROLL).still(MIN));
    assert!(
        look(Some(0.3), NO_SIDEWAYS, NO_SIDEWAYS).still(MIN),
        "the vertical percent reads, and it stood"
    );
}

#[test]
fn the_end_needs_a_stop_by_itself_a_down_leg_that_moved_and_the_end_reached() {
    let down = look(Some(0.4), Some((0.0, 90.0)), Some((0.0, 100.0)));
    let back = look(Some(0.0), Some((0.0, 100.0)), Some((0.0, 100.0)));
    assert!(end_ok(true, &down, &back, MIN));
    assert!(!end_ok(false, &down, &back, MIN), "it never stopped itself");
    let flat = look(Some(0.0), Some((0.0, 90.0)), Some((0.0, 90.0)));
    assert!(!end_ok(true, &flat, &back, MIN), "the down leg never moved");
    let short = look(Some(0.2), Some((0.0, 100.0)), Some((0.0, 90.0)));
    assert!(
        !end_ok(true, &down, &short, MIN),
        "it stopped short of the end"
    );
}

#[test]
fn without_a_percent_the_end_is_the_picture_g6_saw_at_the_end() {
    let down = look(Some(0.4), None, None);
    assert!(end_ok(true, &down, &look(Some(0.0), None, None), MIN));
    assert!(
        !end_ok(true, &down, &look(Some(0.3), None, None), MIN),
        "the picture differs from the end's"
    );
    assert!(!end_ok(true, &down, &look(None, None, None), MIN));
}

#[test]
fn a_rate_is_the_slope_of_the_reads_short_of_the_ends_in_percent_a_second() {
    let steady = [(0, 10.0), (200, 11.0), (400, 12.0), (600, 13.0)];
    let r = rate(&steady, TWO).expect("a rate");
    assert!((r - 5.0).abs() < 1e-9, "{r}");
    let pinned = [(0, 0.0), (200, 0.0), (400, 2.0), (600, 4.0), (800, 100.0)];
    let r = rate(&pinned, TWO).expect("from the two inner reads");
    assert!((r - 10.0).abs() < 1e-9, "{r}");
    let up = [(0, 50.0), (500, 45.0)];
    assert!(rate(&up, TWO).expect("a rate") < 0.0, "upward reads fall");
    assert_eq!(rate(&[(0, 0.0), (200, 3.0)], TWO), None, "one inner read");
    assert_eq!(
        rate(&[(5, 3.0), (5, 4.0)], TWO),
        None,
        "no time between them"
    );
}

#[test]
fn a_rate_needs_the_reads_the_settings_ask_for() {
    let shipped = crate::config::load().expect("loads").g6.slope_reads;
    let four = [(0, 10.0), (200, 11.0), (400, 12.0), (600, 13.0)];
    assert!(rate(&four, 4).is_some());
    assert_eq!(rate(&four, 5), None, "one read short");
    assert!(shipped > TWO, "a slope from two reads is not stable");
}

#[test]
fn the_ratio_must_lie_near_the_test_builds_within_the_slack() {
    let want = 10.0 / 3.0;
    assert!(ratio_ok(3.0, 10.0, want, 0.3));
    assert!(ratio_ok(-3.0, -12.0, want, 0.3), "upward rates");
    assert!(!ratio_ok(3.0, 3.0, want, 0.3), "never went fast");
    assert!(!ratio_ok(3.0, 20.0, want, 0.3));
    assert!(!ratio_ok(0.0, 10.0, want, 0.3), "slow never moved");
    assert!(!ratio_ok(3.0, -10.0, want, 0.3), "fast ran the other way");
    assert!(!ratio_ok(-3.0, 10.0, want, 0.3), "slow ran the other way");
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
fn each_outside_click_and_each_stop_tap_stops_it_once_and_the_end_stops_it_too() {
    let line = ScrollLine {
        press: 1,
        end: 1,
        user: 3,
        ..ScrollLine::default()
    };
    let made = Stops {
        clicks: 1,
        taps: 3,
        ends: 1,
    };
    assert!(stops_ok(&line, made));
    let as_press = Stops { clicks: 0, ..made };
    assert!(
        !stops_ok(&line, as_press),
        "a tap on the pad stopped it as a press"
    );
    let missed = Stops { taps: 2, ..made };
    assert!(!stops_ok(&line, missed), "a stop tap was not counted once");
    let paged = ScrollLine { user: 4, ..line };
    assert!(
        !stops_ok(&paged, made),
        "a page turn of our list, which G6 never aims at"
    );
    let other = [
        ScrollLine { end: 0, ..line },
        ScrollLine { limit: 1, ..line },
        ScrollLine { gone: 1, ..line },
    ];
    for l in other {
        assert!(!stops_ok(&l, made), "{l:?}");
    }
}

#[test]
fn with_no_run_to_the_end_no_end_stop_is_needed() {
    let line = ScrollLine {
        user: 2,
        ..ScrollLine::default()
    };
    let unjudged = Stops {
        clicks: 0,
        taps: 2,
        ends: 0,
    };
    assert!(stops_ok(&line, unjudged), "the end case was left unjudged");
    let ran = Stops {
        ends: 1,
        ..unjudged
    };
    assert!(!stops_ok(&line, ran), "it ran, so it must stop at the end");
}

#[test]
fn an_unjudged_case_fails_nothing_and_a_failed_one_fails_the_app() {
    assert!(pass(&[Some(true), None, Some(true)]));
    assert!(!pass(&[Some(true), Some(false), None]));
    assert!(pass(&[]));
}
