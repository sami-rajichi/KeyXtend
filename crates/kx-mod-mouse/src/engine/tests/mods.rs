//! Modifier + click: the latched modifiers go down just before a click outside and up just after its release.

use super::*;

/// An engine with `latches` held, Right-click as given.
fn latched(rclick: bool, latches: &[Scan]) -> Engine {
    let mut e = eng(rclick);
    e.set_latches(latches.to_vec());
    e
}

#[test]
fn latches_go_around_the_next_click_outside_even_with_both_modes_off() {
    let mut e = latched(false, &[SHIFT]);
    let p0 = p(1, 1);
    let press_with = vec![A::KeyDown(SHIFT), A::Down(Left, p0)];
    assert_eq!(on(&mut e, down(p0), 0), eat(press_with));
    assert_eq!(notes(&mut e), vec![]);
    let lift_with = vec![A::Up(Left, p0), A::KeyUp(SHIFT)];
    assert_eq!(on(&mut e, up(p0), 10), eat(lift_with));
    assert_eq!(notes(&mut e), vec![Note::LatchesUsed]);
    assert_eq!(on(&mut e, up(p0), 20), pass(vec![]));
}

#[test]
fn a_click_carries_at_most_max_latches_in_their_order() {
    let many: Vec<Scan> = (0x70_u16..).take(MAX_LATCHES + 1).map(Scan::new).collect();
    let mut e = latched(false, &many);
    let p0 = p(1, 1);
    let mut want: Vec<PointerAct> = many[..MAX_LATCHES].iter().map(|&k| A::KeyDown(k)).collect();
    want.push(A::Down(Left, p0));
    assert_eq!(on(&mut e, down(p0), 0), eat(want));
}

#[test]
fn latches_stay_for_later_clicks_until_the_keyboard_changes_them() {
    let mut e = latched(false, &[SHIFT]);
    for (i, x) in (1..=2).enumerate() {
        let (at, t) = (p(x, x), u64::try_from(i).unwrap() * 100);
        assert_eq!(
            on(&mut e, down(at), t).1,
            vec![A::KeyDown(SHIFT), A::Down(Left, at)]
        );
        assert_eq!(
            on(&mut e, up(at), t + 10).1,
            vec![A::Up(Left, at), A::KeyUp(SHIFT)]
        );
    }
    e.set_latches(vec![]);
    assert_eq!(on(&mut e, down(p(3, 3)), 300), pass(vec![]));
}

#[test]
fn a_held_back_click_sends_its_modifiers_around_the_replay() {
    let mut e = latched(true, &[SHIFT]);
    let p0 = p(9, 9);
    assert_eq!(on(&mut e, down(p0), 0), eat(vec![]));
    let want = vec![
        A::KeyDown(SHIFT),
        A::Down(Left, p0),
        A::Up(Left, p0),
        A::KeyUp(SHIFT),
    ];
    assert_eq!(on(&mut e, up(p0), 100), eat(want));
}

#[test]
fn a_click_on_our_window_keeps_latched_modifiers() {
    let mut e = latched(true, &[SHIFT]);
    assert_eq!(on(&mut e, ours(p(1, 1)), 0), pass(vec![]));
    assert_eq!(on(&mut e, up(p(1, 1)), 10), pass(vec![]));
    assert!(!notes(&mut e).contains(&Note::LatchesUsed));
    let p2 = p(2, 2);
    on(&mut e, down(p2), 20);
    let want = vec![
        A::KeyDown(SHIFT),
        A::Down(Left, p2),
        A::Up(Left, p2),
        A::KeyUp(SHIFT),
    ];
    assert_eq!(on(&mut e, up(p2), 30), eat(want));
}

#[test]
fn modifiers_stay_down_through_a_drag_until_its_release() {
    let mut e = latched(true, &[SHIFT]);
    let (p0, to, end) = (p(10, 10), p(40, 10), p(80, 10));
    on(&mut e, down(p0), 0);
    let start = vec![A::KeyDown(SHIFT), A::Down(Left, p0), A::Move(to)];
    assert_eq!(on(&mut e, mv(to), 10), eat(start));
    assert_eq!(on(&mut e, mv(end), 20), pass(vec![]));
    assert_eq!(
        on(&mut e, up(end), 30),
        eat(vec![A::Up(Left, end), A::KeyUp(SHIFT)])
    );
    assert!(notes(&mut e).contains(&Note::LatchesUsed));
}

#[test]
fn a_hold_right_clicks_with_its_modifiers() {
    let mut e = latched(true, &[SHIFT]);
    let p0 = p(5, 5);
    on(&mut e, down(p0), 0);
    let fire = [
        vec![A::KeyDown(SHIFT)],
        right_click(p0),
        vec![A::KeyUp(SHIFT)],
    ];
    assert_eq!(tick(&mut e, HOLD), fire.concat());
    assert_eq!(on(&mut e, up(p0), HOLD + 1), eat(vec![]));
    assert!(notes(&mut e).contains(&Note::LatchesUsed));
}

