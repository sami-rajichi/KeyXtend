//! Tests of the host's own record of what its sent input holds down.

use kx_module_api::ports::{Button, MAX_HELD_KEYS, Point, PointerAct, Scan};

use crate::hook::held::Held;

/// Left Ctrl and Left Shift.
const CTRL: Scan = Scan::new(0x1D);
const SHIFT: Scan = Scan::new(0x2A);
const A: Point = Point { x: 1, y: 2 };
const B: Point = Point { x: 3, y: 4 };

/// What `held` lets go of at `at`.
fn ups(held: &Held, at: Option<Point>) -> Vec<PointerAct> {
    held.release(at).collect()
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
        ups(&held, Some(B)),
        want,
        "at the newest point, keys newest first"
    );
    assert_eq!(ups(&held, None)[0], PointerAct::Up(Button::Right, A));
    held.sent(&want);
    assert!(ups(&held, Some(B)).is_empty(), "nothing is left");
}

#[test]
fn a_key_pressed_twice_is_held_once_and_the_record_never_grows() {
    let mut held = Held::default();
    held.sent(&[PointerAct::KeyDown(CTRL), PointerAct::KeyDown(CTRL)]);
    assert_eq!(ups(&held, None), [PointerAct::KeyUp(CTRL)]);
    let many: Vec<PointerAct> = (0x70_u16..)
        .take(MAX_HELD_KEYS + 4)
        .map(|code| PointerAct::KeyDown(Scan::new(code)))
        .collect();
    held.sent(&many);
    assert_eq!(ups(&held, None).len(), MAX_HELD_KEYS);
}

#[test]
fn the_hand_letting_go_itself_clears_the_record() {
    let mut held = Held::default();
    held.sent(&[
        PointerAct::KeyDown(CTRL),
        PointerAct::KeyDown(SHIFT),
        PointerAct::Down(Button::Left, A),
    ]);
    held.forget_button(Button::Left);
    held.forget_key(CTRL);
    held.forget_key(Scan::new(0x38));
    assert_eq!(ups(&held, None), [PointerAct::KeyUp(SHIFT)]);
}
