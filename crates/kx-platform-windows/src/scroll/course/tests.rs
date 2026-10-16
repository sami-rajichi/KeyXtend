//! The fallback to UI Automation's steps: a percent still for `check_ms` while a whole notch went out, short of the end,
//! trips the target until it changes; a percent still under those steps reads as the end.

use super::*;
use crate::scroll::tests::{CONFIG, THREE, aim, at, wheel};
use crate::scroll::{END_SLACK, FULL, PAGE_SCROLL};

/// A percent half way.
const HALF: Option<(f64, f64)> = Some((0.0, 50.0));
/// A third of a notch at Windows' default wheel.
const THIRD: f64 = 1.0;
/// A whole notch at Windows' default wheel.
const NOTCH_LINES: f64 = 3.0;
/// A pause between reads, as when the pad stays still for a minute.
const PAUSE_MS: u32 = 60_000;

/// A course on target 1 that tripped after a still read under a notch of wheel steps.
fn tripped() -> Course {
    let (mut c, a, w) = (Course::new(CONFIG), aim(1, false), wheel(THREE, THREE));
    c.read(&a, Dir::Down, at(0), HALF);
    c.turn(&a, Dir::Down, NOTCH_LINES, w);
    c.read(&a, Dir::Down, at(CONFIG.check_ms), HALF);
    assert!(c.tripped());
    c
}

#[test]
fn the_fallback_waits_for_check_ms_and_a_whole_notch() {
    let (mut c, a, w) = (Course::new(CONFIG), aim(1, false), wheel(THREE, THREE));
    assert_eq!(c.read(&a, Dir::Down, at(0), HALF), Reach::More);
    c.turn(&a, Dir::Down, THIRD, w);
    c.turn(&a, Dir::Down, THIRD, w);
    assert_eq!(
        c.read(&a, Dir::Down, at(CONFIG.check_ms), HALF),
        Reach::More
    );
    assert!(!c.tripped(), "two thirds of a notch");
    c.turn(&a, Dir::Down, THIRD, w);
    c.read(&a, Dir::Down, at(CONFIG.check_ms + 1), HALF);
    assert!(c.tripped(), "a whole notch and check_ms");
    let on = c.turn(&aim(1, true), Dir::Down, 1.0, w).map(|t| t.route);
    assert_eq!(on, Some(Route::Uia), "never the wheel again");
}

#[test]
fn a_notch_sent_early_still_waits_for_check_ms() {
    let (mut c, a, w) = (Course::new(CONFIG), aim(1, false), wheel(THREE, THREE));
    c.read(&a, Dir::Down, at(0), HALF);
    c.turn(&a, Dir::Down, NOTCH_LINES, w);
    c.read(&a, Dir::Down, at(CONFIG.check_ms - 1), HALF);
    assert!(!c.tripped(), "still for less than check_ms");
    c.read(&a, Dir::Down, at(CONFIG.check_ms), HALF);
    assert!(c.tripped());
}

#[test]
fn a_pause_between_reads_starts_the_wait_afresh() {
    let (mut c, a, w) = (Course::new(CONFIG), aim(1, false), wheel(THREE, THREE));
    c.read(&a, Dir::Down, at(0), HALF);
    c.turn(&a, Dir::Down, NOTCH_LINES, w);
    c.read(&a, Dir::Down, at(PAUSE_MS), HALF);
    assert!(!c.tripped(), "a read a minute later");
    c.turn(&a, Dir::Down, NOTCH_LINES, w);
    c.read(&a, Dir::Down, at(PAUSE_MS + CONFIG.check_ms), HALF);
    assert!(c.tripped(), "the wait started again at the late read");
}

