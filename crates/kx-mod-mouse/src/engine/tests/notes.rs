//! The notes for each step, when the hook is needed, and the note buffer on the hot path.

use super::*;
use GrabState::{Armed, Carrying, Off};
use kx_module_api::REDACTED_MARKER;

fn state(rclick: bool, grab: GrabState) -> Note {
    Note::State { rclick, grab }
}

#[test]
fn a_hold_notes_its_start_then_its_fire() {
    let mut e = eng(true);
    let p0 = p(10, 10);
    on(&mut e, down(p0), 0);
    assert_eq!(notes(&mut e), vec![Note::HoldStart { at: p0 }]);
    tick(&mut e, HOLD);
    assert_eq!(notes(&mut e), vec![Note::Fired { at: p0 }]);
    on(&mut e, up(p0), HOLD + 1);
    assert_eq!(notes(&mut e), vec![]);
}

#[test]
fn a_click_a_drag_or_a_mode_switch_ends_the_hold() {
    let p0 = p(10, 10);
    let start = Note::HoldStart { at: p0 };
    let mut e = eng(true);
    on(&mut e, down(p0), 0);
    on(&mut e, up(p0), 10);
    assert_eq!(notes(&mut e), vec![start, Note::HoldEnd]);
    on(&mut e, down(p0), 20);
    on(&mut e, mv(p(50, 50)), 30);
    let moved = Note::Moved { at: p(50, 50) };
    assert_eq!(notes(&mut e), vec![start, moved, Note::HoldEnd]);
    on(&mut e, up(p(50, 50)), 40);
    on(&mut e, down(p0), 50);
    toggle(&mut e, Toggle::RightClick);
    let off = state(false, Off);
    assert_eq!(notes(&mut e), vec![start, Note::HoldEnd, off]);
}

#[test]
fn grab_notes_each_step_and_its_state() {
    let mut e = eng(true);
    let (p0, p2) = (p(10, 10), p(300, 200));
    toggle(&mut e, Toggle::Grab);
    assert_eq!(notes(&mut e), vec![state(false, Armed)]);
    on(&mut e, down(p0), 0);
    let picked = Note::Picked { at: p0 };
    assert_eq!(notes(&mut e), vec![picked, state(false, Carrying)]);
    on(&mut e, up(p0), 10);
    on(&mut e, mv(p2), 20);
    assert_eq!(notes(&mut e), vec![Note::Moved { at: p2 }]);
    on(&mut e, down(p2), 30);
    let dropped = Note::Dropped { at: p2 };
    assert_eq!(notes(&mut e), vec![dropped, state(true, Off)]);
}

#[test]
fn a_cancel_notes_cancelled_then_the_state() {
    let mut e = eng(false);
    toggle(&mut e, Toggle::Grab);
    on(&mut e, down(p(1, 1)), 0);
    notes(&mut e);
    on(&mut e, PointerEvent::Esc, 10);
    assert_eq!(notes(&mut e), vec![Note::Cancelled, state(false, Off)]);
}

#[test]
fn latches_used_follows_the_click_s_modifiers() {
    let mut e = eng(true);
    e.set_latches(vec![SHIFT]);
    toggle(&mut e, Toggle::Grab);
    on(&mut e, down(p(1, 1)), 0);
    on(&mut e, up(p(1, 1)), 10);
    on(&mut e, down(p(2, 2)), 20);
    let used = [Note::Dropped { at: p(2, 2) }, Note::LatchesUsed];
    let all = notes(&mut e);
    assert_eq!(all[all.len() - 3..all.len() - 1], used[..]);
}

#[test]
fn moves_are_noted_only_while_a_mode_is_on_and_coalesce() {
    let mut e = eng(false);
    e.set_latches(vec![SHIFT]);
    on(&mut e, mv(p(1, 1)), 0);
    assert_eq!(notes(&mut e), vec![]);
    toggle(&mut e, Toggle::RightClick);
    on(&mut e, mv(p(2, 2)), 10);
    on(&mut e, mv(p(3, 3)), 20);
    let want = vec![state(true, Off), Note::Moved { at: p(3, 3) }];
    assert_eq!(notes(&mut e), want);
}

#[test]
fn needs_hook_is_false_only_when_everything_is_idle() {
    let mut e = eng(false);
    let p0 = p(1, 1);
    assert!(!e.needs_hook());
    e.set_latches(vec![SHIFT]);
    assert!(e.needs_hook());
    on(&mut e, down(p0), 0);
    e.set_latches(vec![]);
    assert!(e.needs_hook(), "modifiers still pressed");
    on(&mut e, up(p0), 10);
    assert!(!e.needs_hook());
    toggle(&mut e, Toggle::Grab);
    assert!(e.needs_hook());
    on(&mut e, down(p0), 20);
    toggle(&mut e, Toggle::Grab);
    assert!(e.needs_hook(), "the cancelled press's release is awaited");
    on(&mut e, up(p0), 30);
    assert!(!e.needs_hook());
    toggle(&mut e, Toggle::RightClick);
    assert!(e.needs_hook());
    on(&mut e, down(p0), 40);
    toggle(&mut e, Toggle::RightClick);
    assert!(!e.needs_hook(), "the replayed press needs nothing more");
}

#[test]
fn the_hot_path_never_grows_its_buffers() {
    let mut e = eng(true);
    e.set_latches((0x70_u16..).take(MAX_LATCHES).map(Scan::new).collect());
    let (room, mods_room) = (e.notes.capacity(), e.mods.capacity());
    for i in 0..1_000_u64 {
        let at = p(i32::try_from(i % 97).unwrap(), 5);
        let t = i * HOLD;
        on(&mut e, down(at), t);
        on(&mut e, mv(p(at.x + 20, 5)), t + 1);
        tick(&mut e, t + HOLD / 2);
        on(&mut e, up(at), t + HOLD / 2 + 1);
        if i % 5 == 0 {
            toggle(&mut e, Toggle::Grab);
        }
    }
    assert_eq!((e.notes.capacity(), e.mods.capacity()), (room, mods_room));
    assert!(e.notes.len() <= room);
}

#[test]
fn the_worst_call_pushes_exactly_max_acts() {
    let mut e = eng(true);
    e.set_latches((0x70_u16..).take(MAX_LATCHES).map(Scan::new).collect());
    on(&mut e, down(p(1, 1)), 0);
    let (dropped, acts) = on(&mut e, press(Right, p(2, 2), DPI), HOLD);
    assert!(dropped);
    assert_eq!(acts.len(), MAX_ACTS);
}

#[test]
fn a_note_s_debug_hides_its_point() {
    let at = p(1234, 5678);
    for note in [
        Note::HoldStart { at },
        Note::Moved { at },
        Note::Dropped { at },
    ] {
        let text = format!("{note:?}");
        assert!(text.contains(REDACTED_MARKER), "{text}");
        assert!(!text.contains("1234") && !text.contains("5678"), "{text}");
    }
}
