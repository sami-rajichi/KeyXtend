//! Tests of the host's limits: the room reserved once, a full queue, a greedy engine and one that panics.

use kx_module_api::ports::{Button, Point, PointerAct};

use super::fake::{Call, Heard, MAX_ACTS};
use super::run::sends;
use super::{A, B, LEFT_DOWN, LEFT_UP, MOVE, Rig, SHIFT, rig};
use crate::hook::Source;
use crate::hook::queue::QUEUE_ROOM;

/// A rig behind its own input whose engine adds a key to every event, so each move is kept apart and fills the queue.
fn crowding() -> Rig {
    let mut rig = rig(Source::Physical);
    rig.send_ours();
    rig.plan().on_acts = vec![PointerAct::KeyDown(SHIFT)];
    rig
}

#[test]
fn the_lent_buffer_and_the_queue_keep_the_room_reserved_at_the_start() {
    let mut rig = crowding();
    let room = rig.host.room();
    assert_eq!(room.0, MAX_ACTS);
    assert!(room.1 >= QUEUE_ROOM + MAX_ACTS);
    rig.plan().on_acts = vec![PointerAct::KeyDown(SHIFT); MAX_ACTS];
    for _ in 0..QUEUE_ROOM {
        rig.hand(MOVE, A);
    }
    assert_eq!(rig.host.room(), room);
}

#[test]
fn a_move_flood_behind_our_input_goes_out_as_its_newest_move() {
    let mut rig = rig(Source::Physical);
    rig.send_ours();
    for x in 0..1_000 {
        assert!(rig.hand(MOVE, Point { x, y: 0 }));
    }
    rig.host.turn();
    let newest = PointerAct::Move(Point { x: 999, y: 0 });
    assert_eq!(sends(&rig.world.take())[0], [newest]);
    assert_eq!(rig.stats().lost_echoes, 0, "no overflow");
}

#[test]
fn a_full_queue_lets_events_pass_without_the_engine_and_lets_go_at_the_next_turn() {
    let mut rig = crowding();
    let mut held = 0;
    while rig.hand(MOVE, A) {
        held += 1;
    }
    assert!(held >= QUEUE_ROOM / 2, "{held}");
    assert_eq!(
        rig.heard().len(),
        held,
        "the move that found it full skipped the engine"
    );
    assert!(!rig.hand(LEFT_DOWN, B), "passes in order");
    assert!(rig.heard().is_empty());
    assert_eq!(rig.stats().lost_echoes, 1, "once per overflow");
    rig.plan().release_acts = vec![PointerAct::Up(Button::Left, B)];
    rig.world.take();
    rig.host.turn();
    assert!(rig.heard().contains(&Heard::Release), "its view is stale");
    let sent = sends(&rig.world.take());
    assert_eq!(sent.len(), 2, "the queue, then the release");
    assert_eq!(sent[1], [PointerAct::Up(Button::Left, B)]);
}

#[test]
fn a_queued_press_whose_release_passed_in_an_overflow_ends_released() {
    let mut rig = crowding();
    assert!(rig.hand(LEFT_DOWN, A), "queued behind our input");
    while rig.hand(MOVE, A) {}
    assert!(!rig.hand(LEFT_UP, B), "the queue is full, so it passes");
    rig.world.take();
    rig.host.turn();
    let sent = sends(&rig.world.take());
    assert_eq!(sent[0][0], PointerAct::Down(Button::Left, A));
    assert_eq!(sent[1], [PointerAct::Up(Button::Left, B)], "right after it");
}

#[test]
fn a_queued_press_whose_release_passed_a_broken_engine_ends_released() {
    let mut rig = rig(Source::Physical);
    rig.send_ours();
    assert!(rig.hand(LEFT_DOWN, A));
    rig.plan().panic = true;
    assert!(!rig.hand(MOVE, B));
    assert!(!rig.hand(LEFT_UP, B), "a broken engine lets it pass");
    rig.world.take();
    rig.host.turn();
    let sent = sends(&rig.world.take());
    assert_eq!(sent[0], [PointerAct::Down(Button::Left, A)]);
    assert_eq!(sent[1], [PointerAct::Up(Button::Left, B)]);
}

#[test]
fn an_engine_that_pushes_past_max_acts_is_cut_and_counted_and_the_queue_never_grows() {
    let mut rig = rig(Source::Physical);
    let room = rig.host.room();
    let extra = 3;
    rig.plan().on_acts = vec![PointerAct::KeyDown(SHIFT); MAX_ACTS + extra];
    // A debug build also treats this as the engine's panic, after the cut.
    rig.hand(LEFT_DOWN, A);
    assert_eq!(rig.stats.overruns(), u32::try_from(extra).unwrap());
    assert_eq!(rig.host.room().1, room.1, "the queue never grew");
    rig.host.turn();
    assert!(sends(&rig.world.take()).iter().all(|s| s.len() <= MAX_ACTS));
}

#[test]
fn a_panicking_engine_passes_the_event_and_gets_one_release_at_once() {
    let mut rig = rig(Source::Physical);
    rig.plan().panic = true;
    rig.plan().drop = true;
    assert!(!rig.hand(LEFT_DOWN, A), "a panic never drops the event");
    assert!(rig.world.take().contains(&Call::Wake), "a turn comes soon");
    rig.host.turn();
    rig.host.turn();
    let releases = rig.heard().iter().filter(|h| **h == Heard::Release).count();
    assert_eq!(releases, 1);
    assert!(!rig.hand(MOVE, A));
    rig.host.stop();
    assert!(rig.heard().is_empty(), "a broken engine hears no more");
    assert_eq!(rig.world.take().last(), Some(&Call::Unhook));
}