#[test]
fn ctrl_stays_down_from_pick_up_to_drop() {
    let mut e = latched(false, &[CTRL]);
    toggle(&mut e, Toggle::Grab);
    let (p0, p2) = (p(10, 10), p(200, 200));
    let pick = vec![A::KeyDown(CTRL), A::Down(Left, p0)];
    assert_eq!(on(&mut e, down(p0), 0), eat(pick));
    assert_eq!(on(&mut e, up(p0), 10), eat(vec![]));
    assert_eq!(on(&mut e, mv(p2), 20), pass(vec![]));
    let drop = vec![A::Up(Left, p2), A::KeyUp(CTRL)];
    assert_eq!(on(&mut e, down(p2), 30), eat(drop));
    assert_eq!(on(&mut e, up(p2), 40), eat(vec![]));
    assert!(notes(&mut e).contains(&Note::LatchesUsed));
}

#[test]
fn a_cancel_lets_go_of_the_carry_s_modifiers() {
    let mut e = latched(false, &[CTRL]);
    toggle(&mut e, Toggle::Grab);
    let p0 = p(10, 10);
    on(&mut e, down(p0), 0);
    on(&mut e, up(p0), 10);
    let esc = on(&mut e, PointerEvent::Esc, 20);
    assert_eq!(esc, pass(vec![A::Up(Left, p0), A::KeyUp(CTRL)]));
}

#[test]
fn a_latch_change_during_a_click_keeps_its_modifiers() {
    let mut e = latched(false, &[SHIFT]);
    let p0 = p(1, 1);
    on(&mut e, down(p0), 0);
    e.set_latches(vec![CTRL]);
    assert_eq!(
        on(&mut e, up(p0), 10),
        eat(vec![A::Up(Left, p0), A::KeyUp(SHIFT)])
    );
    let next = vec![A::KeyDown(CTRL), A::Down(Left, p0)];
    assert_eq!(on(&mut e, down(p0), 20), eat(next));
}

#[test]
fn a_latch_change_during_a_held_back_click_keeps_its_modifiers() {
    let mut e = latched(true, &[SHIFT]);
    let p0 = p(1, 1);
    on(&mut e, down(p0), 0);
    e.set_latches(vec![]);
    let want = vec![
        A::KeyDown(SHIFT),
        A::Down(Left, p0),
        A::Up(Left, p0),
        A::KeyUp(SHIFT),
    ];
    assert_eq!(on(&mut e, up(p0), 10), eat(want));
}

#[test]
fn several_modifiers_go_down_in_order_and_up_in_reverse() {
    let mut e = latched(false, &[CTRL, SHIFT]);
    let p0 = p(1, 1);
    let press_with = vec![A::KeyDown(CTRL), A::KeyDown(SHIFT), A::Down(Left, p0)];
    assert_eq!(on(&mut e, down(p0), 0), eat(press_with));
    let lift_with = vec![A::Up(Left, p0), A::KeyUp(SHIFT), A::KeyUp(CTRL)];
    assert_eq!(on(&mut e, up(p0), 10), eat(lift_with));
}

#[test]
fn release_cancels_a_carry_with_esc_then_lets_go_of_it_and_its_modifiers() {
    let mut e = latched(false, &[SHIFT]);
    toggle(&mut e, Toggle::Grab);
    let (p0, p3) = (p(10, 10), p(40, 40));
    on(&mut e, down(p0), 0);
    on(&mut e, up(p0), 1);
    on(&mut e, mv(p3), 2);
    let esc = [A::KeyDown(ESC), A::KeyUp(ESC)];
    let want = [&esc[..], &[A::Up(Left, p3), A::KeyUp(SHIFT)]].concat();
    assert_eq!(release(&mut e), want, "never a drop where the pointer is");
    assert_eq!((e.grab(), e.deadline()), (GrabState::Off, None));
    assert_eq!(release(&mut e), vec![]);
}

#[test]
fn release_forgets_a_held_back_press_so_nothing_goes_down() {
    let mut e = latched(true, &[SHIFT]);
    on(&mut e, down(p(10, 10)), 0);
    assert_eq!(release(&mut e), vec![]);
    assert_eq!(e.deadline(), None);
}

#[test]
fn release_during_a_modded_click_lets_go_of_its_modifiers() {
    let mut e = latched(false, &[SHIFT]);
    on(&mut e, down(p(10, 10)), 0);
    assert_eq!(release(&mut e), vec![A::KeyUp(SHIFT)]);
}

#[test]
fn a_missed_release_never_strands_the_last_click_s_modifiers() {
    let mut e = latched(false, &[SHIFT]);
    let p1 = p(40, 40);
    on(&mut e, down(p(10, 10)), 0);
    e.set_latches(vec![CTRL]);
    let want = vec![A::KeyUp(SHIFT), A::KeyDown(CTRL), A::Down(Left, p1)];
    assert_eq!(on(&mut e, down(p1), 10), eat(want));
}
