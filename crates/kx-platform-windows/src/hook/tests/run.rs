//! Tests of the host's loop: turns, the deadline timer, failed sends, the panic hook's let-go and the stop.

use std::rc::Rc;
use std::sync::{Arc, Mutex};

use kx_module_api::ports::{Button, PointerAct, PointerEvent};
use kx_module_api::{Mono, US_PER_MS};

use super::fake::{Call, Fake, Heard, Plan, Script, World};
use super::{A, B, CFG, CTRL, MOVE, Rig, rig};
use crate::hook::Source;
use crate::hook::host::Host;
use crate::hook::stats::Stats;

/// How far ahead the pending press's deadline lies, in ms.
const DUE_MS: u32 = 600;

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
pub(super) fn holding() -> Rig {
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
    let want = vec![PointerAct::Move(A), PointerAct::Down(Button::Right, B)];
    assert_eq!(
        sends(&calls),
        [want],
        "one batch: the queue, then the settle"
    );
    assert_eq!(calls.last(), Some(&Call::Arm(Some(2))));
    assert_eq!(rig.heard(), [Heard::Settle(rig.now())]);
}

/// A rig whose engine holds a press back until a deadline `DUE_MS` from now.
fn pending() -> Rig {
    let rig = rig(Source::Physical);
    let due = rig.now().as_us() + u64::from(DUE_MS) * u64::from(US_PER_MS);
    rig.plan().deadline = Some(Mono::from_us(due));
    rig
}

#[test]
fn the_timer_is_set_and_stopped_only_when_the_deadline_changes() {
    let mut rig = pending();
    rig.host.turn();
    assert_eq!(rig.world.take(), [Call::Arm(Some(DUE_MS + 1))]);
    rig.host.turn();
    assert!(rig.world.take().is_empty(), "the same deadline");
    rig.plan().deadline = None;
    rig.host.turn();
    assert_eq!(rig.world.take(), [Call::Arm(None)]);
    rig.host.turn();
    assert!(rig.world.take().is_empty(), "still none");
}

#[test]
fn a_fired_timer_is_set_again_for_a_deadline_still_to_come() {
    let mut rig = pending();
    rig.host.turn();
    rig.world.take();
    rig.host.fired();
    assert_eq!(rig.world.take(), [Call::Arm(Some(DUE_MS + 1))]);
}

#[test]
fn a_fire_that_used_up_the_deadline_stops_the_timer() {
    let mut rig = pending();
    rig.host.turn();
    rig.world.take();
    rig.plan().deadline = None;
    rig.host.fired();
    assert_eq!(rig.world.take(), [Call::Arm(None)], "it would fire again");
}

#[test]
fn a_refused_timer_is_asked_for_again_at_the_next_turn() {
    let mut rig = pending();
    rig.world.refuse_timer.set(true);
    rig.host.turn();
    rig.host.turn();
    let arms = |calls: Vec<Call>| calls.iter().filter(|c| matches!(c, Call::Arm(_))).count();
    assert_eq!(arms(rig.world.take()), 2);
    rig.world.refuse_timer.set(false);
    rig.host.turn();
    rig.host.turn();
    assert_eq!(arms(rig.world.take()), 1, "set at last");
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
fn a_partial_send_counts_and_waits_for_the_part_the_os_took() {
    let mut rig = rig(Source::Physical);
    rig.world.takes.set(Some(1));
    rig.plan().settle_acts = vec![
        PointerAct::Down(Button::Left, B),
        PointerAct::Up(Button::Left, B),
    ];
    rig.host.turn();
    assert_eq!(rig.stats().send_errors, 1);
    assert!(rig.hand(MOVE, A), "the press went out, so input waits");
    rig.echo(&[PointerAct::Down(Button::Left, B)]);
    rig.host.turn();
    rig.echo(&[PointerAct::Move(A)]);
    assert!(!rig.hand(MOVE, A), "its echo ended the wait");
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
        Call::Send(vec![PointerAct::Move(B), PointerAct::Up(Button::Left, A)]),
        Call::Arm(None),
        Call::Unhook,
    ];
    assert_eq!(rig.world.take(), want);
    assert_eq!(rig.heard(), [Heard::Release]);
}

