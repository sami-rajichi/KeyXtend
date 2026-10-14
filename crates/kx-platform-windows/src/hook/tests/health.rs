//! Tests of the health check: current hooks stay, and lagging ones go back once a period, after letting go.

use kx_module_api::ports::{Button, PointerAct};

use super::fake::Call;
use super::run::holding;
use super::{A, MOVE, OTHER, rig};
use crate::hook::{Mark, Source};

/// How far ahead another program dates its input: about a week.
const FORWARD_MS: u32 = 600_000_000;

#[test]
fn current_hooks_stay() {
    let mut rig = rig(Source::Physical);
    rig.world.pass(400);
    rig.hand(MOVE, A);
    rig.world.last_input.set(Some(rig.world.tick.get()));
    rig.world.pass(2_000);
    rig.host.check();
    assert!(!rig.world.take().contains(&Call::Unhook));
    assert_eq!(rig.stats().reinstalls, 0);
}

#[test]
fn lagging_hooks_go_back_once_after_letting_go() {
    let mut rig = holding();
    rig.world.pass(2_000);
    rig.world.last_input.set(Some(rig.world.tick.get()));
    rig.host.check();
    let calls = rig.world.take();
    let at = calls.iter().position(|c| *c == Call::Unhook).unwrap();
    let want = [
        Call::Unhook,
        Call::Send(vec![PointerAct::Up(Button::Left, A)]),
        Call::Hook,
        Call::Wake,
    ];
    assert_eq!(calls[at..], want, "let go, hooks back, then a turn");
    assert_eq!(rig.stats().reinstalls, 1);
    rig.host.check();
    assert!(!rig.world.take().contains(&Call::Unhook), "once per period");
}

#[test]
fn a_forward_dated_injected_event_cannot_hide_lagging_hooks() {
    let mut rig = rig(Source::Physical);
    let future = Mark {
        time: rig.world.tick.get().wrapping_add(FORWARD_MS),
        ..rig.mark(true, OTHER)
    };
    assert!(!rig.host.mouse(MOVE, A, future));
    rig.world.pass(2_000);
    rig.world.last_input.set(Some(rig.world.tick.get()));
    rig.host.check();
    assert_eq!(rig.stats().reinstalls, 1);
}

#[test]
fn after_a_reinstall_input_waits_for_the_let_gos_echo() {
    let mut rig = holding();
    rig.world.pass(2_000);
    rig.world.last_input.set(Some(rig.world.tick.get()));
    rig.host.check();
    assert!(
        rig.hand(MOVE, A),
        "behind the let-go, which the hooks put back can see"
    );
    rig.echo(&[PointerAct::Up(Button::Left, A)]);
    rig.host.turn();
    rig.echo(&[PointerAct::Move(A)]);
    assert!(!rig.hand(MOVE, A), "its echo ended the wait");
}

#[test]
fn a_refused_reinstall_keeps_the_thread_running() {
    let mut rig = rig(Source::Physical);
    rig.world.pass(2_000);
    rig.world.last_input.set(Some(rig.world.tick.get()));
    rig.world.refuse.set(Some(5));
    rig.host.check();
    assert_eq!(rig.stats().reinstalls, 0);
    rig.host.turn();
}
