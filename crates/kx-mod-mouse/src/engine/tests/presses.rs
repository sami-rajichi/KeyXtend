//! Presses wanted by word prediction (spec §6): the hook stays on and each press is noted with `ours`, while every
//! event passes as the modes and latches say.

use super::*;

/// An engine with presses wanted, with Right-click as given.
fn wanted(rclick: bool) -> Engine {
    let mut e = eng(rclick);
    e.set_wanted(true);
    e
}

/// A press note.
fn pressed(ours: bool) -> Note {
    Note::Pressed { ours }
}

#[test]
fn wanted_alone_needs_the_hook_notes_each_press_and_passes_every_event() {
    let mut e = wanted(false);
    assert!(e.needs_hook());
    let p0 = p(1, 1);
    assert_eq!(on(&mut e, mv(p(50, 50)), 0), pass(vec![]));
    assert_eq!(on(&mut e, down(p0), 1), pass(vec![]));
    assert_eq!(e.deadline(), None, "no press is held back");
    assert_eq!(on(&mut e, up(p0), HOLD * 2), pass(vec![]));
    assert_eq!(on(&mut e, ours(p0), HOLD * 2), pass(vec![]));
    assert_eq!(tick(&mut e, HOLD * 3), vec![]);
    let got = notes(&mut e);
    assert_eq!(got, vec![pressed(false), pressed(true)], "no move is noted");
    e.set_wanted(false);
    assert!(!e.needs_hook(), "no longer wanted");
}

#[test]
fn unwanted_presses_are_not_noted_and_a_mode_keeps_the_hook() {
    let mut e = wanted(true);
    e.set_wanted(false);
    assert!(e.needs_hook(), "Right-click still needs it");
    on(&mut e, ours(p(2, 2)), 0);
    let got = notes(&mut e);
    assert!(!got.contains(&pressed(true)), "{got:?}");
}

#[test]
fn scrolling_or_wanted_alone_keeps_presses_noted() {
    let mut e = wanted(false);
    e.set_scrolling(true);
    e.set_wanted(false);
    on(&mut e, down(p(3, 3)), 0);
    assert_eq!(notes(&mut e), vec![pressed(false)], "scrolling holds");
    e.set_wanted(true);
    e.set_scrolling(false);
    on(&mut e, up(p(3, 3)), 1);
    on(&mut e, down(p(3, 3)), 2);
    assert_eq!(notes(&mut e), vec![pressed(false)], "wanted holds");
    assert!(e.needs_hook());
}

#[test]
fn a_held_back_right_click_press_is_noted_once_and_still_fires() {
    let mut e = wanted(true);
    let p0 = p(10, 10);
    assert_eq!(on(&mut e, down(p0), 0), eat(vec![]), "held as without it");
    let start = Note::HoldStart { at: p0 };
    assert_eq!(notes(&mut e), vec![pressed(false), start]);
    assert_eq!(tick(&mut e, HOLD), right_click(p0));
    on(&mut e, up(p0), HOLD + 1);
    assert_eq!(notes(&mut e), vec![Note::Fired { at: p0 }]);
}
