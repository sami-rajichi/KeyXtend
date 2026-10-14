//! Tests of the record of held input against the hand's own Ups, which may pass ahead of our press, and of reading a
//! key event's scan code.

use kx_module_api::ports::{Button, PointerAct, Scan};

use super::fake::Call;
use super::run::sends;
use super::{A, B, CFG, CTRL, KEY_DOWN, KEY_UP, LEFT_DOWN, LEFT_UP, MOVE, OTHER, Rig, SHIFT, rig};
use crate::hook::Source;
use crate::hook::accept::{sent_scan, user_key};
use crate::keys::{REPLAY, SIM_TAG, TAG};

/// `WM_SYSKEYUP`.
const SYS_UP: u32 = 0x0105;
/// `VK_CONTROL` and `VK_SHIFT`.
const VK_CONTROL: u32 = 0x11;
const VK_SHIFT: u32 = 0x10;

/// A rig whose engine sent only a Ctrl press at a settle, which no echo has followed yet.
fn ctrl_in_flight() -> Rig {
    let mut rig = rig(Source::Physical);
    rig.plan().settle_acts = vec![PointerAct::KeyDown(CTRL)];
    rig.host.turn();
    rig.plan().settle_acts.clear();
    rig.world.take();
    rig
}

#[test]
fn a_key_only_batch_in_flight_keeps_the_record_against_the_hands_key_up() {
    let mut rig = ctrl_in_flight();
    let hand = rig.mark(false, 0);
    rig.host.key(KEY_UP, VK_CONTROL, CTRL, hand);
    rig.host.stop();
    assert_eq!(sends(&rig.world.take()), [vec![PointerAct::KeyUp(CTRL)]]);
}

#[test]
fn a_key_up_ahead_of_our_unechoed_press_is_owed_at_once_and_a_new_press_cancels_it() {
    let mut rig = ctrl_in_flight();
    let hand = rig.mark(false, 0);
    rig.host.key(KEY_UP, VK_CONTROL, CTRL, hand);
    assert_eq!(rig.world.take(), [Call::Wake]);
    rig.host.turn();
    assert_eq!(sends(&rig.world.take()), [vec![PointerAct::KeyUp(CTRL)]]);
    let mut rig = ctrl_in_flight();
    rig.host.key(KEY_UP, VK_CONTROL, CTRL, hand);
    rig.host.key(KEY_DOWN, VK_CONTROL, CTRL, hand);
    rig.host.turn();
    assert!(
        sends(&rig.world.take()).is_empty(),
        "the hand holds Ctrl again"
    );
}

#[test]
fn a_press_whose_echo_was_lost_stays_in_doubt_against_the_hands_release() {
    let mut rig = ctrl_in_flight();
    rig.world.pass(CFG.echo_ms + 1);
    assert!(!rig.hand(MOVE, A), "the wait ends as lost");
    let hand = rig.mark(false, 0);
    rig.host.key(KEY_UP, VK_CONTROL, CTRL, hand);
    rig.host.turn();
    assert_eq!(sends(&rig.world.take()), [vec![PointerAct::KeyUp(CTRL)]]);
}

/// A rig whose engine pressed Left at a settle, then filled the queue behind that unechoed press.
fn left_in_flight_and_full() -> Rig {
    let mut rig = rig(Source::Physical);
    rig.plan().settle_acts = vec![PointerAct::Down(Button::Left, A)];
    rig.host.turn();
    rig.plan().settle_acts.clear();
    rig.plan().on_acts = vec![PointerAct::KeyDown(SHIFT)];
    while rig.hand(MOVE, A) {}
    rig
}

#[test]
fn an_up_that_passes_a_full_queue_ahead_of_our_unechoed_down_is_owed_first_in_the_next_batch() {
    let mut rig = left_in_flight_and_full();
    assert!(!rig.hand(LEFT_UP, B), "the queue is full, so it passes");
    rig.world.take();
    rig.host.turn();
    let sent = sends(&rig.world.take());
    let up = PointerAct::Up(Button::Left, B);
    assert_eq!(
        sent[0].first(),
        Some(&up),
        "before what was queued after it"
    );
}

