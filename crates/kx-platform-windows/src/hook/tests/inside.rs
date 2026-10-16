//! Tests of the echoes heard inside our own send: Windows runs our hooks inside `SendInput`, while the host is busy,
//! so the host settles them right after it notes the batch sent.

use kx_module_api::ports::{Button, PointerAct};

use super::fake::Heard;
use super::run::sends;
use super::{A, B, CFG, CTRL, KEY_UP, LEFT_DOWN, LEFT_UP, MOVE, OTHER, Rig, SHIFT, rig};
use crate::hook::Source;
use crate::hook::inbox::{ECHOES_PER_ACT, Inbox};
use crate::keys::{SIM_TAG, TAG, replay_tag};

/// An id no act out ever has in these tests, which send only a few acts from 0.
const STRAY: u16 = u16::MAX / 2;
const LEFT_A: PointerAct = PointerAct::Down(Button::Left, A);

/// A rig whose sends echo the acts they take inside the call, as Windows does.
fn inside() -> Rig {
    let rig = rig(Source::Physical);
    rig.world.inside.set(true);
    rig
}

impl Rig {
    /// A turn whose settle sends `acts`, with the calls and what the engine heard forgotten.
    fn settle(&mut self, acts: &[PointerAct]) {
        self.plan().settle_acts = acts.to_vec();
        self.host.turn();
        self.plan().settle_acts.clear();
        self.world.take();
        self.heard();
    }

    /// The Ups and key-ups our input sent since the last look.
    fn ups(&self) -> usize {
        let sent = sends(&self.world.take()).concat();
        sent.iter()
            .filter(|act| matches!(act, PointerAct::Up(..) | PointerAct::KeyUp(_)))
            .count()
    }
}

#[test]
fn a_batch_echoed_inside_our_send_leaves_no_wait() {
    let mut rig = inside();
    rig.settle(&[LEFT_A, PointerAct::Up(Button::Left, A)]);
    assert!(rig.host.is_clear());
    assert!(!rig.hand(MOVE, B), "not queued behind our click");
    assert!(rig.world.take().is_empty(), "nothing waits, so no wake");
    assert_eq!(rig.stats().lost_echoes, 0);
}

#[test]
fn a_held_drag_echoed_inside_our_send_lets_the_hands_up_pass_with_no_owed_up() {
    let mut rig = inside();
    rig.plan().drop = true;
    rig.plan().on_acts = vec![LEFT_A, PointerAct::Move(B)];
    assert!(rig.hand(LEFT_DOWN, A), "the engine holds the press back");
    (rig.plan().drop, rig.plan().on_acts) = (false, Vec::new());
    rig.host.turn();
    assert!(
        rig.host.is_clear(),
        "the Down and the Move came back inside the send"
    );
    rig.world.take();
    rig.world.pass(CFG.echo_ms + 1);
    assert!(!rig.hand(LEFT_UP, B), "the hand's own Up passes");
    rig.host.turn();
    rig.host.stop();
    assert_eq!(rig.ups(), 0, "the hand's Up is the only one");
    assert_eq!(rig.stats().lost_echoes, 0);
}

#[test]
fn echoes_in_windows_order_put_no_press_in_doubt() {
    let mut rig = inside();
    rig.settle(&[PointerAct::KeyDown(CTRL), LEFT_A]);
    let hand = rig.mark(false, 0);
    rig.host.key(KEY_UP, 0, CTRL, hand);
    assert!(!rig.hand(LEFT_UP, B));
    rig.host.turn();
    rig.host.stop();
    assert_eq!(rig.ups(), 0, "the hand let both go, and no Up is owed");
}

#[test]
fn a_replayed_key_down_and_up_echoed_inside_our_send_settle() {
    let mut rig = inside();
    rig.settle(&[PointerAct::KeyDown(SHIFT), PointerAct::KeyUp(SHIFT)]);
    assert!(rig.host.is_clear());
    assert!(!rig.hand(MOVE, A), "not queued behind our keys");
    assert_eq!(rig.stats().lost_echoes, 0);
}

#[test]
fn an_inbox_echo_whose_id_is_not_out_changes_nothing() {
    let mut rig = rig(Source::Physical);
    rig.world.strays.replace(vec![1, u16::MAX, STRAY]);
    rig.settle(&[LEFT_A]);
    assert!(!rig.host.is_clear());
    assert!(rig.hand(MOVE, B), "still behind our press");
    let mut rig = inside();
    rig.world.strays.replace(vec![1, u16::MAX, STRAY]);
    rig.settle(&[LEFT_A]);
    assert!(!rig.hand(MOVE, B), "its own echo still settles it");
}

#[test]
fn a_full_inbox_drops_the_rest_keeps_its_room_and_the_wait_ends_lost() {
    let mut rig = inside();
    let room = rig.host.room();
    assert_eq!(
        room.3,
        room.2 * ECHOES_PER_ACT,
        "a batch's acts, a move and a button each"
    );
    rig.world.strays.replace(vec![STRAY; room.3]);
    rig.settle(&[LEFT_A]);
    assert!(!rig.host.is_clear(), "its own echoes were dropped");
    assert_eq!(rig.host.room(), room);
    rig.world.pass(CFG.echo_ms + 1);
    assert!(!rig.hand(MOVE, B), "the wait ends at echo_ms");
    assert_eq!(rig.stats().lost_echoes, 1);
    rig.world.strays.replace(Vec::new());
    rig.settle(&[PointerAct::Move(A)]);
    assert!(!rig.hand(MOVE, B), "the next send's echoes fit again");
    assert_eq!(rig.stats().lost_echoes, 1);
}

