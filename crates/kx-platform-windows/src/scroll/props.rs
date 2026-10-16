//! Properties over random runs and desks: carried steps never drift by a whole step, the fallback trips only on a
//! percent still under wheel steps for `check_ms`, and a missed history point lands on the screen, off the keyboard.

use super::entry::{from_history, pushed_out, to_history};
use super::tests::{CONFIG, NOTCH, aim, at, wheel};
use super::*;
use crate::screen::{contains, edges, rect_of};
use kx_module_api::ports::Rect;
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
/// Desk corners lie within this many pixels of the origin, either way.
const SPAN: i32 = 20_000;
/// The keyboard reaches past the desk by up to this share of it, as 1 / `REACH`.
const REACH: i32 = 8;
/// The largest `edge_px` tried, the setting's top.
const MAX_EDGE: u32 = 64;
/// Most percent reads in one run; a walk of this many one-point steps from `START` stays short of either end.
const MAX_READS: usize = 40;
/// Where a walking percent starts.
const START: f64 = 50.0;

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

/// A desk, a keyboard that may reach past it, and an entry point inside the keyboard when `inside` and it has room.
fn desk() -> impl Strategy<Value = (Rect, Rect, Point)> {
    (-SPAN..SPAN, -SPAN..SPAN, 1..SPAN, 1..SPAN)
        .prop_flat_map(|(x, y, w, h)| {
            let (xs, ys) = (
                x - w / REACH..x + w + w / REACH,
                y - h / REACH..y + h + h / REACH,
            );
            let corners = (xs.clone(), ys.clone(), xs.clone(), ys.clone());
            (Just(Rect { x, y, w, h }), corners, (xs, ys), any::<bool>())
        })
        .prop_map(|(desk, (x0, y0, x1, y1), (ex, ey), inside)| {
            let kb = rect_of([x0.min(x1), y0.min(y1), x0.max(x1), y0.max(y1)]);
            let entry = if inside && kb.w > 0 && kb.h > 0 {
                Point {
                    x: kb.x + ex.rem_euclid(kb.w),
                    y: kb.y + ey.rem_euclid(kb.h),
                }
            } else {
                Point { x: ex, y: ey }
            };
            (desk, kb, entry)
        })
}

/// True when `kb` covers all of `desk`.
fn covers(kb: Rect, desk: Rect) -> bool {
    let (k, d) = (edges(kb), edges(desk));
    k[0] <= d[0] && k[1] <= d[1] && k[2] >= d[2] && k[3] >= d[3]
}

/// A run of read gaps in ms, each at most one check.
fn gaps() -> impl Strategy<Value = Vec<u32>> {
    vec(0..=CONFIG.check_ms, 1..=MAX_READS)
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
    fn a_still_percent_under_wheel_steps_trips_at_the_first_read_after_check_ms(
        gaps in gaps(),
        pct in 0.0..FULL,
    ) {
        let (mut c, a, w) = (Course::new(CONFIG), aim(1, false), wheel(1, 1));
        let (still, mut now) = (Some((0.0, pct)), 0);
        c.read(&a, Dir::Down, at(now), still);
        for gap in gaps {
            prop_assert!(c.turn(&a, Dir::Down, 1.0, w).is_some());
            now += gap;
            c.read(&a, Dir::Down, at(now), still);
            prop_assert_eq!(c.tripped(), now >= CONFIG.check_ms, "at {} ms", now);
        }
    }

    #[test]
    fn a_percent_that_moves_at_every_read_never_trips(
        gaps in gaps(),
        walk in vec(any::<bool>(), MAX_READS),
    ) {
        let (mut c, a, w) = (Course::new(CONFIG), aim(1, false), wheel(1, 1));
        let (mut pct, mut now) = (START, 0);
        c.read(&a, Dir::Down, at(now), Some((0.0, pct)));
        for (gap, up) in gaps.into_iter().zip(walk) {
            c.turn(&a, Dir::Down, 1.0, w);
            (pct, now) = (if up { pct + 1.0 } else { pct - 1.0 }, now + gap);
            c.read(&a, Dir::Down, at(now), Some((0.0, pct)));
        }
        prop_assert!(!c.tripped(), "moved either way at every read");
    }

    #[test]
    fn a_missed_history_point_lands_on_the_screen_off_the_keyboard(
        (desk, kb, entry) in desk(),
        edge in 0..=MAX_EDGE,
    ) {
        match pushed_out(entry, kb, desk, edge) {
            None => prop_assert!(covers(kb, desk), "no point though the screen has room"),
            Some(at) => {
                prop_assert!(contains(desk, at), "off the screen");
                prop_assert!(!contains(kb, at), "on the keyboard");
            }
        }
    }

    #[test]
    fn history_words_round_trip(x in any::<i16>(), y in any::<i16>()) {
        let at = Point { x: i32::from(x), y: i32::from(y) };
        let (wx, wy) = to_history(at);
        prop_assert!((0..=i32::from(u16::MAX)).contains(&wx));
        prop_assert!((0..=i32::from(u16::MAX)).contains(&wy));
        prop_assert_eq!(from_history(wx, wy), at);
    }
}