#[test]
fn a_press_that_passes_after_it_cancels_the_owed_up() {
    let mut rig = left_in_flight_and_full();
    assert!(!rig.hand(LEFT_UP, B));
    assert!(!rig.hand(LEFT_DOWN, B), "the hand holds Left again");
    rig.world.take();
    rig.host.turn();
    let ups = |act: &PointerAct| matches!(act, PointerAct::Up(Button::Left, _));
    assert!(!sends(&rig.world.take()).concat().iter().any(ups));
}

#[test]
fn a_key_only_batchs_echo_ends_the_wait() {
    let mut rig = ctrl_in_flight();
    let echo = rig.mark(true, REPLAY);
    rig.host.key(KEY_DOWN, VK_CONTROL, CTRL, echo);
    assert!(!rig.hand(MOVE, A), "nothing waits");
    let hand = rig.mark(false, 0);
    rig.host.key(KEY_UP, VK_CONTROL, CTRL, hand);
    rig.host.stop();
    assert!(sends(&rig.world.take()).is_empty(), "the hand let it go");
}

/// A rig whose engine clicked Left, a wait given up in a stall, then pressed Left again for the hand's held-back press;
/// that click's late echo is back, so only the newer press is out.
fn late_click_then_newer_press() -> Rig {
    let mut rig = rig(Source::Physical);
    let click = [
        PointerAct::Down(Button::Left, A),
        PointerAct::Up(Button::Left, A),
    ];
    rig.plan().settle_acts = click.to_vec();
    rig.host.turn();
    rig.plan().settle_acts.clear();
    rig.world.pass(CFG.echo_ms + 1);
    rig.plan().drop = true;
    rig.plan().on_acts = vec![PointerAct::Down(Button::Left, B), PointerAct::Move(B)];
    assert!(
        rig.hand(LEFT_DOWN, B),
        "the wait ends as lost, and the engine holds it back"
    );
    (rig.plan().drop, rig.plan().on_acts) = (false, Vec::new());
    rig.host.turn();
    rig.echo(&click);
    rig.world.take();
    rig
}

#[test]
fn a_given_up_clicks_late_echo_keeps_the_hands_up_behind_our_newer_press() {
    let mut rig = late_click_then_newer_press();
    assert!(rig.hand(LEFT_UP, B), "it waits behind our press");
}

#[test]
fn an_up_that_passes_our_newer_press_after_a_late_echo_is_owed_first() {
    let mut rig = late_click_then_newer_press();
    rig.world.pass(CFG.echo_ms + 1);
    assert!(!rig.hand(LEFT_UP, B), "a second lost wait lets it pass");
    rig.host.turn();
    let first = sends(&rig.world.take()).concat().first().copied();
    assert_eq!(first, Some(PointerAct::Up(Button::Left, B)));
}

/// A rig whose engine pressed `acts` at a settle, with our echo back, so nothing of ours is in flight.
fn pressed(acts: &[PointerAct]) -> Rig {
    let mut rig = rig(Source::Physical);
    rig.plan().settle_acts = acts.to_vec();
    rig.host.turn();
    rig.plan().settle_acts.clear();
    rig.echo(acts);
    assert!(!rig.hand(MOVE, A), "our echo is back");
    rig.world.take();
    rig.heard();
    rig
}

#[test]
fn the_hands_own_up_with_nothing_in_flight_clears_the_record() {
    let mut rig = pressed(&[PointerAct::Down(Button::Left, A)]);
    assert!(!rig.hand(LEFT_UP, B), "the engine lets it through");
    rig.host.stop();
    let want = [Call::Arm(None), Call::Unhook];
    assert_eq!(rig.world.take(), want, "no Up for Left");
}

