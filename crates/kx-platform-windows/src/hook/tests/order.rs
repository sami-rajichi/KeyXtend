//! Tests of the queue behind our input: moves coalesce, buttons and keys keep a headroom, nothing grows past the room,
//! and an Up that passed ahead of its queued Down follows the queue.

use kx_module_api::ports::{Button, Point, PointerAct, Scan};

use crate::hook::accept::Kind;
use crate::hook::held::SLOTS;
use crate::hook::queue::{HEADROOM, QUEUE_ROOM, Queue};

/// The most acts the engine pushes in one call.
const MAX_ACTS: usize = 3;
/// Left Shift.
const SHIFT: Scan = Scan::new(0x2A);

fn at(x: i32) -> Point {
    Point { x, y: 0 }
}

fn key() -> PointerAct {
    PointerAct::KeyDown(SHIFT)
}

/// A queue whose room is full of moves, each between two keys.
fn crowded() -> Queue {
    let mut q = Queue::new(MAX_ACTS);
    let mut x = 0;
    while q.acts().len() < QUEUE_ROOM {
        assert!(q.put(PointerAct::Move(at(x))));
        assert!(q.put(key()));
        x += 1;
    }
    q
}

#[test]
fn the_queue_reserves_its_room_once_and_is_full_before_a_call_could_overflow_it() {
    let mut q = Queue::new(MAX_ACTS);
    let room = q.room();
    let mut pushed = 0;
    while !q.full() {
        assert!(q.put(key()));
        pushed += 1;
    }
    // Left over: less than one call's room, and an Up per button for `seal`.
    assert!(room - pushed <= MAX_ACTS + SLOTS, "{room} {pushed}");
    let _ = q.extend(&[key(); MAX_ACTS]);
    assert_eq!(q.room(), room, "never reallocated");
    assert!(q.overflow() && !q.overflow(), "counted once");
    q.clear();
    assert!(q.is_empty() && !q.full() && q.overflow(), "a new episode");
}

#[test]
fn a_move_flood_coalesces_into_the_newest_move() {
    let mut q = Queue::new(MAX_ACTS);
    for x in 0..1_000 {
        assert!(q.put(PointerAct::Move(at(x))));
    }
    assert_eq!(q.acts(), [PointerAct::Move(at(999))]);
    assert!(q.put(PointerAct::Down(Button::Left, at(1))));
    assert!(q.put(PointerAct::Move(at(2))));
    assert_eq!(
        q.acts().len(),
        3,
        "a button keeps the moves around it apart"
    );
}

#[test]
fn the_headroom_keeps_a_button_pair_in_order_once_the_room_is_full() {
    let mut q = crowded();
    let room = q.room();
    assert!(!q.put(PointerAct::Move(at(-1))), "no room left for a move");
    let pair = [
        PointerAct::Down(Button::Left, at(5)),
        PointerAct::Up(Button::Left, at(6)),
    ];
    assert_eq!(q.extend(&pair), 0, "nothing cut");
    assert_eq!(q.acts()[QUEUE_ROOM..], pair);
    assert!(!q.full(), "an engine call still fits");
    assert_eq!(q.room(), room);
}

#[test]
fn buttons_and_keys_past_the_reserve_are_cut_and_never_allocated() {
    let mut q = crowded();
    let room = q.room();
    let more = 64;
    let fits = HEADROOM + MAX_ACTS + 1;
    assert_eq!(q.extend(&vec![key(); more]), more - fits);
    assert_eq!(q.acts().len(), QUEUE_ROOM + fits);
    assert!(q.full());
    assert_eq!(q.room(), room);
}

/// A queue holding a left press at `at(1)`.
fn pressed() -> Queue {
    let mut q = Queue::new(MAX_ACTS);
    q.put(PointerAct::Down(Button::Left, at(1)));
    q
}

#[test]
fn an_up_that_passed_ahead_of_its_queued_down_ends_the_queues_batch() {
    let mut q = pressed();
    q.put(PointerAct::Down(Button::Right, at(1)));
    q.passed(Kind::Up(Button::Left), at(7));
    q.passed(Kind::Up(Button::Middle), at(7));
    q.passed(Kind::Move, at(8));
    q.seal();
    let want = [
        PointerAct::Down(Button::Left, at(1)),
        PointerAct::Down(Button::Right, at(1)),
        PointerAct::Up(Button::Left, at(7)),
    ];
    assert_eq!(q.acts(), want);
}

#[test]
fn a_press_after_the_passed_up_or_a_queued_up_needs_no_fix() {
    let mut again = pressed();
    again.passed(Kind::Up(Button::Left), at(7));
    again.passed(Kind::Down(Button::Left), at(8));
    again.seal();
    assert_eq!(again.acts().len(), 1, "pressed again, so the hand holds it");
    let mut lets_go = pressed();
    lets_go.passed(Kind::Up(Button::Left), at(9));
    lets_go.put(PointerAct::Up(Button::Left, at(3)));
    lets_go.seal();
    assert_eq!(lets_go.acts().len(), 2, "the queue lets go itself");
    let mut cleared = pressed();
    cleared.passed(Kind::Up(Button::Left), at(9));
    cleared.clear();
    cleared.put(PointerAct::Down(Button::Left, at(1)));
    cleared.seal();
    assert_eq!(cleared.acts().len(), 1, "a clear forgets what passed");
}

#[test]
fn an_up_that_passed_with_nothing_queued_needs_no_fix() {
    let mut q = Queue::new(MAX_ACTS);
    q.passed(Kind::Up(Button::Left), at(1));
    q.put(PointerAct::Down(Button::Left, at(2)));
    q.seal();
    assert_eq!(q.acts().len(), 1);
}

#[test]
fn a_full_queue_still_has_room_for_every_buttons_up() {
    let mut q = Queue::new(MAX_ACTS);
    let room = q.room();
    let presses = [Button::Left, Button::Right, Button::Middle];
    for button in presses {
        q.put(PointerAct::Down(button, at(1)));
    }
    while q.put(key()) {}
    for button in presses {
        q.passed(Kind::Up(button), at(2));
    }
    q.seal();
    assert!(
        q.acts()
            .ends_with(&presses.map(|b| PointerAct::Up(b, at(2))))
    );
    assert_eq!(q.room(), room, "never reallocated");
}