#[test]
fn a_send_leaves_the_newest_point_at_its_newest_pointed_act_not_at_an_echo() {
    let mut rig = inside();
    rig.settle(&[LEFT_A, PointerAct::Move(B)]);
    rig.host.stop();
    let want = vec![PointerAct::Up(Button::Left, B)];
    assert_eq!(
        sends(&rig.world.take()),
        [want],
        "let go where the pointer went, though Down's echo came last"
    );
}

#[test]
fn a_late_echo_of_ours_after_the_send_keeps_the_newest_point() {
    let mut rig = rig(Source::Physical);
    rig.settle(&[LEFT_A, PointerAct::Move(B)]);
    rig.echo(&[LEFT_A]);
    rig.host.stop();
    let want = vec![PointerAct::Up(Button::Left, B)];
    assert_eq!(
        sends(&rig.world.take()),
        [want],
        "the Down's late echo at A does not move the pointer back"
    );
}

#[test]
fn a_panic_inside_our_send_leaves_no_stale_echo_for_the_next_batch() {
    let mut rig = rig(Source::Physical);
    rig.world.strays.replace(vec![0]);
    rig.world.panic_send.set(true);
    rig.settle(&[LEFT_A]);
    rig.world.strays.replace(Vec::new());
    rig.world.panic_send.set(false);
    rig.plan().release_acts = vec![PointerAct::Move(B)];
    rig.host.turn();
    assert!(
        !rig.host.is_clear(),
        "the next batch takes id 0 again, and only its own echo settles it"
    );
}

#[test]
fn the_hands_events_heard_inside_our_send_pass_without_the_engine_and_are_counted() {
    let mut rig = inside();
    rig.world.hand_inside.set(2);
    rig.plan().settle_acts = vec![LEFT_A];
    rig.host.turn();
    assert_eq!(
        rig.heard(),
        [Heard::Settle(rig.now())],
        "the engine heard none"
    );
    assert_eq!(rig.stats().passed_inside, 2);
    rig.host.turn();
    assert_eq!(rig.stats().passed_inside, 4, "each send adds its own");
    rig.plan().settle_acts.clear();
    rig.host.turn();
    assert_eq!(
        rig.stats().passed_inside,
        4,
        "no send, nothing heard inside"
    );
}

#[test]
fn a_dropped_echo_inside_our_send_ends_its_wait_as_lost_unless_a_newer_one_came() {
    let mut rig = inside();
    rig.world.lose.set(Some(0));
    rig.settle(&[PointerAct::Move(A)]);
    assert!(!rig.host.is_clear(), "its only echo was dropped");
    rig.world.pass(CFG.echo_ms + 1);
    assert!(!rig.hand(MOVE, B), "the wait ends at echo_ms");
    assert_eq!(rig.stats().lost_echoes, 1);
    rig.world.lose.set(Some(0));
    rig.settle(&[LEFT_A, PointerAct::Move(B)]);
    assert!(rig.host.is_clear(), "a newer echo of the batch came");
    assert_eq!(rig.stats().lost_echoes, 1);
}

#[test]
fn the_inbox_counts_only_the_users_events_for_its_source() {
    let mut hand = Inbox::new(1, Source::Physical);
    hand.user(false, 0);
    for extra in [OTHER, replay_tag(1), TAG, SIM_TAG] {
        hand.user(true, extra);
    }
    assert_eq!(hand.take_passed(), 1, "only the hand's");
    assert_eq!(hand.take_passed(), 0, "taken");
    let mut sim = Inbox::new(1, Source::Simulated);
    sim.user(false, 0);
    sim.user(true, SIM_TAG);
    assert_eq!(sim.take_passed(), 1, "only the gate tool's");
    let mut none = Inbox::none();
    none.user(false, 0);
    assert_eq!(none.take_passed(), 0, "nothing lent counts nothing");
}

#[test]
fn the_inbox_keeps_only_our_echoes_within_its_room() {
    let mut inbox = Inbox::new(1, Source::Physical);
    assert!(!inbox.hear(TAG), "our keyboard's input is not an echo");
    for id in 0..3 {
        let kept = inbox.hear(replay_tag(id));
        assert_eq!(kept, usize::from(id) < ECHOES_PER_ACT, "{id}");
    }
    let ids: Vec<_> = inbox.drain().collect();
    assert_eq!(ids, [0, 1], "oldest first");
    assert_eq!(inbox.room(), ECHOES_PER_ACT, "drained, the room stays");
    inbox.hear(replay_tag(5));
    inbox.clear();
    assert_eq!(inbox.drain().count(), 0);
    assert_eq!(inbox.room(), ECHOES_PER_ACT, "cleared, the room stays");
    assert!(
        !Inbox::none().hear(replay_tag(0)),
        "nothing lent keeps nothing"
    );
}
