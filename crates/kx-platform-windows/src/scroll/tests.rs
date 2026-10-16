//! Wheel deltas, the carried part of a step, routes and percents, on examples.
#![allow(
    clippy::float_cmp,
    reason = "The values compared are whole numbers or exact binary fractions."
)]

use super::*;
use kx_module_api::ports::{Aim, ScrollConfig, TargetId};
use kx_module_api::{Mono, US_PER_MS};

mod area;

/// Windows' `WHEEL_DELTA`; a Windows test checks it against the bindings.
pub(super) const NOTCH: u32 = 120;
/// Windows' default wheel: three lines or characters a notch.
pub(super) const THREE: u32 = 3;
/// The `[scroll]` defaults.
pub(super) const CONFIG: ScrollConfig = ScrollConfig {
    check_ms: 250,
    read_ms: 300,
    edge_px: 8,
};
/// A sixty-fourth of a line: at three lines a notch it is five eighths of a step, exactly.
const BIT: f64 = 1.0 / 64.0;

/// A wheel of `NOTCH` with these settings.
pub(super) const fn wheel(lines: u32, chars: u32) -> Wheel {
    Wheel {
        notch: NOTCH,
        lines,
        chars,
        at_pointer: true,
    }
}

/// Target `id`, with the pointer on it or not.
pub(super) fn aim(id: u64, on_pointer: bool) -> Aim {
    Aim {
        at: Point::default(),
        on_pointer,
        area: None,
        ours: false,
        id: TargetId::new(id),
    }
}

/// `ms` milliseconds after the start.
pub(super) fn at(ms: u32) -> Mono {
    Mono::from_us(u64::from(ms) * u64::from(US_PER_MS))
}

/// `n` as a turn's count.
fn count(n: u32) -> u16 {
    u16::try_from(n).unwrap()
}

/// What each of `lines` turned Down on a fresh course at a post target: none, or the route and count.
fn turns(lines: &[f64], w: Wheel) -> Vec<Option<(Route, u16)>> {
    let (mut c, a) = (Course::new(CONFIG), aim(1, false));
    let mut turn = |l| c.turn(&a, Dir::Down, l, w).map(|t| (t.route, t.count));
    lines.iter().map(|&l| turn(l)).collect()
}

#[test]
fn three_lines_a_notch_make_a_line_a_third_of_a_notch() {
    let w = wheel(THREE, THREE);
    assert_eq!(w.per_line(Dir::Down), Some(f64::from(NOTCH / THREE)));
    let post = |n| Some((Route::Post, count(n)));
    assert_eq!(turns(&[3.0, 1.0], w), [post(NOTCH), post(NOTCH / THREE)]);
}

#[test]
fn one_line_a_notch_makes_a_line_a_whole_notch() {
    let w = wheel(1, 1);
    assert_eq!(w.per_line(Dir::Up), Some(f64::from(NOTCH)));
    let post = |n| Some((Route::Post, count(n)));
    assert_eq!(turns(&[1.0, 0.5], w), [post(NOTCH), post(NOTCH / 2)]);
}

#[test]
fn a_page_or_no_lines_a_notch_goes_by_ui_automation_steps() {
    for setting in [PAGE_SCROLL, 0] {
        let w = wheel(setting, THREE);
        assert_eq!(w.per_line(Dir::Down), None, "{setting}");
        let uia = |n| Some((Route::Uia, n));
        assert_eq!(turns(&[2.5, 0.5], w), [uia(2), uia(1)], "{setting}");
        let mut c = Course::new(CONFIG);
        let on = c.turn(&aim(1, true), Dir::Down, 1.0, w).map(|t| t.route);
        assert_eq!(on, Some(Route::Uia), "even with the pointer on it");
    }
}