#[test]
fn an_up_that_passes_while_our_down_is_in_flight_keeps_the_record() {
    let mut rig = rig(Source::Physical);
    rig.plan().settle_acts = vec![PointerAct::Down(Button::Left, A)];
    rig.host.turn();
    rig.plan().panic = true;
    assert!(!rig.hand(LEFT_UP, B), "a broken engine lets it pass");
    rig.world.take();
    rig.host.stop();
    let want = vec![PointerAct::Up(Button::Left, B)];
    assert_eq!(sends(&rig.world.take()), [want]);
}

/// A rig whose engine carries: the hand's press passed and its release was dropped.
fn carrying() -> Rig {
    let mut rig = rig(Source::Physical);
    rig.plan().drop_ups = true;
    assert!(!rig.hand(LEFT_DOWN, A), "the pick-up passes");
    assert!(rig.hand(LEFT_UP, A), "its release is dropped");
    rig.world.take();
    rig
}

#[test]
fn a_carried_press_is_let_go_by_the_host_even_when_the_engines_release_panics() {
    let mut rig = carrying();
    rig.plan().panic = true;
    rig.host.stop();
    let want = vec![PointerAct::Up(Button::Left, A)];
    assert_eq!(sends(&rig.world.take()), [want]);
}

#[test]
fn the_hands_next_release_lets_the_carried_press_go_so_a_stop_sends_nothing_more() {
    let mut rig = carrying();
    rig.plan().drop_ups = false;
    assert!(!rig.hand(LEFT_DOWN, B));
    assert!(!rig.hand(LEFT_UP, B));
    rig.plan().panic = true;
    rig.world.take();
    rig.host.stop();
    assert!(sends(&rig.world.take()).is_empty());
}

#[test]
fn a_stop_tries_the_records_let_go_once_even_when_it_is_refused() {
    let mut rig = rig(Source::Physical);
    rig.plan().settle_acts = vec![PointerAct::Down(Button::Left, A)];
    rig.host.turn();
    rig.plan().panic = true;
    assert!(!rig.hand(MOVE, B));
    rig.world.takes.set(Some(0));
    rig.world.take();
    rig.host.stop();
    assert_eq!(sends(&rig.world.take()).len(), 1);
    assert_eq!(rig.stats().send_errors, 1);
}

#[test]
fn the_hands_own_key_up_clears_the_record_and_another_programs_does_not() {
    let mut rig = pressed(&[PointerAct::KeyDown(CTRL), PointerAct::KeyDown(SHIFT)]);
    let hand = rig.mark(false, 0);
    rig.host.key(KEY_UP, VK_CONTROL, CTRL, hand);
    let other = rig.mark(true, OTHER);
    rig.host.key(KEY_UP, VK_SHIFT, SHIFT, other);
    rig.host.stop();
    assert_eq!(sends(&rig.world.take()), [vec![PointerAct::KeyUp(SHIFT)]]);
}

#[test]
fn a_key_counts_only_from_the_users_source() {
    let (up, down) = (
        Some(PointerAct::KeyUp(CTRL)),
        Some(PointerAct::KeyDown(CTRL)),
    );
    assert_eq!(user_key(Source::Physical, KEY_UP, CTRL, false, 0), up);
    assert_eq!(user_key(Source::Physical, SYS_UP, CTRL, false, 0), up);
    assert_eq!(user_key(Source::Physical, KEY_DOWN, CTRL, false, 0), down);
    assert_eq!(user_key(Source::Physical, KEY_UP, CTRL, true, TAG), None);
    assert_eq!(user_key(Source::Simulated, KEY_UP, CTRL, true, SIM_TAG), up);
    assert_eq!(user_key(Source::Simulated, KEY_UP, CTRL, false, 0), None);
}

#[test]
fn a_hook_scan_code_reads_back_as_the_scan_we_send() {
    assert_eq!(sent_scan(0x1D, false), CTRL);
    assert_eq!(sent_scan(0x1D, true), Scan::new(0xE01D), "right Ctrl");
    assert_eq!(
        sent_scan(0x36, true),
        Scan::new(0x36),
        "right Shift is sent plain"
    );
    assert_eq!(sent_scan(0x0100 | 0x2A, false), SHIFT, "only the low byte");
}
