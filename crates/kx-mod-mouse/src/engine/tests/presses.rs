//! Presses wanted by an asker or an open panel (spec §6, ADR-0041 decision 6): the hook stays on and each press and
//! release is noted with `ours`, while every event passes as the modes and latches say.

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

/// A release note.
fn released(ours: bool) -> Note {
    Note::Released { ours }
}

/// A left release on one of our windows.
fn up_ours(at: Point) -> PointerEvent {
    PointerEvent::Up {
        button: Left,
        at,
        ours: true,
    }
}

#[test]
fn wanted_alone_needs_the_hook_notes_each_press_and_release_and_passes_every_event() {
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
    let want = vec![pressed(false), released(false), pressed(true)];
    assert_eq!(got, want, "no move is noted");
    e.set_wanted(false);
    assert!(!e.needs_hook(), "no longer wanted");
}

#[test]
fn unwanted_presses_are_not_noted_and_a_mode_keeps_the_hook() {
    let mut e = wanted(true);
    e.set_wanted(false);
    assert!(e.needs_hook(), "Right-click still needs it");
    on(&mut e, ours(p(2, 2)), 0);
    on(&mut e, up_ours(p(2, 2)), 1);
    let got = notes(&mut e);
    assert!(!got.contains(&pressed(true)), "{got:?}");
    assert!(!got.contains(&released(true)), "{got:?}");
}

#[test]
fn scrolling_or_wanted_alone_keeps_presses_noted_and_only_wanted_notes_releases() {
    let mut e = wanted(false);
    e.set_scrolling(true);
    e.set_wanted(false);
    on(&mut e, down(p(3, 3)), 0);
    on(&mut e, up(p(3, 3)), 1);
    assert_eq!(
        notes(&mut e),
        vec![pressed(false)],
        "scrolling holds presses"
    );
    e.set_wanted(true);
    e.set_scrolling(false);
    on(&mut e, down(p(3, 3)), 2);
    on(&mut e, up(p(3, 3)), 3);
    let both = vec![pressed(false), released(false)];
    assert_eq!(notes(&mut e), both, "wanted holds both");
    assert!(e.needs_hook());
}

#[test]
fn each_release_of_any_button_is_noted_with_where_it_landed() {
    let mut e = wanted(false);
    let p0 = p(5, 5);
    on(&mut e, ours(p0), 0);
    assert_eq!(on(&mut e, up_ours(p0), 1), pass(vec![]));
    on(&mut e, press(Right, p0, DPI), 2);
    assert_eq!(on(&mut e, lift(Right, p0), 3), pass(vec![]));
    let want = vec![
        pressed(true),
        released(true),
        pressed(false),
        released(false),
    ];
    assert_eq!(notes(&mut e), want);
}

#[test]
fn a_held_back_press_is_noted_once_and_fires_and_its_dropped_release_is_noted() {
    let mut e = wanted(true);
    let p0 = p(10, 10);
    assert_eq!(on(&mut e, down(p0), 0), eat(vec![]), "held as without it");
    let start = Note::HoldStart { at: p0 };
    assert_eq!(notes(&mut e), vec![pressed(false), start]);
    assert_eq!(tick(&mut e, HOLD), right_click(p0));
    assert_eq!(on(&mut e, up(p0), HOLD + 1), eat(vec![]), "dropped");
    assert_eq!(notes(&mut e), vec![Note::Fired { at: p0 }, released(false)]);
}

#[test]
fn a_release_note_shows_only_where_it_landed() {
    let text = format!("{:?}", released(true));
    assert_eq!(text, "Released { ours: true }");
}