#[test]
fn a_moving_percent_or_one_at_the_end_or_unknown_never_trips() {
    let w = wheel(THREE, THREE);
    let runs: [(&[f64], Reach); 4] = [
        (&[10.0, 20.0, 30.0, 40.0], Reach::More),
        (&[40.0, 30.0, 20.0, 10.0], Reach::More),
        (&[FULL, FULL, FULL, FULL], Reach::End),
        (&[-1.0, -1.0, -1.0, -1.0], Reach::Unknown),
    ];
    for (percents, reach) in runs {
        let (mut c, a) = (Course::new(CONFIG), aim(1, false));
        for (i, &p) in (0..).zip(percents) {
            c.turn(&a, Dir::Down, NOTCH_LINES, w);
            let read = c.read(&a, Dir::Down, at(i * CONFIG.check_ms), Some((0.0, p)));
            assert_eq!(read, reach);
        }
        assert!(!c.tripped(), "{percents:?}");
    }
    let (mut c, a) = (Course::new(CONFIG), aim(1, false));
    for i in 0..4 {
        c.turn(&a, Dir::Down, NOTCH_LINES, w);
        let read = c.read(&a, Dir::Down, at(i * CONFIG.check_ms), None);
        assert_eq!(read, Reach::Unknown);
    }
    assert!(!c.tripped(), "no percent");
}

#[test]
fn still_reads_without_wheel_steps_never_trip() {
    let a = aim(1, false);
    let mut c = Course::new(CONFIG);
    for i in 0..4 {
        c.read(&a, Dir::Down, at(i * CONFIG.check_ms), HALF);
    }
    assert!(!c.tripped(), "a hung window got no steps");
    let mut c = Course::new(CONFIG);
    for i in 0..4 {
        c.turn(&a, Dir::Down, NOTCH_LINES, wheel(PAGE_SCROLL, THREE));
        c.read(&a, Dir::Down, at(i * CONFIG.check_ms), HALF);
    }
    assert!(!c.tripped(), "UI Automation's steps are no wheel steps");
}

#[test]
fn a_tripped_target_keeps_ui_automation_until_the_target_changes() {
    let mut c = tripped();
    let w = wheel(THREE, THREE);
    let up = c.turn(&aim(1, false), Dir::Up, 1.0, w).map(|t| t.route);
    assert_eq!(up, Some(Route::Uia), "a new way keeps it");
    let other = c.turn(&aim(2, false), Dir::Up, 1.0, w).map(|t| t.route);
    assert_eq!(other, Some(Route::Post), "a new target starts afresh");
    assert!(!c.tripped());
}

#[test]
fn a_percent_still_under_ui_automation_steps_reads_as_the_end() {
    let (mut c, a, w) = (tripped(), aim(1, false), wheel(THREE, THREE));
    let start = CONFIG.check_ms * 2;
    c.turn(&a, Dir::Down, 1.0, w);
    assert_eq!(c.read(&a, Dir::Down, at(start), HALF), Reach::More);
    c.turn(&a, Dir::Down, 1.0, w);
    let early = c.read(&a, Dir::Down, at(start + CONFIG.check_ms - 1), HALF);
    assert_eq!(early, Reach::More, "still for less than check_ms");
    c.turn(&a, Dir::Down, 1.0, w);
    let end = c.read(&a, Dir::Down, at(start + CONFIG.check_ms), HALF);
    assert_eq!(end, Reach::End, "the steps cannot pass it");
    let moved = c.read(
        &a,
        Dir::Down,
        at(start + CONFIG.check_ms * 2),
        Some((0.0, 60.0)),
    );
    assert_eq!(moved, Reach::More, "it moved after all");
}

#[test]
fn a_percent_a_rounding_away_from_its_end_is_at_it() {
    let (mut c, a) = (Course::new(CONFIG), aim(1, false));
    let near = FULL - END_SLACK / 2.0;
    assert_eq!(c.read(&a, Dir::Down, at(0), Some((0.0, near))), Reach::End);
    assert_eq!(
        c.read(&a, Dir::Up, at(0), Some((0.0, END_SLACK / 2.0))),
        Reach::End
    );
}