#[test]
fn sideways_reads_the_character_setting_by_the_same_rule() {
    let w = Wheel {
        notch: NOTCH,
        lines: PAGE_SCROLL,
        chars: 1,
        at_pointer: true,
    };
    assert_eq!(w.per_line(Dir::Left), Some(f64::from(NOTCH)));
    assert_eq!(w.per_line(Dir::Right), Some(f64::from(NOTCH)));
    assert_eq!(w.per_line(Dir::Up), None);
    let w = wheel(THREE, 0);
    assert_eq!(w.per_line(Dir::Right), None);
    assert_eq!(w.per_line(Dir::Down), Some(f64::from(NOTCH / THREE)));
    assert_eq!(wheel(THREE, PAGE_SCROLL).per_line(Dir::Left), None);
    let no_notch = Wheel { notch: 0, ..w };
    assert_eq!(
        no_notch.per_line(Dir::Down),
        None,
        "a notch of 0 turns nothing"
    );
}

#[test]
fn a_part_of_a_step_carries_to_the_next_tick() {
    let post = |n| Some((Route::Post, n));
    let got = turns(&[BIT, BIT, BIT, BIT, BIT], wheel(THREE, THREE));
    assert_eq!(got, [None, post(1), None, post(1), post(1)], "0.625 a tick");
}

#[test]
fn a_new_target_or_way_drops_the_carried_part_but_a_new_route_keeps_it() {
    let w = wheel(THREE, THREE);
    let (post, on) = (aim(1, false), aim(1, true));
    let mut c = Course::new(CONFIG);
    assert_eq!(c.turn(&post, Dir::Down, BIT, w), None);
    let sent = c.turn(&on, Dir::Down, BIT, w).map(|t| (t.route, t.count));
    assert_eq!(sent, Some((Route::Send, 1)), "kept");
    assert_eq!(c.turn(&post, Dir::Down, BIT, w), None);
    assert_eq!(c.turn(&post, Dir::Up, BIT, w), None, "dropped: 0.625 again");
    assert_eq!(c.turn(&aim(2, false), Dir::Up, BIT, w), None, "dropped");
    assert!(c.turn(&aim(2, false), Dir::Up, BIT, w).is_some());
}

#[test]
fn the_pointer_on_the_target_sends_and_elsewhere_posts() {
    assert_eq!(route(true, true), Route::Send);
    assert_eq!(route(false, true), Route::Post);
    assert_eq!(route(true, false), Route::Uia);
    assert_eq!(route(false, false), Route::Uia);
    let mut c = Course::new(CONFIG);
    let on = c.turn(&aim(1, true), Dir::Right, 1.0, wheel(THREE, THREE));
    let on = on.map(|t| (t.route, t.count));
    assert_eq!(on, Some((Route::Send, count(NOTCH / THREE))));
}

#[test]
fn a_turn_stops_at_the_largest_wheel_word_and_drops_the_rest() {
    let w = wheel(1, 1);
    let most = Some((Route::Post, MAX_TURN));
    assert_eq!(turns(&[1e6, 0.0], w), [most, None], "nothing piles up");
    assert_eq!(turns(&[f64::MAX, 0.5], w), [most, Some((Route::Post, 60))]);
    let bad = [f64::NAN, f64::INFINITY, -1.0, 0.0];
    assert_eq!(
        turns(&bad, w),
        [None; 4],
        "lines the port forbids turn nothing"
    );
}

#[test]
fn wheel_params_pack_signed_words() {
    assert_eq!(wheel_wparam(-120), 0xFF88_0000);
    assert_eq!(wheel_wparam(120), 0x0078_0000);
    assert_eq!(point_lparam(Point { x: -1, y: 2 }), 0x0002_FFFF);
}

#[test]
fn each_way_turns_its_own_wheel_its_own_way() {
    assert_eq!((sign(Dir::Down), Dir::Down.sideways()), (-1, false));
    assert_eq!((sign(Dir::Up), Dir::Up.sideways()), (1, false));
    assert_eq!((sign(Dir::Right), Dir::Right.sideways()), (1, true));
    assert_eq!((sign(Dir::Left), Dir::Left.sideways()), (-1, true));
    let t = Turn {
        route: Route::Post,
        count: 40,
    };
    assert_eq!((t.delta(Dir::Down), t.delta(Dir::Right)), (-40, 40));
}

