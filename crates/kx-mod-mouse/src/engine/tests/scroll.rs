//! Scrolling (spec §5.2): the hook stays on, moves are noted for the badge and each physical press is noted for the
//! scroll's stop, while every mode works as before.

use super::*;
use Button::Middle;

/// An engine that scrolls, with Right-click as given.
fn scrolling(rclick: bool) -> Engine {
    let mut e = eng(rclick);
    e.set_scrolling(true);
    e
}

/// A press note.
fn pressed(ours: bool) -> Note {
    Note::Pressed { ours }
}

#[test]
fn scrolling_alone_needs_the_hook_and_passes_every_event_unchanged() {
    let mut e = scrolling(false);
    assert!(e.needs_hook());
    let p0 = p(1, 1);
    assert_eq!(on(&mut e, down(p0), 0), pass(vec![]));
    assert_eq!(e.deadline(), None, "no press is held back");
    assert_eq!(on(&mut e, mv(p(50, 50)), 10), pass(vec![]));
    assert_eq!(on(&mut e, up(p0), HOLD * 2), pass(vec![]));
    assert_eq!(on(&mut e, press(Right, p0, DPI), HOLD * 2), pass(vec![]));
    assert_eq!(on(&mut e, PointerEvent::Esc, HOLD * 2), pass(vec![]));
    assert_eq!(tick(&mut e, HOLD * 3), vec![]);
    e.set_scrolling(false);
    assert!(!e.needs_hook(), "scrolling ended");
}

#[test]
fn while_scrolling_moves_are_noted_and_each_press_says_whether_it_was_ours() {
    let mut e = scrolling(false);
    on(&mut e, mv(p(2, 2)), 0);
    on(&mut e, mv(p(3, 3)), 1);
    on(&mut e, down(p(3, 3)), 2);
    on(&mut e, up(p(3, 3)), 3);
    on(&mut e, ours(p(4, 4)), 4);
    on(&mut e, press(Middle, p(4, 4), DPI), 5);
    let want = vec![
        Note::Moved { at: p(3, 3) },
        pressed(false),
        pressed(true),
        pressed(false),
    ];
    assert_eq!(notes(&mut e), want);
}

#[test]
fn no_press_is_noted_while_not_scrolling_and_no_move_with_no_mode_either() {
    let mut e = eng(false);
    e.set_latches(vec![SHIFT]);
    on(&mut e, mv(p(2, 2)), 0);
    on(&mut e, down(p(2, 2)), 1);
    on(&mut e, up(p(2, 2)), 2);
    let got = notes(&mut e);
    let seen = |n: &Note| matches!(n, Note::Pressed { .. } | Note::Moved { .. });
    assert!(!got.iter().any(seen), "{got:?}");
    let mut e = scrolling(true);
    e.set_scrolling(false);
    on(&mut e, down(p(2, 2)), 0);
    let got = notes(&mut e);
    assert!(!got.contains(&pressed(false)), "{got:?}");
}

#[test]
fn a_held_back_right_click_press_is_noted_once_at_its_physical_down() {
    let mut e = scrolling(true);
    let p0 = p(10, 10);
    assert_eq!(on(&mut e, down(p0), 0), eat(vec![]));
    let start = Note::HoldStart { at: p0 };
    assert_eq!(notes(&mut e), vec![pressed(false), start]);
    assert_eq!(tick(&mut e, HOLD), right_click(p0), "the hold still fires");
    assert_eq!(notes(&mut e), vec![Note::Fired { at: p0 }]);
    on(&mut e, up(p0), HOLD + 1);
    assert_eq!(notes(&mut e), vec![]);
}

#[test]
fn grab_while_scrolling_picks_up_and_drops_and_notes_each_press() {
    let mut e = scrolling(false);
    toggle(&mut e, Toggle::Grab);
    notes(&mut e);
    let (p0, p1) = (p(1, 1), p(9, 9));
    on(&mut e, down(p0), 0);
    on(&mut e, up(p0), 1);
    let (dropped, acts) = on(&mut e, down(p1), 2);
    assert!(dropped);
    assert_eq!(acts, vec![A::Up(Left, p1)], "the drop");
    let got = notes(&mut e);
    let presses = got.iter().filter(|n| **n == pressed(false)).count();
    assert_eq!(presses, 2, "{got:?}");
    assert!(got.contains(&Note::Picked { at: p0 }), "{got:?}");
}

#[test]
fn a_mode_turned_off_while_scrolling_keeps_the_hook() {
    let mut e = scrolling(true);
    toggle(&mut e, Toggle::RightClick);
    assert!(e.needs_hook(), "scrolling still needs it");
    e.set_scrolling(false);
    assert!(!e.needs_hook());
}

#[test]
fn a_press_note_s_debug_says_only_whether_it_was_ours() {
    assert_eq!(format!("{:?}", pressed(true)), "Pressed { ours: true }");
}
