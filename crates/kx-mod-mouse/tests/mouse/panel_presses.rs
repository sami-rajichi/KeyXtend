//! Presses have two sources, word prediction and an open panel (ADR-0040 decision 5), with one result: they are told
//! while either wants them, and one going off never silences the other.

use crate::live::{Live, WAIT, hear, on, synced};
use crate::rig::{A, Heard, Rig, T0, T1, T2, down, predicting, up};
use kx_mod_mouse::Msg;
use kx_module_api::Bus;
use kx_module_api::ports::{AssistChanged, PanelPressesWanted, PointerPressed, PressesAsk};

/// A press outside our windows, as told on the bus.
const AWAY: Heard = Heard::Pressed(PointerPressed { ours: false });

/// A started rig with what it said so far taken.
fn started() -> Rig {
    let mut rig = Rig::new();
    rig.begin();
    rig.take();
    rig
}

/// Whether a press at `now` is told, as the hook runs.
fn told(rig: &mut Rig, now: kx_module_api::Mono) -> bool {
    assert!(!rig.feed(down(A), now), "the press reaches the app");
    assert!(!rig.feed(up(A), now));
    rig.take().contains(&AWAY)
}

#[test]
fn presses_are_told_while_either_source_wants_them() {
    for (prediction, panel) in [(false, false), (true, false), (false, true), (true, true)] {
        let mut rig = started();
        rig.send(Msg::Presses(predicting(prediction)));
        rig.send(Msg::PanelPresses(panel));
        rig.settle(T0);
        let want = prediction || panel;
        assert_eq!(rig.hook.running(), want, "{prediction} {panel}");
        if want {
            assert!(told(&mut rig, T1), "{prediction} {panel}");
        }
    }
}

#[test]
fn the_panel_going_off_leaves_prediction_heard() {
    let mut rig = started();
    rig.send(Msg::Presses(predicting(true)));
    rig.send(Msg::PanelPresses(true));
    rig.send(Msg::PanelPresses(false));
    rig.settle(T0);
    assert!(rig.hook.running());
    assert!(told(&mut rig, T1));
    rig.send(Msg::Presses(predicting(false)));
    rig.settle(T2);
    assert!(!rig.hook.running(), "nothing wants them now");
}

#[test]
fn prediction_going_off_leaves_the_panel_heard() {
    let mut rig = started();
    rig.send(Msg::PanelPresses(true));
    rig.send(Msg::Presses(predicting(true)));
    rig.send(Msg::Presses(predicting(false)));
    rig.settle(T0);
    assert!(rig.hook.running());
    assert!(told(&mut rig, T1));
    rig.send(Msg::PanelPresses(false));
    rig.settle(T2);
    assert!(!rig.hook.running(), "nothing wants them now");
}

#[test]
fn a_repeated_word_from_one_source_changes_nothing() {
    let mut rig = started();
    rig.send(Msg::PanelPresses(true));
    rig.send(Msg::PanelPresses(true));
    assert_eq!(rig.hook.starts(), 1);
    rig.send(Msg::Presses(predicting(false)));
    rig.settle(T0);
    assert!(
        told(&mut rig, T1),
        "prediction never wanted them, the panel does"
    );
}

#[test]
fn a_fresh_run_after_a_fault_takes_the_panels_wish() {
    let mut rig = started();
    rig.send(Msg::PanelPresses(true));
    rig.settle(T0);
    rig.hook.give_up();
    rig.pump();
    rig.take();
    assert!(rig.hook.running(), "the panel still needs the hook");
    assert_eq!(rig.hook.starts(), 2, "one fresh run");
    assert!(told(&mut rig, T1));
}

#[test]
fn the_panels_wish_never_re_arms_a_spent_restart() {
    let mut rig = started();
    rig.send(Msg::PanelPresses(true));
    rig.settle(T0);
    for _ in 0..2 {
        rig.hook.give_up();
        rig.pump();
    }
    assert!(!rig.hook.running(), "the second fault started nothing");
    rig.send(Msg::PanelPresses(false));
    rig.send(Msg::PanelPresses(true));
    assert_eq!(rig.hook.starts(), 2, "as prediction's wish does not");
}

#[test]
fn a_stop_lets_the_hook_go_while_the_panel_wants_presses() {
    let mut rig = started();
    rig.send(Msg::PanelPresses(true));
    assert!(!rig.send(Msg::Stop), "the actor is done");
    assert!(!rig.hook.running());
}

/// The real relay: each start asks, and either source's answer brings the hook back.
#[test]
fn both_sources_are_heard_on_the_bus_at_a_late_start() {
    let listen = |bus: &Bus| (hear::<AssistChanged>(bus), hear::<PressesAsk>(bus));
    let (mut live, ((_e1, states), (_e2, asks))) = Live::start(|_| {}, listen);
    assert_eq!(states.recv_timeout(WAIT), Ok(on(false)));
    assert_eq!(asks.recv_timeout(WAIT), Ok(PressesAsk));
    live.bus.publish(&PanelPressesWanted { on: true }).unwrap();
    synced(&live, &states);
    assert!(live.hook.running(), "the panel's answer starts the hook");
    live.bus.publish(&predicting(true)).unwrap();
    live.bus.publish(&PanelPressesWanted { on: false }).unwrap();
    synced(&live, &states);
    live.hook.settle(T0);
    synced(&live, &states);
    assert!(live.hook.running(), "prediction still wants them");
    live.bus.publish(&predicting(false)).unwrap();
    synced(&live, &states);
    live.hook.settle(T1);
    synced(&live, &states);
    assert!(!live.hook.running(), "unwanted lets it go");
    live.stop();
}
