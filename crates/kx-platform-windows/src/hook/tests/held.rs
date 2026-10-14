//! Tests of the host's own record of what its sent input holds down, and of what its records show in `Debug`.

use kx_module_api::REDACTED_MARKER;
use kx_module_api::ports::{Button, MAX_HELD_KEYS, Point, PointerAct, Scan};

use crate::hook::accept::Kind;
use crate::hook::batch::{Desk, batch, normalise};
use crate::hook::held::Held;
use crate::hook::queue::Queue;

/// Left Ctrl and Left Shift.
const CTRL: Scan = Scan::new(0x1D);
const SHIFT: Scan = Scan::new(0x2A);
const A: Point = Point { x: 1, y: 2 };
const B: Point = Point { x: 3, y: 4 };
/// A click point whose numbers stand out in any text.
const SECRET: Point = Point { x: 1234, y: 5678 };
/// A desktop that holds `SECRET`.
const DESK: Desk = Desk {
    left: 0,
    top: 0,
    width: 4000,
    height: 6000,
};

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
    held.forget(PointerAct::Up(Button::Left, A));
    held.forget(PointerAct::KeyUp(CTRL));
    held.forget(PointerAct::KeyUp(Scan::new(0x38)));
    held.forget(PointerAct::KeyDown(SHIFT));
    assert_eq!(ups(&held, None), [PointerAct::KeyUp(SHIFT)]);
}

#[test]
fn the_records_and_a_batch_hide_every_point_and_key_in_debug() {
    let (press, key) = (
        PointerAct::Down(Button::Left, SECRET),
        PointerAct::KeyDown(CTRL),
    );
    let mut held = Held::default();
    held.sent(&[press, key]);
    let mut queue = Queue::new(1);
    assert!(queue.put(press));
    queue.passed(Kind::Up(Button::Left), SECRET);
    let inputs = batch(&[press, key], DESK);
    let text = format!("{held:?} {queue:?} {inputs:?}");
    assert!(text.contains(REDACTED_MARKER), "{text}");
    let dx = normalise(SECRET.x, DESK.left, DESK.width).to_string();
    for shown in ["1234", "5678", &dx, "Scan("] {
        assert!(!text.contains(shown), "{shown} in {text}");
    }
}
