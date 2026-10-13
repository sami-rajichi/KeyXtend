//! Grab: the next press outside picks up at once, the next click drops; our window, Esc or a toggle cancels.

use super::*;
use GrabState::{Armed, Carrying, Off};

/// The cancel's input: Esc to the app, then the button let go at `at`.
fn cancel(at: Point) -> Vec<PointerAct> {
    vec![A::KeyDown(ESC), A::KeyUp(ESC), A::Up(Left, at)]
}

/// An engine carrying from `p0`, with the grabbing press already released.
fn carrying(rclick: bool, p0: Point) -> Engine {
    let mut e = eng(rclick);
    toggle(&mut e, Toggle::Grab);
    on(&mut e, down(p0), 0);
    on(&mut e, up(p0), 10);
    e
}

#[test]
fn grab_picks_up_at_once_then_drops_on_the_next_click_and_restores_right_click() {
    let mut e = eng(true);
    assert_eq!(toggle(&mut e, Toggle::Grab), vec![]);
    assert_eq!((e.rclick(), e.grab()), (false, Armed));
    let (p0, p2) = (p(10, 10), p(300, 200));
    assert_eq!(on(&mut e, down(p0), 0), pass(vec![]));
    assert_eq!((e.grab(), e.deadline()), (Carrying, None));
    assert_eq!(on(&mut e, up(p0), 10), eat(vec![]));
    assert_eq!(on(&mut e, mv(p2), 20), pass(vec![]));
    assert_eq!(on(&mut e, down(p2), 30), eat(vec![A::Up(Left, p2)]));
    assert_eq!(on(&mut e, up(p2), 40), eat(vec![]));
    assert_eq!((e.rclick(), e.grab()), (true, Off));
    assert_eq!(on(&mut e, up(p2), 50), pass(vec![]));
}

#[test]
fn grab_never_waits_for_the_hold_time() {
    let mut e = eng(false);
    toggle(&mut e, Toggle::Grab);
    let p0 = p(1, 1);
    assert_eq!(on(&mut e, down(p0), 0), pass(vec![]));
    assert_eq!(tick(&mut e, HOLD), vec![]);
    assert_eq!(on(&mut e, up(p0), HOLD), eat(vec![]));
    assert_eq!(e.grab(), Carrying);
}

#[test]
fn a_drop_leaves_right_click_off_when_it_was_off() {
    let p2 = p(90, 90);
    let mut e = carrying(false, p(1, 1));
    assert_eq!(on(&mut e, down(p2), 20), eat(vec![A::Up(Left, p2)]));
    assert_eq!((e.rclick(), e.grab()), (false, Off));
}

#[test]
fn esc_cancels_a_carry_and_releases_the_button_where_the_pointer_is() {
    let p3 = p(50, 60);
    let mut e = carrying(true, p(10, 10));
    on(&mut e, mv(p3), 20);
    let esc = PointerEvent::Esc;
    assert_eq!(on(&mut e, esc, 30), pass(vec![A::Up(Left, p3)]));
    assert_eq!((e.rclick(), e.grab()), (true, Off));
    assert_eq!(on(&mut e, esc, 40), pass(vec![]));
    assert_eq!(on(&mut e, down(p3), 50), eat(vec![]));
}

#[test]
fn esc_while_the_grabbing_press_is_still_down_waits_for_its_release() {
    let mut e = eng(false);
    toggle(&mut e, Toggle::Grab);
    let p0 = p(7, 7);
    on(&mut e, down(p0), 0);
    let esc = PointerEvent::Esc;
    assert_eq!(on(&mut e, esc, 1), pass(vec![A::Up(Left, p0)]));
    assert_eq!(on(&mut e, up(p0), 2), eat(vec![]));
    assert_eq!(on(&mut e, up(p0), 3), pass(vec![]));
}

#[test]
fn a_click_on_our_window_cancels_a_carry_with_esc() {
    let key = p(500, 900);
    let mut e = carrying(true, p(7, 7));
    assert_eq!(on(&mut e, ours(key), 20), eat(cancel(key)));
    assert_eq!(on(&mut e, up(key), 30), eat(vec![]));
    assert_eq!((e.rclick(), e.grab()), (true, Off));
}

