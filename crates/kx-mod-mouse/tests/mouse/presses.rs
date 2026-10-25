//! The actor while word prediction wants presses (spec §6): the hook stays on and each press is told with `ours`, so a
//! click outside our windows ends the word; every press still reaches the app as the modes say.

use crate::live::{Live, WAIT, hear, on, synced};
use crate::logs::{MESSAGE, lines_of};
use crate::rig::{A, B, CODE, Heard, Rig, T0, T1, T2, down, moved, ours, shown, told, up};
use kx_mod_mouse::Msg;
use kx_module_api::keys;
use kx_module_api::ports::GrabState::Off;
use kx_module_api::ports::{AssistChanged, HookError, PointerPressed, PressesWanted};
use tracing::Level;

/// The field the flag's log line holds.
const ON: &str = "on";

/// A press told on the bus.
fn pressed(ours: bool) -> Heard {
    Heard::Pressed(PointerPressed { ours })
}

/// A started rig whose presses are wanted, with what it said so far taken.
fn wanted(rig: Rig) -> Rig {
    let mut rig = rig;
    rig.begin();
    rig.send(Msg::Presses(true));
    rig.settle(T0);
    rig.take();
    rig
}

#[test]
fn wanted_alone_starts_the_hook_and_tells_each_press_which_reaches_the_app() {
    let mut rig = Rig::new();
    rig.begin();
    rig.take();
    rig.send(Msg::Presses(true));
    assert!(rig.hook.running(), "presses need the hook");
    assert!(rig.take().is_empty(), "no state change and no notice");
    assert!(!rig.feed(moved(A), T0));
    assert!(!rig.feed(down(B), T1), "the press passes");
    assert!(!rig.feed(up(B), T1));
    assert!(!rig.feed(ours(A), T2));
    assert!(rig.hook.take_sent().is_empty(), "nothing of ours is sent");
    assert_eq!(rig.take(), [pressed(false), pressed(true)], "no badge");
}

#[test]
fn unwanted_lets_the_hook_go_once_idle() {
    let mut rig = wanted(Rig::new());
    rig.send(Msg::Presses(false));
    rig.settle(T1);
    assert!(!rig.hook.running(), "nothing else needs it");
    assert_eq!(rig.hook.starts(), 1);
    assert!(rig.take().is_empty(), "no state change and no notice");
}

#[test]
fn unwanted_with_a_mode_on_keeps_the_hook() {
    let mut rig = wanted(Rig::saved_rclick());
    rig.send(Msg::Presses(false));
    rig.settle(T1);
    assert!(rig.hook.running(), "Right-click still needs it");
    assert!(rig.feed(down(A), T2), "the press is held back as before");
    let told = rig.take();
    assert!(!told.contains(&pressed(false)), "{told:?}");
}

#[test]
fn scrolling_and_wanted_keep_presses_told_while_either_holds() {
    let mut rig = wanted(Rig::new());
    rig.send(Msg::Scroll(true));
    rig.send(Msg::Presses(false));
    rig.settle(T0);
    let _ = rig.feed(down(A), T0);
    let _ = rig.feed(up(A), T0);
    assert_eq!(rig.take(), [pressed(false)], "scrolling holds");
    rig.send(Msg::Presses(true));
    rig.send(Msg::Scroll(false));
    rig.settle(T1);
    assert!(rig.hook.running(), "presses hold the hook");
    let _ = rig.feed(ours(B), T1);
    assert_eq!(rig.take(), [pressed(true)], "wanted holds");
    rig.send(Msg::Presses(false));
    rig.settle(T2);
    assert!(!rig.hook.running());
    assert_eq!(rig.hook.starts(), 1);
}

#[test]
fn a_hook_refused_for_presses_tells_no_one_and_the_next_change_tries_again() {
    let lines = lines_of(|| {
        let mut rig = Rig::new();
        rig.begin();
        rig.take();
        rig.hook.fail_next(HookError::Refused(CODE));
        rig.send(Msg::Presses(true));
        assert!(!rig.hook.running());
        assert!(rig.take().is_empty(), "the user pressed no mouse key");
        rig.send(Msg::Presses(false));
        rig.send(Msg::Presses(true));
        assert!(rig.hook.running(), "the next change tries again");
    });
    let warned = lines.iter().filter(|(l, _)| *l == Level::WARN).count();
    assert_eq!(warned, 1, "{lines:?}");
}

#[test]
fn a_given_up_engine_while_presses_are_wanted_gets_one_fresh_run() {
    let mut rig = wanted(Rig::new());
    rig.hook.give_up();
    rig.pump();
    assert_eq!(rig.take(), [shown(false, Off), told(keys::MOUSE_RESET)]);
    assert!(rig.hook.running(), "presses still need the hook");
    assert_eq!(rig.hook.starts(), 2, "one fresh run");
    let _ = rig.feed(down(A), T1);
    assert_eq!(rig.take(), [pressed(false)]);
}

#[test]
fn only_the_flag_s_changes_are_logged_with_no_point() {
    let lines = lines_of(|| {
        let mut rig = Rig::new();
        rig.begin();
        rig.send(Msg::Presses(true));
        let _ = rig.feed(down(A), T0);
        rig.send(Msg::Presses(true));
        rig.send(Msg::Presses(false));
        rig.settle(T1);
    });
    let flags: Vec<_> = lines.iter().filter(|(_, f)| f.contains_key(ON)).collect();
    let ons: Vec<_> = flags.iter().map(|(_, f)| f[ON].as_str()).collect();
    assert_eq!(ons, ["true", "false"], "{lines:?}");
    for (_, fields) in &flags {
        let names: Vec<_> = fields.keys().copied().collect();
        assert_eq!(names, [MESSAGE, ON]);
    }
    let points = [A.x, A.y].map(|n| n.to_string());
    for line in lines.iter().flat_map(|(_, f)| f.values()) {
        assert!(points.iter().all(|p| !line.contains(p.as_str())), "{line}");
    }
}

#[test]
fn a_stop_while_presses_are_wanted_lets_the_hook_go() {
    let mut rig = wanted(Rig::new());
    assert!(!rig.send(Msg::Stop), "the actor is done");
    assert!(!rig.hook.running());
    assert_eq!(rig.hook.stops(), 1);
}

#[test]
fn the_actor_thread_hears_presses_wanted_and_a_stop_clears_it() {
    let (mut live, (_ear, states)) = Live::start(|_| {}, hear::<AssistChanged>);
    assert_eq!(states.recv_timeout(WAIT), Ok(on(false)));
    live.bus.publish(&PressesWanted { on: true }).unwrap();
    synced(&live, &states);
    assert!(live.hook.running(), "wanted presses start the hook");
    live.stop();
    assert!(!live.hook.running(), "the stop let it go");
    let gone = AssistChanged {
        live: false,
        ..on(false)
    };
    assert_eq!(states.try_recv(), Ok(gone));
    live.restart();
    assert_eq!(states.recv_timeout(WAIT), Ok(on(false)));
    assert!(!live.hook.running(), "a new start wants no presses");
    live.bus.publish(&PressesWanted { on: true }).unwrap();
    live.bus.publish(&PressesWanted { on: false }).unwrap();
    synced(&live, &states);
    live.hook.settle(T0);
    synced(&live, &states);
    assert!(!live.hook.running(), "unwanted lets it go");
    live.stop();
}
