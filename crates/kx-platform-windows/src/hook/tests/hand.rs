//! Tests of the record of held input against the hand's own Ups, and of reading a key event's scan code.

use kx_module_api::ports::{Button, PointerAct, Scan};

use super::fake::Call;
use super::run::sends;
use super::{A, B, CTRL, KEY_UP, LEFT_UP, MOVE, OTHER, Rig, SHIFT, rig};
use crate::hook::Source;
use crate::hook::accept::{key_up, sent_scan};
use crate::keys::{REPLAY, SIM_TAG, TAG};

/// `WM_SYSKEYUP`, `WM_KEYDOWN`, and `VK_CONTROL` and `VK_SHIFT`.
const SYS_UP: u32 = 0x0105;
const KEY_DOWN: u32 = 0x0100;
const VK_CONTROL: u32 = 0x11;
const VK_SHIFT: u32 = 0x10;

/// A rig whose engine pressed `acts` at a settle, with our echo back, so nothing of ours is in flight.
fn pressed(acts: Vec<PointerAct>) -> Rig {
    let mut rig = rig(Source::Physical);
    rig.plan().settle_acts = acts;
    rig.host.turn();
    rig.plan().settle_acts.clear();
    assert!(!rig.injected(MOVE, A, REPLAY), "our echo is back");
    rig.world.take();
    rig
}

#[test]
fn the_hands_own_up_with_nothing_in_flight_clears_the_record() {
    let mut rig = pressed(vec![PointerAct::Down(Button::Left, A)]);
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
    let mut rig = pressed(vec![PointerAct::KeyDown(CTRL), PointerAct::KeyDown(SHIFT)]);
    let hand = rig.mark(false, 0);
    rig.host.key(KEY_UP, VK_CONTROL, CTRL, hand);
    let other = rig.mark(true, OTHER);
    rig.host.key(KEY_UP, VK_SHIFT, SHIFT, other);
    rig.host.stop();
    assert_eq!(sends(&rig.world.take()), [vec![PointerAct::KeyUp(SHIFT)]]);
}

#[test]
fn a_key_up_counts_only_from_the_users_source() {
    assert!(key_up(Source::Physical, KEY_UP, false, 0));
    assert!(key_up(Source::Physical, SYS_UP, false, 0));
    assert!(!key_up(Source::Physical, KEY_DOWN, false, 0));
    assert!(!key_up(Source::Physical, KEY_UP, true, TAG));
    assert!(key_up(Source::Simulated, KEY_UP, true, SIM_TAG));
    assert!(!key_up(Source::Simulated, KEY_UP, false, 0));
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
