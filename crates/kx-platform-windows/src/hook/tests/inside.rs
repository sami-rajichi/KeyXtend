//! Tests of the echoes heard inside our own send: Windows runs our hooks inside `SendInput`, while the host is busy,
//! so the host settles them right after it notes the batch sent.

use kx_module_api::ports::{Button, Point, PointerAct};

use super::run::sends;
use super::{A, B, CFG, CTRL, KEY_UP, LEFT_DOWN, LEFT_UP, MOVE, Rig, SHIFT, rig};
use crate::hook::Source;
use crate::hook::inbox::{ECHOES_PER_ACT, Inbox};
use crate::keys::{TAG, replay_tag};

/// An id no act out ever has in these tests, which send only a few acts from 0.
const STRAY: u16 = u16::MAX / 2;
const LEFT_A: PointerAct = PointerAct::Down(Button::Left, A);
/// A point whose numbers stand out in debug output.
const FAR: Point = Point { x: 4_321, y: 8_765 };

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
fn a_mouse_echo_inside_our_send_moves_the_newest_point() {
    let mut rig = rig(Source::Physical);
    rig.settle(&[LEFT_A]);
    rig.echo(&[LEFT_A]);
    rig.world.inside.set(true);
    rig.settle(&[PointerAct::Move(B)]);
    rig.host.stop();
    let want = vec![PointerAct::Up(Button::Left, B)];
    assert_eq!(
        sends(&rig.world.take()),
        [want],
        "let go where the pointer is"
    );
}

#[test]
fn the_inbox_keeps_only_our_echoes_within_its_room() {
    let mut inbox = Inbox::new(1);
    assert!(
        !inbox.hear(TAG, None),
        "our keyboard's input is not an echo"
    );
    for id in 0..3 {
        let kept = inbox.hear(replay_tag(id), Some(A));
        assert_eq!(kept, usize::from(id) < ECHOES_PER_ACT, "{id}");
    }
    assert_eq!(inbox.room(), ECHOES_PER_ACT);
    let ids: Vec<_> = inbox.drain().map(|e| e.id).collect();
    assert_eq!(ids, [0, 1], "oldest first");
    assert_eq!(inbox.room(), ECHOES_PER_ACT, "drained, the room stays");
    assert!(
        !Inbox::none().hear(replay_tag(0), None),
        "nothing lent keeps nothing"
    );
    inbox.hear(replay_tag(0), Some(FAR));
    let shown = format!("{inbox:?}");
    assert!(
        !shown.contains("4321") && !shown.contains("8765"),
        "{shown}"
    );
}
