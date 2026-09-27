use super::*;
use Button::{Left, Right};

const HOLD: i64 = 1_500_000;
const STILL: i32 = 7;
const SHIFT: u16 = 0x10;

fn p(x: i32, y: i32) -> Pt {
    Pt { x, y }
}

fn eng(mode: Mode) -> Engine {
    let mut e = Engine::new(HOLD, STILL);
    e.set_mode(mode);
    e
}

fn right_click(at: Pt) -> Vec<Act> {
    vec![Act::Down(Right, at), Act::Up(Right, at)]
}

#[test]
fn off_mode_lets_everything_through() {
    let mut e = eng(Mode::Off);
    assert_eq!(e.on(Event::Down(Left, p(1, 1), false), 0), pass(vec![]));
    assert_eq!(e.deadline(), None);
    assert_eq!(e.on(Event::Up(Left, p(1, 1)), HOLD * 2), pass(vec![]));
}

#[test]
fn a_short_click_is_replayed_at_the_press_point() {
    let mut e = eng(Mode::RightClick);
    let p0 = p(10, 10);
    assert_eq!(e.on(Event::Down(Left, p0, false), 0), eat(vec![]));
    assert_eq!(e.deadline(), Some(HOLD));
    let want = vec![Act::Down(Left, p0), Act::Up(Left, p0)];
    assert_eq!(e.on(Event::Up(Left, p(12, 11)), 200_000), eat(want));
    assert_eq!(e.deadline(), None);
}

#[test]
fn a_still_hold_becomes_a_right_click_and_its_release_is_dropped() {
    let mut e = eng(Mode::RightClick);
    let p0 = p(10, 10);
    e.on(Event::Down(Left, p0, false), 0);
    assert_eq!(e.on(Event::Tick, HOLD - 1), pass(vec![]));
    assert_eq!(e.on(Event::Tick, HOLD), pass(right_click(p0)));
    assert_eq!(e.on(Event::Up(Left, p0), HOLD + 9), eat(vec![]));
    assert_eq!(e.on(Event::Down(Left, p0, false), HOLD * 2), eat(vec![]));
}

#[test]
fn six_px_is_still_but_eight_px_starts_a_normal_drag() {
    let p0 = p(100, 100);
    let mut e = eng(Mode::RightClick);
    e.on(Event::Down(Left, p0, false), 0);
    assert_eq!(e.on(Event::Move(p(106, 100)), 10), pass(vec![]));
    assert_eq!(e.on(Event::Tick, HOLD), pass(right_click(p0)));

    let mut e = eng(Mode::RightClick);
    e.on(Event::Down(Left, p0, false), 0);
    let to = p(108, 100);
    let want = vec![Act::Down(Left, p0), Act::Move(to)];
    assert_eq!(e.on(Event::Move(to), 10), eat(want));
    assert_eq!(e.deadline(), None);
    assert_eq!(e.on(Event::Up(Left, to), 20), pass(vec![]));
    assert_eq!(e.on(Event::Tick, HOLD), pass(vec![]));
}

#[test]
fn a_release_at_exactly_the_hold_time_counts_as_a_hold() {
    let mut e = eng(Mode::RightClick);
    let p0 = p(5, 5);
    e.on(Event::Down(Left, p0, false), 0);
    assert_eq!(e.on(Event::Up(Left, p0), HOLD), eat(right_click(p0)));
    assert_eq!(e.on(Event::Up(Left, p0), HOLD + 1), pass(vec![]));
}

#[test]
fn a_second_button_replays_the_held_press_first() {
    let mut e = eng(Mode::RightClick);
    let (p0, p1) = (p(1, 1), p(2, 2));
    e.on(Event::Down(Left, p0, false), 0);
    let want = vec![Act::Down(Left, p0), Act::Down(Right, p1)];
    assert_eq!(e.on(Event::Down(Right, p1, false), 10), eat(want));
    assert_eq!(e.on(Event::Up(Right, p1), 20), pass(vec![]));
    assert_eq!(e.on(Event::Up(Left, p1), 30), pass(vec![]));
}

#[test]
fn a_mode_switch_replays_the_held_press() {
    let mut e = eng(Mode::RightClick);
    let p0 = p(3, 3);
    e.on(Event::Down(Left, p0, false), 0);
    assert_eq!(e.set_mode(Mode::Off), vec![Act::Down(Left, p0)]);
    assert_eq!(e.on(Event::Up(Left, p0), 10), pass(vec![]));
}