#[test]
fn toggling_grab_during_a_carry_cancels_it_with_esc() {
    let p3 = p(70, 70);
    let mut e = carrying(false, p(7, 7));
    on(&mut e, mv(p3), 20);
    assert_eq!(toggle(&mut e, Toggle::Grab), cancel(p3));
    assert_eq!((e.rclick(), e.grab()), (false, Off));
    assert_eq!(on(&mut e, down(p3), HOLD * 3), pass(vec![]));
}

#[test]
fn toggling_right_click_during_a_carry_cancels_it_and_turns_right_click_on() {
    let p3 = p(70, 70);
    let mut e = carrying(false, p(7, 7));
    on(&mut e, mv(p3), 20);
    assert_eq!(toggle(&mut e, Toggle::RightClick), cancel(p3));
    assert_eq!((e.rclick(), e.grab()), (true, Off));
}

#[test]
fn a_toggle_while_the_grabbing_press_is_down_waits_for_its_release() {
    let mut e = eng(false);
    toggle(&mut e, Toggle::Grab);
    let p0 = p(7, 7);
    on(&mut e, down(p0), 0);
    assert_eq!(toggle(&mut e, Toggle::Grab), cancel(p0));
    assert_eq!(on(&mut e, up(p0), 1), eat(vec![]));
    assert_eq!(on(&mut e, up(p0), 2), pass(vec![]));
}

#[test]
fn grab_toggled_twice_without_a_carry_restores_right_click() {
    let mut e = eng(true);
    assert_eq!(toggle(&mut e, Toggle::Grab), vec![]);
    assert_eq!(toggle(&mut e, Toggle::Grab), vec![]);
    assert_eq!((e.rclick(), e.grab()), (true, Off));
}

#[test]
fn right_click_toggled_while_grab_is_armed_ends_grab() {
    let mut e = eng(false);
    toggle(&mut e, Toggle::Grab);
    assert_eq!(toggle(&mut e, Toggle::RightClick), vec![]);
    assert_eq!((e.rclick(), e.grab()), (true, Off));
}

#[test]
fn arming_grab_during_a_hold_replays_the_held_press() {
    let mut e = eng(true);
    let (p0, p1) = (p(3, 3), p(30, 30));
    on(&mut e, down(p0), 0);
    assert_eq!(toggle(&mut e, Toggle::Grab), vec![A::Down(Left, p0)]);
    assert_eq!((e.grab(), e.deadline()), (Armed, None));
    assert_eq!(on(&mut e, up(p0), 10), pass(vec![]));
    assert_eq!(on(&mut e, down(p1), 20), pass(vec![]));
    assert_eq!(e.grab(), Carrying);
}

#[test]
fn a_press_on_our_window_never_picks_up() {
    let mut e = eng(false);
    toggle(&mut e, Toggle::Grab);
    let key = p(500, 900);
    assert_eq!(on(&mut e, ours(key), 0), pass(vec![]));
    assert_eq!(on(&mut e, up(key), 10), pass(vec![]));
    assert_eq!(e.grab(), Armed);
}

#[test]
fn other_buttons_and_moves_pass_during_a_carry() {
    let p1 = p(40, 40);
    let mut e = carrying(false, p(7, 7));
    assert_eq!(on(&mut e, mv(p1), 20), pass(vec![]));
    assert_eq!(on(&mut e, press(Right, p1, DPI), 30), pass(vec![]));
    assert_eq!(on(&mut e, lift(Right, p1), 40), pass(vec![]));
    assert_eq!(e.grab(), Carrying);
}

#[test]
fn set_rclick_sets_what_the_user_sees() {
    let mut acts = Vec::new();
    let mut e = eng(false);
    e.set_rclick(true, &mut acts);
    assert!(e.rclick());
    toggle(&mut e, Toggle::Grab);
    e.set_rclick(false, &mut acts);
    assert_eq!((e.rclick(), e.grab()), (false, Armed));
    toggle(&mut e, Toggle::Grab);
    assert_eq!((e.rclick(), e.grab()), (false, Off));
    toggle(&mut e, Toggle::Grab);
    e.set_rclick(true, &mut acts);
    assert_eq!((e.rclick(), e.grab(), acts), (true, Off, vec![]));
}
