//! The fallback to UI Automation's steps and the end they cannot pass: a percent still for `check_ms` short of the end
//! trips the target after a notch of wheel delta, and reads as the end after a UI Automation step. Only steps sent
//! before the previous read count, so a target that shows them late is never judged early.

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
/// The `[scroll] check_ms` default.
const CHECK: u32 = CONFIG.check_ms;
/// The `[scroll] tick_ms` default.
const TICK: u32 = 16;
/// The shortest `[scroll]` times: `check_ms` and `read_ms` at their minimum, 100.
const FASTEST: ScrollConfig = ScrollConfig {
    check_ms: 100,
    read_ms: 100,
    edge_px: 8,
};

/// A course on target 1 that tripped: a notch sent before a still read, judged at the next.
fn tripped() -> Course {
    let (mut c, a, w) = (Course::new(CONFIG), aim(1, false), wheel(THREE, THREE));
    c.read(&a, Dir::Down, at(0), HALF);
    c.turn(&a, Dir::Down, NOTCH_LINES, w);
    c.read(&a, Dir::Down, at(CHECK), HALF);
    assert!(!c.tripped(), "the notch has not had a whole read to show");
    c.read(&a, Dir::Down, at(CHECK * 2), HALF);
    assert!(c.tripped());
    c
}

#[test]
fn the_fallback_waits_for_check_ms_and_a_whole_notch() {
    let (mut c, a, w) = (Course::new(CONFIG), aim(1, false), wheel(THREE, THREE));
    assert_eq!(c.read(&a, Dir::Down, at(0), HALF), Reach::More);
    c.turn(&a, Dir::Down, THIRD, w);
    c.turn(&a, Dir::Down, THIRD, w);
    c.read(&a, Dir::Down, at(CHECK), HALF);
    c.turn(&a, Dir::Down, THIRD, w);
    assert_eq!(c.read(&a, Dir::Down, at(CHECK * 2), HALF), Reach::More);
    assert!(
        !c.tripped(),
        "two thirds of a notch before the previous read"
    );
    c.read(&a, Dir::Down, at(CHECK * 3), HALF);
    assert!(c.tripped(), "a whole notch and check_ms");
    let on = c.turn(&aim(1, true), Dir::Down, 1.0, w).map(|t| t.route);
    assert_eq!(on, Some(Route::Uia), "never the wheel again");
}

#[test]
fn a_notch_sent_early_still_waits_for_check_ms() {
    let (mut c, a, w) = (Course::new(CONFIG), aim(1, false), wheel(THREE, THREE));
    c.read(&a, Dir::Down, at(0), HALF);
    c.turn(&a, Dir::Down, NOTCH_LINES, w);
    c.read(&a, Dir::Down, at(CHECK / 2), HALF);
    c.read(&a, Dir::Down, at(CHECK - 1), HALF);
    assert!(!c.tripped(), "still for less than check_ms");
    c.read(&a, Dir::Down, at(CHECK), HALF);
    assert!(c.tripped());
}

#[test]
fn a_notch_one_tick_before_the_read_waits_for_the_next_read() {
    let (mut c, a, w) = (Course::new(CONFIG), aim(1, false), wheel(THREE, THREE));
    c.read(&a, Dir::Down, at(0), HALF);
    c.read(&a, Dir::Down, at(CHECK - TICK), HALF);
    c.turn(&a, Dir::Down, NOTCH_LINES, w);
    c.read(&a, Dir::Down, at(CHECK), HALF);
    assert!(!c.tripped(), "a late target has not shown it yet");
    c.read(&a, Dir::Down, at(CHECK * 2), HALF);
    assert!(c.tripped());
}

#[test]
fn a_step_one_tick_before_the_read_waits_for_the_next_read() {
    let (mut c, a, w) = (
        Course::new(CONFIG),
        aim(1, false),
        wheel(PAGE_SCROLL, THREE),
    );
    c.read(&a, Dir::Down, at(0), HALF);
    c.read(&a, Dir::Down, at(CHECK - TICK), HALF);
    c.turn(&a, Dir::Down, 1.0, w);
    let late = c.read(&a, Dir::Down, at(CHECK), HALF);
    assert_eq!(late, Reach::More, "a late target has not shown it yet");
    assert_eq!(c.read(&a, Dir::Down, at(CHECK * 2), HALF), Reach::End);
}

#[test]
fn steps_sent_before_the_percent_last_moved_never_count() {
    let (mut c, a, w) = (Course::new(CONFIG), aim(1, false), wheel(THREE, THREE));
    let moved = Some((0.0, 60.0));
    c.read(&a, Dir::Down, at(0), HALF);
    c.turn(&a, Dir::Down, NOTCH_LINES, w);
    c.read(&a, Dir::Down, at(CHECK), moved);
    c.read(&a, Dir::Down, at(CHECK * 2), moved);
    assert!(!c.tripped(), "that notch moved it");
}

