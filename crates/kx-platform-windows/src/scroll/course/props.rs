//! Properties over random runs: carried steps never drift by a whole step, a target that moves at each notch or step
//! is never judged still, even when it shows them a read late, one that stays still is judged once `check_ms` passed
//! and its steps had a whole read to show, and the steps give the wheel back only when they end where it stopped.

use super::*;
use crate::scroll::tests::{CONFIG, NOTCH, THREE, aim, at, wheel};
use crate::scroll::{FULL, PAGE_SCROLL};
use proptest::collection::vec;
use proptest::prelude::*;

/// Random runs per property.
const CASES: u32 = 512;
/// Most ticks in one run.
const MAX_TICKS: usize = 200;
/// The most lines a steady tick turns: the top speed, 60 lines a second, over the longest tick, 100 ms; a tick after
/// a gap can owe more.
const MAX_LINES: f64 = 6.0;
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
/// How far a percent moves a notch or a step: the most a run sends leaves it short of either end.
const NUDGE: f64 = 1.0 / 64.0;
/// Most percent reads in one run; a walk of this many one-point steps from `START` stays short of either end.
const MAX_READS: usize = 40;
/// Most ticks between two reads, so some reads come more than `check_ms` apart.
const MAX_TICKS_PER_READ: usize = 4;

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
        let wheel = Wheel { notch, lines: setting, chars: setting, at_pointer: true };
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
    fn a_target_that_moves_at_each_notch_or_step_is_never_judged_still(
        notch in 1..=MAX_NOTCH,
        setting in setting(),
        sideways in any::<bool>(),
        late in any::<bool>(),
        tick in TICK_MS,
        reads in vec(vec(0.0..=MAX_LINES, 1..=MAX_TICKS_PER_READ), 1..=MAX_READS),
    ) {
        let wheel = Wheel { notch, lines: setting, chars: setting, at_pointer: true };
        let need = if glides(setting) { notch } else { 1 };
        let (dir, a) = (if sideways { Dir::Left } else { Dir::Down }, aim(1, false));
        let (mut c, mut now, mut sent, mut prev) = (Course::new(CONFIG), 0, 0_u32, 0_u32);
        for ticks in reads {
            // A late target shows a step only at the read after the one that followed it.
            let shown = if late { prev } else { sent };
            let pct = START + f64::from(shown / need) * NUDGE;
            prop_assert_eq!(c.read(&a, dir, at(now), pair(dir, pct)), Reach::More);
            prev = sent;
            for &lines in &ticks {
                if let Some(turn) = c.turn(&a, dir, lines, wheel) {
                    sent += u32::from(turn.count);
                }
            }
            now += tick * u32::try_from(ticks.len()).unwrap();
        }
        prop_assert!(!c.tripped(), "it moved at every notch");
    }

    #[test]
    fn a_still_target_is_judged_once_check_ms_passed_and_its_steps_had_a_read_to_show(
        setting in setting(),
        tick in TICK_MS,
        owed in vec(0.0..=MAX_LINES, 1..=MAX_TICKS),
        pct in 0.0..START,
    ) {
        let w = Wheel { notch: NOTCH, lines: setting, chars: setting, at_pointer: true };
        let need = if glides(setting) { NOTCH } else { 1 };
        let a = aim(1, false);
        let (mut c, mut now, mut sent, mut prev) = (Course::new(CONFIG), 0, 0_u32, 0_u32);
        for lines in owed {
            let due = now >= CONFIG.check_ms && prev >= need;
            let reach = c.read(&a, Dir::Down, at(now), pair(Dir::Down, pct));
            let judged = if glides(setting) { c.tripped() } else { reach == Reach::End };
            prop_assert_eq!(judged, due, "at {} ms after {} sent", now, prev);
            if judged && glides(setting) {
                // The steps that follow end at this percent and give the wheel back, a property of its own.
                break;
            }
            prev = sent;
            let ours = |t: &Turn| !glides(setting) || t.route != Route::Uia;
            if let Some(turn) = c.turn(&a, Dir::Down, lines, w).filter(ours) {
                sent += u32::from(turn.count);
            }
            now += tick;
        }
    }

    #[test]
    fn the_next_route_is_always_the_route_the_turn_takes(
        setting in setting(),
        ticks in vec((prop_oneof![9 => Just(1_u64), 1 => Just(2)], any::<bool>(), any::<bool>(), 0.0..=MAX_LINES), 1..=MAX_TICKS),
    ) {
        let w = Wheel { notch: NOTCH, lines: setting, chars: setting, at_pointer: true };
        let (mut c, mut now) = (Course::new(CONFIG), 0);
        for (id, on_pointer, read, lines) in ticks {
            let a = aim(id, on_pointer);
            if read {
                c.read(&a, Dir::Down, at(now), pair(Dir::Down, START));
                now += CONFIG.check_ms;
            }
            let next = c.next_route(&a, Dir::Down, w);
            if let Some(turn) = c.turn(&a, Dir::Down, lines, w) {
                prop_assert_eq!(turn.route, next);
            }
        }
    }

    #[test]
    fn steps_give_the_wheel_back_only_when_they_end_where_it_stopped(
        tick in TICK_MS,
        stopped in 0.0..START,
        moved in any::<bool>(),
        owed in vec(0.0..=MAX_LINES, 1..=MAX_TICKS),
    ) {
        let (mut c, a, w) = (Course::new(CONFIG), aim(1, false), wheel(THREE, THREE));
        let (mut now, mut pct) = (0, stopped);
        for lines in owed {
            if c.read(&a, Dir::Down, at(now), pair(Dir::Down, pct)) == Reach::End {
                prop_assert_eq!(c.tripped(), moved, "an end at {} after a trip at {}", pct, stopped);
            }
            if moved && c.tripped() {
                pct = stopped + NUDGE;
            }
            c.turn(&a, Dir::Down, lines, w);
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