#[test]
fn percents_count_only_when_they_move_the_scroll_way() {
    let (top, lower) = (Some((0.0, 0.0)), Some((0.0, 4.0)));
    assert!(percent_moved(Dir::Down, top, lower));
    assert!(!percent_moved(Dir::Up, top, lower));
    assert!(percent_moved(Dir::Up, lower, top));
    assert!(!percent_moved(Dir::Right, top, lower));
    assert!(!percent_moved(Dir::Down, top, None));
    let cannot = Some((0.0, -1.0));
    assert!(!percent_moved(Dir::Down, cannot, Some((0.0, 3.0))));
}

#[test]
fn bad_percents_are_refused() {
    assert_eq!(along(Dir::Right, (25.0, -1.0)), Some(25.0));
    assert_eq!(
        along(Dir::Down, (25.0, -1.0)),
        None,
        "cannot scroll that way"
    );
    for bad in [f64::NAN, f64::INFINITY, FULL + 0.5, -0.5] {
        assert_eq!(along(Dir::Up, (0.0, bad)), None, "{bad}");
    }
    assert_eq!(along(Dir::Left, (FULL, 0.0)), Some(FULL));
}

#[test]
fn a_percent_rounded_just_past_its_range_is_its_end() {
    let (over, under) = (FULL + END_SLACK / 2.0, -END_SLACK / 2.0);
    assert_eq!(along(Dir::Down, (0.0, over)), Some(FULL));
    assert_eq!(along(Dir::Right, (under, 0.0)), Some(0.0));
    assert_eq!(along(Dir::Down, (0.0, FULL + END_SLACK * 2.0)), None);
}

#[test]
fn the_end_reads_from_the_percent() {
    assert_eq!(reach(Dir::Down, Some(FULL)), Reach::End);
    assert_eq!(reach(Dir::Right, Some(FULL)), Reach::End);
    assert_eq!(reach(Dir::Down, Some(99.5)), Reach::More);
    assert_eq!(reach(Dir::Up, Some(0.0)), Reach::End);
    assert_eq!(reach(Dir::Left, Some(0.0)), Reach::End);
    assert_eq!(reach(Dir::Up, Some(0.5)), Reach::More);
    assert_eq!(reach(Dir::Up, None), Reach::Unknown);
    assert_eq!(reach(Dir::Down, Some(FULL - END_SLACK / 2.0)), Reach::End);
    assert_eq!(reach(Dir::Left, Some(END_SLACK / 2.0)), Reach::End);
    assert_eq!(reach(Dir::Down, Some(FULL - END_SLACK * 2.0)), Reach::More);
}

#[test]
fn routes_keep_their_report_names() {
    let names = Route::ALL.map(Route::name);
    assert_eq!(names, ["uia", "post", "send"]);
}

#[test]
fn a_point_packed_in_an_lparam_reads_back_signed() {
    for (x, y) in [(0, 0), (-1, -360), (2559, 1439), (-32768, 32767)] {
        let p = Point { x, y };
        assert_eq!(lparam_point(point_lparam(p)), p);
    }
    assert_eq!(lparam_point(0x0001_FFFF), Point { x: -1, y: 1 });
}

#[cfg(windows)]
#[test]
fn the_constants_match_windows() {
    use windows::Win32::UI::Accessibility::UIA_ScrollPatternNoScroll;
    use windows::Win32::UI::WindowsAndMessaging::WHEEL_DELTA;
    assert_eq!(NOTCH, WHEEL_DELTA);
    assert_eq!(along(Dir::Down, (0.0, UIA_ScrollPatternNoScroll)), None);
}