#[test]
fn a_pause_between_reads_starts_the_wait_afresh() {
    let (mut c, a, w) = (Course::new(CONFIG), aim(1, false), wheel(THREE, THREE));
    c.read(&a, Dir::Down, at(0), HALF);
    c.turn(&a, Dir::Down, NOTCH_LINES, w);
    c.read(&a, Dir::Down, at(CHECK), HALF);
    c.read(&a, Dir::Down, at(PAUSE_MS), HALF);
    assert!(!c.tripped(), "a read a minute later");
    c.turn(&a, Dir::Down, NOTCH_LINES, w);
    c.read(&a, Dir::Down, at(PAUSE_MS + CHECK), HALF);
    c.read(&a, Dir::Down, at(PAUSE_MS + CHECK * 2), HALF);
    assert!(c.tripped(), "the wait started again at the late read");
}

#[test]
fn the_shortest_settings_still_judge_a_normal_gap_between_reads() {
    let (mut c, a, w) = (Course::new(FASTEST), aim(1, false), wheel(THREE, THREE));
    let gap = FASTEST.check_ms * 2 + FASTEST.read_ms;
    for i in 0..3 {
        c.turn(&a, Dir::Down, NOTCH_LINES, w);
        c.read(&a, Dir::Down, at(i * gap), HALF);
    }
    assert!(c.tripped(), "a check, a late tick and a slow read apart");
    let mut c = Course::new(FASTEST);
    for i in 0..3 {
        c.turn(&a, Dir::Down, NOTCH_LINES, w);
        c.read(&a, Dir::Down, at(i * (gap + 1)), HALF);
    }
    assert!(!c.tripped(), "a longer gap is a pause");
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
            let read = c.read(&a, Dir::Down, at(i * CHECK), Some((0.0, p)));
            assert_eq!(read, reach);
        }
        assert!(!c.tripped(), "{percents:?}");
    }
    let (mut c, a) = (Course::new(CONFIG), aim(1, false));
    for i in 0..4 {
        c.turn(&a, Dir::Down, NOTCH_LINES, w);
        assert_eq!(c.read(&a, Dir::Down, at(i * CHECK), None), Reach::Unknown);
    }
    assert!(!c.tripped(), "no percent");
}

#[test]
fn still_reads_without_wheel_steps_never_trip() {
    let a = aim(1, false);
    let mut c = Course::new(CONFIG);
    for i in 0..4 {
        c.read(&a, Dir::Down, at(i * CHECK), HALF);
    }
    assert!(!c.tripped(), "a hung window got no steps");
    let mut c = Course::new(CONFIG);
    for i in 0..4 {
        c.turn(&a, Dir::Down, NOTCH_LINES, wheel(PAGE_SCROLL, THREE));
        c.read(&a, Dir::Down, at(i * CHECK), HALF);
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
    let start = CHECK * 3;
    c.turn(&a, Dir::Down, 1.0, w);
    assert_eq!(c.read(&a, Dir::Down, at(start), HALF), Reach::More);
    c.turn(&a, Dir::Down, 1.0, w);
    c.read(&a, Dir::Down, at(start + CHECK / 2), HALF);
    let early = c.read(&a, Dir::Down, at(start + CHECK - 1), HALF);
    assert_eq!(early, Reach::More, "still for less than check_ms");
    let end = c.read(&a, Dir::Down, at(start + CHECK), HALF);
    assert_eq!(end, Reach::End, "the steps cannot pass it");
    let moved = Some((0.0, 60.0));
    let after = c.read(&a, Dir::Down, at(start + CHECK * 2), moved);
    assert_eq!(after, Reach::More, "it moved after all");
}

#[test]
fn a_percent_a_rounding_away_from_its_end_is_at_it() {
    let (mut c, a) = (Course::new(CONFIG), aim(1, false));
    let near = Some((0.0, FULL - END_SLACK / 2.0));
    assert_eq!(c.read(&a, Dir::Down, at(0), near), Reach::End);
    let near = Some((0.0, END_SLACK / 2.0));
    assert_eq!(c.read(&a, Dir::Up, at(0), near), Reach::End);
}

#[test]
fn the_next_route_is_the_one_the_turn_takes() {
    let (c, w) = (Course::new(CONFIG), wheel(THREE, THREE));
    assert_eq!(c.next_route(&aim(1, true), Dir::Down, w), Route::Send);
    assert_eq!(c.next_route(&aim(1, false), Dir::Down, w), Route::Post);
    let no_lines = wheel(0, THREE);
    assert_eq!(c.next_route(&aim(1, true), Dir::Down, no_lines), Route::Uia);
    assert_eq!(
        c.next_route(&aim(1, false), Dir::Left, no_lines),
        Route::Post,
        "sideways reads the characters"
    );
    let t = tripped();
    assert_eq!(t.next_route(&aim(1, true), Dir::Down, w), Route::Uia);
    assert_eq!(
        t.next_route(&aim(1, false), Dir::Up, w),
        Route::Uia,
        "a new way keeps it"
    );
    assert_eq!(
        t.next_route(&aim(2, true), Dir::Down, w),
        Route::Send,
        "a new target starts afresh"
    );
}
