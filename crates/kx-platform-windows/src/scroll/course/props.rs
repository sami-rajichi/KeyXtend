//! Properties over random runs: carried steps never drift by a whole step, a target that moves once a notch never
//! trips, and one that ignores the wheel trips once `check_ms` and a whole notch have passed.

use super::*;
use crate::scroll::tests::{CONFIG, NOTCH, aim, at, wheel};
use crate::scroll::{FULL, PAGE_SCROLL};
use proptest::collection::vec;
use proptest::prelude::*;

/// Random runs per property.
const CASES: u32 = 512;
/// Most ticks in one run.
const MAX_TICKS: usize = 200;
/// Most lines one tick turns: the top speed, 60 lines a second, over a long tick.
const MAX_LINES: f64 = 2.0;
/// Floating sums round, so a drift is judged this much past a whole step.
const EPS: f64 = 1e-6;
/// The largest wheel setting tried, in lines a notch.
const MAX_SETTING: u32 = 100;
/// The largest notch tried: ten of Windows'.
const MAX_NOTCH: u32 = NOTCH * 10;
/// The `[scroll] tick_ms` range.
const TICK_MS: std::ops::RangeInclusive<u32> = 8..=100;
/// Where a moving percent starts.
const START: f64 = 50.0;
/// How far a percent moves a notch: `MAX_TICKS` notches at most leave it short of either end.
const NUDGE: f64 = 1.0 / 16.0;
/// Most percent reads in one walk; this many one-point steps from `START` stay short of either end.
const MAX_READS: usize = 40;

/// A wheel setting: mostly lines, sometimes none or a page.
fn setting() -> impl Strategy<Value = u32> {
    prop_oneof![8 => 1..=MAX_SETTING, 1 => Just(0), 1 => Just(PAGE_SCROLL)]
}

/// True when a wheel with `setting` glides: it scrolls some lines a notch, not none or a page.
fn glides(setting: u32) -> bool {
    setting != 0 && setting != PAGE_SCROLL
}

/// The steps one line takes with `setting`, worked out apart from the code: one UIA step without a wheel that glides.
fn unit(notch: u32, setting: u32) -> f64 {
    if glides(setting) {
        f64::from(notch) / f64::from(setting)
    } else {
        1.0
    }
}

/// The route a tick takes, worked out apart from the code.
fn want(setting: u32, on_pointer: bool) -> Route {
    match (glides(setting), on_pointer) {
        (false, _) => Route::Uia,
        (true, true) => Route::Send,
        (true, false) => Route::Post,
    }
}

/// UI Automation's pair with `pct` towards `dir`, as a read gives it.
#[allow(
    clippy::unnecessary_wraps,
    reason = "It stands for what a read returns."
)]
fn pair(dir: Dir, pct: f64) -> Option<(f64, f64)> {
    Some(if dir.sideways() {
        (pct, 0.0)
    } else {
        (0.0, pct)
    })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(CASES))]

    #[test]
    fn carried_steps_never_drift_by_a_whole_step(
        notch in 1..=MAX_NOTCH,
        setting in setting(),
        sideways in any::<bool>(),
        ticks in vec((0.0..=MAX_LINES, any::<bool>()), 0..=MAX_TICKS),
    ) {
        let wheel = Wheel { notch, lines: setting, chars: setting };
        let dir = if sideways { Dir::Left } else { Dir::Down };
        let mut course = Course::new(CONFIG);
        let (mut sum, mut lines) = (0_u32, 0.0);
        for (owed, on_pointer) in ticks {
            if let Some(turn) = course.turn(&aim(1, on_pointer), dir, owed, wheel) {
                prop_assert_eq!(turn.route, want(setting, on_pointer));
                prop_assert!(turn.count > 0);
                sum += u32::from(turn.count);
            }
            lines += owed;
        }
        let left = lines * unit(notch, setting) - f64::from(sum);
        prop_assert!((-EPS..1.0 + EPS).contains(&left), "{} left over", left);
    }

    #[test]
    fn a_target_that_moves_once_a_notch_never_trips(
        notch in 1..=MAX_NOTCH,
        setting in 1..=MAX_SETTING,
        sideways in any::<bool>(),
        tick in TICK_MS,
        owed in vec(0.0..=MAX_LINES, 1..=MAX_TICKS),
    ) {
        let wheel = Wheel { notch, lines: setting, chars: setting };
        let (dir, a) = (if sideways { Dir::Left } else { Dir::Down }, aim(1, false));
        let (mut c, mut now, mut sent) = (Course::new(CONFIG), 0, 0_u32);
        for lines in owed {
            let pct = START + f64::from(sent / notch) * NUDGE;
            prop_assert_eq!(c.read(&a, dir, at(now), pair(dir, pct)), Reach::More);
            if let Some(turn) = c.turn(&a, dir, lines, wheel) {
                sent += u32::from(turn.count);
            }
            now += tick;
        }
        prop_assert!(!c.tripped(), "it moved at every notch");
    }

    #[test]
    fn a_target_that_ignores_the_wheel_trips_once_check_ms_and_a_notch_passed(
        setting in 1..=MAX_SETTING,
        tick in TICK_MS,
        owed in vec(0.0..=MAX_LINES, 1..=MAX_TICKS),
        pct in 0.0..START,
    ) {
        let w = Wheel { notch: NOTCH, lines: setting, chars: setting };
        let a = aim(1, false);
        let (mut c, mut now, mut sent) = (Course::new(CONFIG), 0, 0_u32);
        for lines in owed {
            c.read(&a, Dir::Down, at(now), pair(Dir::Down, pct));
            let due = now >= CONFIG.check_ms && sent >= NOTCH;
            prop_assert_eq!(c.tripped(), due, "at {} ms after {} delta", now, sent);
            if let Some(turn) = c.turn(&a, Dir::Down, lines, w).filter(|t| t.route != Route::Uia) {
                sent += u32::from(turn.count);
            }
            now += tick;
        }
    }

    #[test]
    fn a_percent_that_moves_at_every_read_never_trips(
        gaps in vec(0..=CONFIG.check_ms, 1..=MAX_READS),
        walk in vec(any::<bool>(), MAX_READS),
    ) {
        let (mut c, a, w) = (Course::new(CONFIG), aim(1, false), wheel(1, 1));
        let (mut pct, mut now) = (START, 0);
        c.read(&a, Dir::Down, at(now), pair(Dir::Down, pct));
        for (gap, up) in gaps.into_iter().zip(walk) {
            c.turn(&a, Dir::Down, 1.0, w);
            (pct, now) = (if up { pct + 1.0 } else { pct - 1.0 }, now + gap);
            prop_assert!(pct > 0.0 && pct < FULL);
            c.read(&a, Dir::Down, at(now), pair(Dir::Down, pct));
        }
        prop_assert!(!c.tripped(), "moved either way at every read");
    }
}
