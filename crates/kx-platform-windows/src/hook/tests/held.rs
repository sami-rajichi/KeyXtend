//! Tests of the host's own record of what its sent input holds down.

use kx_module_api::ports::{Button, Point, PointerAct, Scan};

use crate::hook::held::{HELD_KEYS, Held, SLOTS, slot};

/// Left Ctrl and Left Shift.
const CTRL: Scan = Scan::new(0x1D);
const SHIFT: Scan = Scan::new(0x2A);
const A: Point = Point { x: 1, y: 2 };
const B: Point = Point { x: 3, y: 4 };

#[test]
fn every_button_has_its_own_slot() {
    let slots = [Button::Left, Button::Right, Button::Middle].map(slot);
    assert!(slots.iter().all(|&s| s < SLOTS));
    assert_eq!(slots, [0, 1, 2]);
}

#[test]
fn the_record_lets_go_of_what_sent_input_still_holds_buttons_first() {
    let mut held = Held::default();
    held.sent(&[
        PointerAct::KeyDown(CTRL),
        PointerAct::KeyDown(SHIFT),
        PointerAct::Down(Button::Left, A),
        PointerAct::Up(Button::Left, A),
        PointerAct::Down(Button::Right, A),
    ]);
    let want = [
        PointerAct::Up(Button::Right, B),
        PointerAct::KeyUp(SHIFT),
        PointerAct::KeyUp(CTRL),
    ];
    assert_eq!(
        held.release(Some(B)),
        want,
        "at the newest point, keys newest first"
    );
    assert_eq!(held.release(None)[0], PointerAct::Up(Button::Right, A));
    held.sent(&want);
    assert!(held.release(Some(B)).is_empty(), "nothing is left");
}

#[test]
fn a_key_pressed_twice_is_held_once_and_the_record_never_grows() {
    let mut held = Held::default();
    held.sent(&[PointerAct::KeyDown(CTRL), PointerAct::KeyDown(CTRL)]);
    assert_eq!(held.release(None), [PointerAct::KeyUp(CTRL)]);
    let many: Vec<PointerAct> = (0x70_u16..)
        .take(HELD_KEYS + 4)
        .map(|code| PointerAct::KeyDown(Scan::new(code)))
        .collect();
    held.sent(&many);
    assert_eq!(held.release(None).len(), HELD_KEYS);
}