#[test]
fn stopping_lets_go_of_what_our_input_still_holds_before_it_unhooks() {
    let mut rig = rig(Source::Physical);
    rig.plan().settle_acts = vec![PointerAct::KeyDown(CTRL), PointerAct::Down(Button::Left, A)];
    rig.host.turn();
    rig.plan().settle_acts.clear();
    rig.world.take();
    rig.host.stop();
    let want = [
        Call::Send(vec![
            PointerAct::Up(Button::Left, A),
            PointerAct::KeyUp(CTRL),
        ]),
        Call::Arm(None),
        Call::Unhook,
    ];
    assert_eq!(rig.world.take(), want);
}

#[test]
fn a_broken_engine_whose_release_panics_still_ends_with_ctrl_and_left_up() {
    let mut rig = rig(Source::Physical);
    rig.plan().settle_acts = vec![PointerAct::KeyDown(CTRL), PointerAct::Down(Button::Left, A)];
    rig.host.turn();
    rig.plan().panic = true;
    assert!(!rig.hand(MOVE, B), "the engine panics, so the move passes");
    rig.world.take();
    rig.host.turn();
    let want = vec![PointerAct::Up(Button::Left, B), PointerAct::KeyUp(CTRL)];
    assert_eq!(sends(&rig.world.take()), [want]);
    let last = [Heard::Release, Heard::Dropped];
    assert!(
        rig.heard().ends_with(&last),
        "its one release, then the drop"
    );
}

#[test]
fn a_panic_outside_the_engine_asks_for_one_turn_never_a_loop() {
    let mut rig = pending();
    rig.world.panic_arm.set(true);
    rig.host.turn();
    assert!(
        rig.world.take().contains(&Call::Wake),
        "the fall asks for a turn"
    );
    for _ in 0..3 {
        rig.host.turn();
        assert!(!rig.world.take().contains(&Call::Wake), "no wake loop");
    }
}

#[test]
fn a_release_lets_go_of_the_queue_the_engine_and_our_held_input_and_keeps_the_hooks() {
    let mut rig = holding();
    rig.plan().settle_acts = vec![PointerAct::KeyDown(CTRL)];
    rig.host.turn();
    rig.plan().settle_acts.clear();
    assert!(rig.hand(MOVE, B), "held back behind our Ctrl");
    rig.world.take();
    rig.heard();
    rig.host.release();
    let want = vec![
        PointerAct::Move(B),
        PointerAct::Up(Button::Left, A),
        PointerAct::KeyUp(CTRL),
    ];
    let calls = rig.world.take();
    assert_eq!(sends(&calls), [want], "one batch");
    assert!(!calls.contains(&Call::Unhook), "the hooks stay");
    assert_eq!(rig.heard(), [Heard::Release]);
}

#[test]
fn a_release_asks_for_a_turn_so_the_engines_notes_go_out_at_once() {
    let mut rig = holding();
    rig.host.release();
    let calls = rig.world.take();
    assert_eq!(calls.last(), Some(&Call::Wake), "{calls:?}");
    rig.host.turn();
    assert_eq!(rig.heard().last(), Some(&Heard::Settle(rig.now())));
}

#[test]
fn after_a_release_the_run_keeps_hearing_the_hand() {
    let mut rig = holding();
    rig.host.release();
    rig.world.pass(CFG.echo_ms + 1);
    rig.heard();
    rig.world.take();
    assert!(!rig.hand(MOVE, A));
    assert_eq!(
        rig.heard(),
        [Heard::On(PointerEvent::Move { at: A }, rig.now())]
    );
    rig.host.turn();
    assert_eq!(rig.heard(), [Heard::Settle(rig.now())]);
    assert!(!rig.world.take().contains(&Call::Unhook));
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
fn a_refused_install_reports_the_os_code() {
    let world = Rc::new(World::default());
    world.refuse.set(Some(1428));
    let engine = Box::new(Script(Arc::new(Mutex::new(Plan::default()))));
    let stats = Arc::new(Stats::new());
    let fake = Fake(Rc::clone(&world));
    let mut host = Host::new(fake, engine, Source::Physical, &CFG, stats);
    assert_eq!(host.begin(), Err(1428));
}
