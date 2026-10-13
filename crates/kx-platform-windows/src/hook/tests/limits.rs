//! Tests of the host's limits: the room reserved once, a full queue, and an engine that panics.

use kx_module_api::ports::{Button, PointerAct};

use super::fake::{Call, Heard, MAX_ACTS};
use super::run::sends;
use super::{A, B, LEFT_DOWN, MOVE, rig};
use crate::hook::Source;
use crate::hook::queue::QUEUE_ROOM;

#[test]
fn the_lent_buffer_and_the_queue_keep_the_room_reserved_at_the_start() {
    let mut rig = rig(Source::Physical);
    let room = rig.host.room();
    assert_eq!(room.0, MAX_ACTS);
    assert!(room.1 >= QUEUE_ROOM + MAX_ACTS);
    rig.send_ours();
    rig.plan().on_acts = vec![PointerAct::Move(A); MAX_ACTS];
    for _ in 0..QUEUE_ROOM {
        rig.hand(MOVE, A);
    }
    assert_eq!(rig.host.room(), room);
}

#[test]
fn a_full_queue_lets_events_pass_without_the_engine_and_lets_go_at_the_next_turn() {
    let mut rig = rig(Source::Physical);
    rig.send_ours();
    let mut held = 0;
    while rig.hand(MOVE, A) {
        held += 1;
    }
    assert!(held >= QUEUE_ROOM, "{held}");
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
fn a_panicking_engine_passes_the_event_and_gets_one_release() {
    let mut rig = rig(Source::Physical);
    rig.plan().panic = true;
    rig.plan().drop = true;
    assert!(!rig.hand(LEFT_DOWN, A), "a panic never drops the event");
    rig.host.turn();
    rig.host.turn();
    let releases = rig.heard().iter().filter(|h| **h == Heard::Release).count();
    assert_eq!(releases, 1);
    assert!(!rig.hand(MOVE, A));
    rig.host.stop();
    assert!(rig.heard().is_empty(), "a broken engine hears no more");
    assert_eq!(rig.world.take().last(), Some(&Call::Unhook));
}