#[test]
fn a_press_on_our_own_window_is_never_held() {
    let mut e = eng(Mode::RightClick);
    let p0 = p(4, 4);
    assert_eq!(e.on(Event::Down(Left, p0, true), 0), pass(vec![]));
    assert_eq!(e.on(Event::Tick, HOLD), pass(vec![]));
    assert_eq!(e.on(Event::Up(Left, p0), HOLD + 1), pass(vec![]));
}

#[test]
fn grab_latches_then_drops_on_the_next_click_and_restores_right_click() {
    let mut e = eng(Mode::RightClick);
    e.set_mode(Mode::Grab);
    let (p0, p2) = (p(10, 10), p(300, 200));
    e.on(Event::Down(Left, p0, false), 0);
    assert_eq!(e.on(Event::Tick, HOLD), pass(vec![Act::Down(Left, p0)]));
    assert_eq!(e.on(Event::Up(Left, p0), HOLD + 10), eat(vec![]));
    assert_eq!(e.on(Event::Move(p2), HOLD + 20), pass(vec![]));
    let drop = e.on(Event::Down(Left, p2, false), HOLD + 30);
    assert_eq!(drop, eat(vec![Act::Up(Left, p2)]));
    assert_eq!(e.on(Event::Up(Left, p2), HOLD + 40), eat(vec![]));
    assert_eq!(e.mode(), Mode::RightClick);
}

#[test]
fn esc_cancels_a_carry_and_releases_the_button_where_the_pointer_is() {
    let mut e = eng(Mode::Grab);
    let (p0, p3) = (p(10, 10), p(50, 60));
    e.on(Event::Down(Left, p0, false), 0);
    e.on(Event::Tick, HOLD);
    e.on(Event::Up(Left, p0), HOLD + 10);
    e.on(Event::Move(p3), HOLD + 20);
    assert_eq!(e.on(Event::Esc, HOLD + 30), pass(vec![Act::Up(Left, p3)]));
    assert_eq!(e.mode(), Mode::Off);
    assert_eq!(e.on(Event::Esc, HOLD + 40), pass(vec![]));
}

#[test]
fn a_latched_modifier_is_released_after_the_next_left_click() {
    let mut e = eng(Mode::Off);
    assert_eq!(e.latch(SHIFT), vec![Act::KeyDown(SHIFT)]);
    assert_eq!(e.latch(SHIFT), vec![]);
    assert_eq!(e.on(Event::Down(Left, p(1, 1), false), 0), pass(vec![]));
    let up = e.on(Event::Up(Left, p(1, 1)), 10);
    assert_eq!(up, pass(vec![Act::KeyUp(SHIFT)]));
    assert_eq!(e.on(Event::Up(Left, p(1, 1)), 20), pass(vec![]));
}

#[test]
fn a_latched_modifier_waits_for_a_held_back_click() {
    let mut e = eng(Mode::RightClick);
    let p0 = p(9, 9);
    e.latch(SHIFT);
    e.on(Event::Down(Left, p0, false), 0);
    let want = vec![Act::Down(Left, p0), Act::Up(Left, p0), Act::KeyUp(SHIFT)];
    assert_eq!(e.on(Event::Up(Left, p0), 100), eat(want));
}

#[test]
fn reset_releases_a_carry_and_latched_modifiers() {
    let mut e = eng(Mode::Grab);
    let (p0, p3) = (p(10, 10), p(40, 40));
    e.latch(SHIFT);
    e.on(Event::Down(Left, p0, false), 0);
    e.on(Event::Tick, HOLD);
    e.on(Event::Up(Left, p0), HOLD + 1);
    e.on(Event::Move(p3), HOLD + 2);
    assert_eq!(e.reset(), vec![Act::Up(Left, p3), Act::KeyUp(SHIFT)]);
    assert_eq!((e.mode(), e.deadline()), (Mode::Off, None));

    let mut e = eng(Mode::RightClick);
    e.latch(SHIFT);
    e.on(Event::Down(Left, p0, false), 0);
    assert_eq!(e.reset(), vec![Act::Down(Left, p0), Act::KeyUp(SHIFT)]);
}

