//! Time: the exact hold time, a late tick, a missed release, and values at the ends of their range.

use super::*;

#[test]
fn a_release_at_exactly_the_hold_time_counts_as_a_hold() {
    let mut e = eng(true);
    let p0 = p(5, 5);
    on(&mut e, down(p0), 0);
    assert_eq!(on(&mut e, up(p0), HOLD), eat(right_click(p0)));
    assert_eq!(on(&mut e, up(p0), HOLD + 1), pass(vec![]));
}

#[test]
fn input_after_the_hold_time_fires_the_hold_first_even_if_the_tick_is_late() {
    let mut e = eng(true);
    let (p0, to) = (p(100, 100), p(130, 100));
    on(&mut e, down(p0), 0);
    let late = on(&mut e, mv(to), HOLD + 10);
    assert_eq!(late, eat([right_click(p0), vec![A::Move(to)]].concat()));
    assert_eq!(on(&mut e, up(to), HOLD + 20), eat(vec![]));
}

#[test]
fn a_press_that_reveals_a_late_hold_reaches_the_app_after_its_right_click() {
    let mut e = eng(true);
    let (p0, p1) = (p(10, 10), p(20, 20));
    on(&mut e, down(p0), 0);
    let late = on(&mut e, press(Right, p1, DPI), HOLD);
    let want = [right_click(p0), vec![A::Down(Right, p1)]].concat();
    assert_eq!(late, eat(want));
    assert_eq!(on(&mut e, lift(Right, p1), HOLD + 1), pass(vec![]));
    assert_eq!(on(&mut e, up(p1), HOLD + 2), eat(vec![]));
}

#[test]
fn esc_that_reveals_a_late_hold_still_passes() {
    let mut e = eng(true);
    let p0 = p(10, 10);
    on(&mut e, down(p0), 0);
    assert_eq!(on(&mut e, PointerEvent::Esc, HOLD), pass(right_click(p0)));
}

#[test]
fn a_press_while_a_release_is_awaited_starts_afresh() {
    let mut e = eng(true);
    let (p0, p1) = (p(10, 10), p(40, 40));
    on(&mut e, down(p0), 0);
    tick(&mut e, HOLD);
    assert_eq!(on(&mut e, down(p1), HOLD * 2), eat(vec![]));
    let click = vec![A::Down(Left, p1), A::Up(Left, p1)];
    assert_eq!(on(&mut e, up(p1), HOLD * 2 + 1), eat(click));
}

#[test]
fn a_press_while_a_press_is_held_back_clicks_the_first_then_holds_the_second() {
    let mut e = eng(true);
    e.set_latches(vec![SHIFT]);
    let (p0, p1) = (p(10, 10), p(40, 40));
    on(&mut e, down(p0), 0);
    let click = vec![
        A::KeyDown(SHIFT),
        A::Down(Left, p0),
        A::Up(Left, p0),
        A::KeyUp(SHIFT),
    ];
    assert_eq!(on(&mut e, down(p1), 10), eat(click));
    assert_eq!(e.deadline(), Some(mono(10 + HOLD)));
    let fire = [
        vec![A::KeyDown(SHIFT)],
        right_click(p1),
        vec![A::KeyUp(SHIFT)],
    ];
    assert_eq!(tick(&mut e, 10 + HOLD), fire.concat());
}

#[test]
fn a_press_on_our_window_while_a_press_is_held_back_follows_its_click() {
    let mut e = eng(true);
    let (p0, key) = (p(10, 10), p(500, 900));
    on(&mut e, down(p0), 0);
    let want = vec![A::Down(Left, p0), A::Up(Left, p0), A::Down(Left, key)];
    assert_eq!(on(&mut e, ours(key), 10), eat(want));
    assert_eq!(on(&mut e, up(key), 20), pass(vec![]));
    assert_eq!(e.deadline(), None);
}

#[test]
fn a_huge_hold_time_or_a_far_move_never_overflows() {
    let cfg = EngineCfg {
        hold_us: u64::MAX,
        still_px: STILL,
        rclick: true,
    };
    let mut e = Engine::new(cfg);
    let (p0, to) = (p(i32::MIN, i32::MIN), p(i32::MAX, i32::MAX));
    on(&mut e, press(Left, p0, u32::MAX), 5);
    assert_eq!(e.deadline(), Some(mono(u64::MAX)));
    assert_eq!(tick(&mut e, u64::MAX - 1), vec![]);
    let want = vec![A::Down(Left, p0), A::Move(to)];
    assert_eq!(on(&mut e, mv(to), 6), eat(want));
}
