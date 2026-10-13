//! Tests of the host's loop: turns, the deadline timer, failed sends, the health check and the stop.

use std::rc::Rc;
use std::sync::{Arc, Mutex};

use kx_module_api::Mono;
use kx_module_api::ports::{Button, PointerAct};

use super::fake::{Call, Fake, Heard, Plan, Script, World};
use super::{A, B, CFG, MOVE, Rig, rig};
use crate::hook::Source;
use crate::hook::host::Host;
use crate::hook::stats::Stats;

/// The calls that sent input, in order.
pub(super) fn sends(calls: &[Call]) -> Vec<Vec<PointerAct>> {
    calls
        .iter()
        .filter_map(|c| match c {
            Call::Send(acts) => Some(acts.clone()),
            _ => None,
        })
        .collect()
}

/// A rig whose engine lets go of the left button when released.
fn holding() -> Rig {
    let rig = rig(Source::Physical);
    rig.plan().release_acts = vec![PointerAct::Up(Button::Left, A)];
    rig
}

#[test]
fn a_turn_sends_the_queue_then_settles_and_arms_the_deadline_rounded_up() {
    let mut rig = rig(Source::Physical);
    rig.send_ours();
    assert!(rig.hand(MOVE, A));
    rig.heard();
    rig.plan().settle_acts = vec![PointerAct::Down(Button::Right, B)];
    rig.plan().deadline = Some(Mono::from_us(rig.now().as_us() + 1_200));
    rig.host.turn();
    let calls = rig.world.take();
    let want = vec![
        vec![PointerAct::Move(A)],
        vec![PointerAct::Down(Button::Right, B)],
    ];
    assert_eq!(sends(&calls), want, "the queue first, then the settle");
    assert_eq!(calls.last(), Some(&Call::Arm(Some(2))));
    assert_eq!(rig.heard(), [Heard::Settle(rig.now())]);
}

#[test]
fn a_turn_with_no_deadline_stops_the_timer() {
    let mut rig = rig(Source::Physical);
    rig.host.turn();
    assert_eq!(rig.world.take(), [Call::Arm(None)]);
}

#[test]
fn a_failed_send_counts_and_ends_the_wait() {
    let mut rig = rig(Source::Physical);
    rig.world.takes.set(Some(0));
    rig.plan().settle_acts = vec![PointerAct::Move(B)];
    rig.host.turn();
    assert_eq!(rig.stats().send_errors, 1);
    assert!(!rig.hand(MOVE, A), "no echo is coming, so nothing waits");
}

#[test]
fn a_partial_send_counts_too() {
    let mut rig = rig(Source::Physical);
    rig.world.takes.set(Some(1));
    rig.plan().settle_acts = vec![PointerAct::Move(B), PointerAct::Move(A)];
    rig.host.turn();
    assert_eq!(rig.stats().send_errors, 1);
}

#[test]
fn stopping_sends_the_queue_then_the_release_and_only_then_unhooks() {
    let mut rig = holding();
    rig.send_ours();
    assert!(rig.hand(MOVE, B));
    rig.world.take();
    rig.heard();
    rig.host.stop();
    let want = [
        Call::Send(vec![PointerAct::Move(B)]),
        Call::Send(vec![PointerAct::Up(Button::Left, A)]),
        Call::Arm(None),
        Call::Unhook,
    ];
    assert_eq!(rig.world.take(), want);
    assert_eq!(rig.heard(), [Heard::Release]);
}

#[test]
fn stopping_with_nothing_held_still_unhooks() {
    let mut rig = rig(Source::Physical);
    rig.host.stop();
    assert_eq!(rig.world.take(), [Call::Arm(None), Call::Unhook]);
}

#[test]
fn a_stop_whose_release_is_refused_still_unhooks_and_counts_it() {
    let mut rig = holding();
    rig.world.takes.set(Some(0));
    rig.host.stop();
    assert_eq!(rig.world.take().last(), Some(&Call::Unhook));
    assert_eq!(rig.stats().send_errors, 1);
}

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
    ];
    assert_eq!(calls[at..], want, "let go before the hooks go back");
    assert_eq!(rig.stats().reinstalls, 1);
    rig.host.check();
    assert!(!rig.world.take().contains(&Call::Unhook), "once per period");
}

#[test]
fn after_a_reinstall_no_wait_for_an_echo_the_hooks_could_not_see() {
    let mut rig = holding();
    rig.world.pass(2_000);
    rig.world.last_input.set(Some(rig.world.tick.get()));
    rig.host.check();
    assert!(!rig.hand(MOVE, A));
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

#[test]
fn a_refused_install_reports_the_os_code() {
    let world = Rc::new(World::default());
    world.refuse.set(Some(1428));
    let engine = Box::new(Script(Arc::new(Mutex::new(Plan::default()))));
    let stats = Arc::new(Stats::new());
    let fake = Fake(Rc::clone(&world));
    let mut host = Host::new(fake, engine, Source::Physical, &CFG, stats);
    assert_eq!(host.begin(), Err(1428));
}
