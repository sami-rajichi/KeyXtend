//! Tests of the wait for our own input: each echo matches our next act of its kind, so only the newest batch's first
//! echo ends the wait, and input waits behind it.

use kx_module_api::ports::{Button, PointerAct};
use kx_module_api::{Mono, US_PER_MS};

use super::fake::Call;
use super::{A, B, CFG, CTRL, KEY_DOWN, LEFT_DOWN, MOVE, SHIFT, rig};
use crate::hook::Source;
use crate::hook::echo::{Echo, Wait};
use crate::keys::{REPLAY, TAG};

/// Room for a few acts.
const ROOM: usize = 8;

fn ms(n: u64) -> Mono {
    Mono::from_us(n * u64::from(US_PER_MS))
}

#[test]
fn the_wait_ends_when_our_input_comes_back_or_after_echo_ms() {
    let mut echo = Echo::new(CFG.echo_ms, ROOM);
    assert_eq!(echo.check(ms(1)), Wait::Clear);
    echo.sent(&[PointerAct::Move(A)], ms(10));
    assert_eq!(echo.check(ms(260)), Wait::Waiting);
    echo.back(PointerAct::Move(B));
    assert_eq!(echo.check(ms(261)), Wait::Clear, "a move's point may round");
    echo.sent(&[PointerAct::Move(A)], ms(300));
    assert_eq!(echo.check(ms(551)), Wait::Lost);
    assert_eq!(echo.check(ms(552)), Wait::Clear, "counted once");
}

#[test]
fn nothing_taken_starts_no_wait() {
    let mut echo = Echo::new(CFG.echo_ms, ROOM);
    echo.sent(&[], ms(10));
    assert!(echo.is_clear());
}

#[test]
fn an_older_batchs_late_echo_never_ends_the_newest_batchs_wait() {
    let mut echo = Echo::new(CFG.echo_ms, ROOM);
    echo.sent(
        &[PointerAct::KeyDown(CTRL), PointerAct::KeyUp(CTRL)],
        ms(10),
    );
    echo.back(PointerAct::KeyDown(CTRL));
    assert!(
        echo.is_clear(),
        "the first echo proves the whole batch passed"
    );
    echo.sent(&[PointerAct::Down(Button::Left, A)], ms(11));
    echo.back(PointerAct::KeyUp(CTRL));
    assert!(!echo.is_clear(), "the older batch's tail");
    echo.back(PointerAct::Down(Button::Left, A));
    assert!(echo.is_clear());
}

#[test]
fn a_move_before_a_press_is_the_press_moving_not_its_echo() {
    let mut echo = Echo::new(CFG.echo_ms, ROOM);
    echo.sent(&[PointerAct::Down(Button::Left, A)], ms(10));
    echo.back(PointerAct::Move(A));
    assert!(!echo.is_clear());
    echo.back(PointerAct::Down(Button::Right, A));
    assert!(!echo.is_clear(), "another button");
    echo.back(PointerAct::Down(Button::Left, A));
    assert!(echo.is_clear());
}

#[test]
fn a_press_or_key_passes_by_acts_whose_echo_never_came() {
    let mut echo = Echo::new(CFG.echo_ms, ROOM);
    echo.sent(&[PointerAct::KeyDown(CTRL)], ms(10));
    echo.sent(
        &[PointerAct::Move(A), PointerAct::Up(Button::Left, A)],
        ms(11),
    );
    echo.back(PointerAct::Move(A));
    assert!(!echo.is_clear(), "a move matches only the next act");
    echo.back(PointerAct::Up(Button::Left, A));
    assert!(echo.is_clear());
}

#[test]
fn many_batches_out_keep_the_room() {
    let mut echo = Echo::new(CFG.echo_ms, ROOM);
    for n in 0..100 {
        echo.sent(&[PointerAct::KeyDown(CTRL), PointerAct::KeyUp(CTRL)], ms(n));
        echo.back(PointerAct::KeyDown(CTRL));
        echo.back(PointerAct::KeyUp(CTRL));
    }
    assert_eq!(echo.room(), ROOM, "acts that came back make room");
}

#[test]
fn our_input_passes_and_ends_the_wait_for_it() {
    let mut rig = rig(Source::Physical);
    rig.send_ours();
    assert!(!rig.injected(MOVE, B, REPLAY));
    assert!(rig.heard().is_empty(), "never reaches the engine");
    assert!(!rig.hand(MOVE, A), "nothing to wait for");
    rig.host.turn();
    assert!(!rig.world.take().iter().any(|c| matches!(c, Call::Send(_))));
}

#[test]
fn our_keyboards_input_passes_and_leaves_the_wait_for_the_hosts() {
    let mut rig = rig(Source::Physical);
    rig.send_ours();
    assert!(!rig.injected(MOVE, B, TAG));
    let mark = rig.mark(true, TAG);
    rig.host.key(KEY_DOWN, 0, CTRL, mark);
    assert!(rig.heard().is_empty(), "never reaches the engine");
    assert!(rig.hand(MOVE, A), "still behind our move");
}

#[test]
fn let_through_input_waits_behind_our_unechoed_input_in_order() {
    let mut rig = rig(Source::Physical);
    rig.send_ours();
    assert!(rig.hand(MOVE, A), "held back");
    rig.plan().on_acts = vec![PointerAct::KeyUp(SHIFT)];
    assert!(rig.hand(LEFT_DOWN, B), "held back");
    assert_eq!(rig.world.take(), [Call::Wake], "one wake");
    rig.host.turn();
    let want = vec![
        PointerAct::Move(A),
        PointerAct::Down(Button::Left, B),
        PointerAct::KeyUp(SHIFT),
    ];
    assert_eq!(rig.world.take().first(), Some(&Call::Send(want)));
}

#[test]
fn after_echo_ms_the_wait_ends_and_counts_a_lost_echo() {
    let mut rig = rig(Source::Physical);
    rig.send_ours();
    rig.world.pass(CFG.echo_ms + 1);
    assert!(!rig.hand(MOVE, A), "the wait is over");
    assert!(!rig.hand(MOVE, A));
    assert_eq!(rig.stats().lost_echoes, 1);
}