#[test]
fn input_after_the_hold_time_fires_the_hold_first_even_if_the_tick_is_late() {
    let mut e = eng(Mode::RightClick);
    let p0 = p(100, 100);
    e.on(Event::Down(Left, p0, false), 0);
    let late = e.on(Event::Move(p(130, 100)), HOLD + 10);
    assert_eq!(late, pass(right_click(p0)));
    assert_eq!(e.on(Event::Up(Left, p(130, 100)), HOLD + 20), eat(vec![]));
}

#[test]
fn a_release_at_exactly_the_hold_time_latches_in_grab() {
    let mut e = eng(Mode::Grab);
    let (p0, p2) = (p(1, 1), p(90, 90));
    e.on(Event::Down(Left, p0, false), 0);
    assert_eq!(
        e.on(Event::Up(Left, p0), HOLD),
        eat(vec![Act::Down(Left, p0)])
    );
    assert_eq!(e.on(Event::Move(p2), HOLD + 1), pass(vec![]));
    let drop = e.on(Event::Down(Left, p2, false), HOLD + 2);
    assert_eq!(drop, eat(vec![Act::Up(Left, p2)]));
}

#[test]
fn esc_while_the_grabbing_press_is_still_down_waits_for_its_release() {
    let mut e = eng(Mode::Grab);
    let p0 = p(7, 7);
    e.on(Event::Down(Left, p0, false), 0);
    e.on(Event::Tick, HOLD);
    assert_eq!(e.on(Event::Esc, HOLD + 1), pass(vec![Act::Up(Left, p0)]));
    assert_eq!(e.on(Event::Up(Left, p0), HOLD + 2), eat(vec![]));
    assert_eq!(e.on(Event::Up(Left, p0), HOLD + 3), pass(vec![]));
}

#[test]
fn a_mode_switch_during_a_carry_releases_the_button() {
    let mut e = eng(Mode::Grab);
    let (p0, p3) = (p(7, 7), p(70, 70));
    e.on(Event::Down(Left, p0, false), 0);
    e.on(Event::Tick, HOLD);
    e.on(Event::Up(Left, p0), HOLD + 1);
    e.on(Event::Move(p3), HOLD + 2);
    assert_eq!(e.set_mode(Mode::Off), vec![Act::Up(Left, p3)]);
    assert_eq!(e.on(Event::Down(Left, p0, false), HOLD * 3), pass(vec![]));
}

#[test]
fn a_click_on_our_window_cancels_a_carry_with_esc() {
    let mut e = eng(Mode::RightClick);
    e.set_mode(Mode::Grab);
    let (p0, key) = (p(7, 7), p(500, 900));
    e.on(Event::Down(Left, p0, false), 0);
    e.on(Event::Tick, HOLD);
    e.on(Event::Up(Left, p0), HOLD + 1);
    let cancel = vec![Act::KeyDown(ESC), Act::KeyUp(ESC), Act::Up(Left, key)];
    assert_eq!(e.on(Event::Down(Left, key, true), HOLD + 2), eat(cancel));
    assert_eq!(e.on(Event::Up(Left, key), HOLD + 3), eat(vec![]));
    assert_eq!(e.mode(), Mode::RightClick);
}

#[test]
fn a_click_on_our_window_keeps_latched_modifiers() {
    let mut e = eng(Mode::Off);
    e.latch(SHIFT);
    e.on(Event::Down(Left, p(1, 1), true), 0);
    assert_eq!(e.on(Event::Up(Left, p(1, 1)), 10), pass(vec![]));
    e.on(Event::Down(Left, p(2, 2), false), 20);
    let up = e.on(Event::Up(Left, p(2, 2)), 30);
    assert_eq!(up, pass(vec![Act::KeyUp(SHIFT)]));
}

#[test]
fn a_huge_hold_time_or_a_far_move_never_overflows() {
    let mut e = Engine::new(i64::MAX, STILL);
    e.set_mode(Mode::RightClick);
    let (p0, to) = (p(i32::MIN, i32::MIN), p(i32::MAX, i32::MAX));
    e.on(Event::Down(Left, p0, false), 5);
    assert_eq!(e.deadline(), Some(i64::MAX));
    assert_eq!(e.on(Event::Tick, i64::MAX - 1), pass(vec![]));
    let want = vec![Act::Down(Left, p0), Act::Move(to)];
    assert_eq!(e.on(Event::Move(to), 6), eat(want));
}
