//! The actor while word prediction wants presses (spec §6): the hook stays on and each press is told with `ours`, so a
//! click outside our windows ends the word; every press still reaches the app as the modes say. Each start asks for
//! the flag, so any start order works.

use crate::live::{Live, WAIT, hear, on, synced};
use crate::logs::{MESSAGE, lines_of};
use crate::rig::{A, B, CODE, Heard, Rig, SHIFT, T0, T1, T2, down, moved, ours, shown, told, up};
use kx_mod_mouse::Msg;
use kx_module_api::ports::GrabState::Off;
use kx_module_api::ports::{
    AssistChanged, HookError, Point, PointerPressed, PressesAsk, PressesWanted,
};
use kx_module_api::{Bus, Mono, keys};
use std::sync::mpsc::Receiver;
use tracing::Level;

/// The field the flag's log line holds.
const ON: &str = "on";
/// A point whose numbers appear nowhere else in the output.
const SPOT: Point = Point { x: 4817, y: 2963 };

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
    assert!(!rig.feed(down(A), T0), "the press passes");
    assert!(!rig.feed(up(A), T0));
    assert_eq!(rig.take(), [pressed(false)], "scrolling holds");
    rig.send(Msg::Presses(true));
    rig.send(Msg::Scroll(false));
    rig.settle(T1);
    assert!(rig.hook.running(), "presses hold the hook");
    assert!(!rig.feed(ours(B), T1), "the press passes");
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
        for _ in 0..2 {
            rig.hook.fail_next(HookError::Refused(CODE));
            rig.send(Msg::Presses(true));
            assert!(!rig.hook.running());
            rig.send(Msg::Presses(false));
        }
        assert!(rig.take().is_empty(), "the user pressed no mouse key");
        rig.send(Msg::Presses(true));
        assert!(rig.hook.running(), "the next change tries again");
    });
    let warned = lines.iter().filter(|(l, _)| *l == Level::WARN).count();
    assert_eq!(warned, 1, "the second refusal stays silent: {lines:?}");
}

/// The live run's engine faults, as a panic makes the host give it up.
fn fault(rig: &mut Rig) {
    rig.hook.give_up();
    rig.pump();
}

/// The word ends, the hook settles at `now`, and the next word starts.
fn next_word(rig: &mut Rig, now: Mono) {
    rig.send(Msg::Presses(false));
    rig.settle(now);
    rig.send(Msg::Presses(true));
}

/// How many reset notices `heard` holds.
fn resets(heard: &[Heard]) -> usize {
    let reset = told(keys::MOUSE_RESET);
    heard.iter().filter(|h| **h == reset).count()
}

#[test]
fn a_healthy_run_after_a_fault_re_arms_presses_for_the_next_word() {
    let mut rig = wanted(Rig::new());
    fault(&mut rig);
    assert_eq!(rig.hook.starts(), 2, "one fresh run");
    next_word(&mut rig, T1);
    assert!(
        rig.hook.running(),
        "the fresh run ended well, so the next word starts one"
    );
    assert_eq!(rig.hook.starts(), 3);
    assert!(!rig.feed(down(A), T2));
    assert_eq!(rig.take().last(), Some(&pressed(false)));
}

#[test]
fn a_fault_at_every_word_tells_the_reset_once_and_never_loops() {
    let mut rig = wanted(Rig::new());
    let words = 3;
    for _ in 0..words {
        fault(&mut rig);
        next_word(&mut rig, T1);
    }
    assert_eq!(resets(&rig.take()), 1, "once per program run");
    assert_eq!(
        rig.hook.starts(),
        1 + 2 * words,
        "one fresh run per fault, one run per word"
    );
}

#[test]
fn a_fault_on_every_run_starts_nothing_more_until_the_user_acts() {
    let mut rig = wanted(Rig::new());
    for _ in 0..3 {
        while rig.hook.running() {
            fault(&mut rig);
        }
        next_word(&mut rig, T1);
    }
    assert_eq!(resets(&rig.take()), 1);
    assert_eq!(rig.hook.starts(), 2, "the first run and one fresh run");
    rig.send(Msg::Latches(vec![SHIFT]));
    rig.send(Msg::Latches(Vec::new()));
    rig.settle(T2);
    assert!(
        rig.hook.running(),
        "a latch change re-arms it, and presses keep it"
    );
    assert!(!rig.feed(down(A), T2));
    assert_eq!(rig.take(), [pressed(false)]);
}

#[test]
fn a_given_up_engine_while_presses_are_wanted_gets_one_fresh_run() {
    let mut rig = wanted(Rig::new());
    rig.hook.give_up();
    rig.pump();
    assert_eq!(rig.take(), [shown(false, Off), told(keys::MOUSE_RESET)]);
    assert!(rig.hook.running(), "presses still need the hook");
    assert_eq!(rig.hook.starts(), 2, "one fresh run");
    assert!(!rig.feed(down(A), T1));
    assert_eq!(rig.take(), [pressed(false)]);
}

#[test]
fn only_the_flag_s_changes_are_logged_with_no_point() {
    let lines = lines_of(|| {
        let mut rig = Rig::new();
        rig.begin();
        rig.send(Msg::Presses(true));
        assert!(!rig.feed(down(SPOT), T0));
        rig.send(Msg::Presses(true));
        rig.send(Msg::Presses(false));
        rig.settle(T1);
    });
    let flags: Vec<_> = lines.iter().filter(|(_, f)| f.contains_key(ON)).collect();
    let ons: Vec<_> = flags.iter().map(|(_, f)| f[ON].as_str()).collect();
    assert_eq!(ons, ["true", "false"], "{lines:?}");
    let points = [SPOT.x, SPOT.y].map(|n| n.to_string());
    for (_, fields) in &flags {
        let names: Vec<_> = fields.keys().copied().collect();
        assert_eq!(names, [MESSAGE, ON]);
        let shown = |v: &String| points.iter().any(|p| v.contains(p.as_str()));
        assert!(!fields.values().any(shown), "{fields:?}");
    }
}

#[test]
fn a_stop_while_presses_are_wanted_lets_the_hook_go() {
    let mut rig = wanted(Rig::new());
    assert!(!rig.send(Msg::Stop), "the actor is done");
    assert!(!rig.hook.running());
    assert_eq!(rig.hook.stops(), 1);
}

/// Plays word prediction holding a word: it hears the mouse's ask and answers with its flag.
fn answer(live: &Live, asks: &Receiver<PressesAsk>, states: &Receiver<AssistChanged>) {
    assert_eq!(asks.recv_timeout(WAIT), Ok(PressesAsk));
    live.bus.publish(&PressesWanted { on: true }).unwrap();
    synced(live, states);
}

#[test]
fn each_start_asks_for_presses_so_a_restart_mid_word_gets_the_hook_back() {
    let listen = |bus: &Bus| (hear::<AssistChanged>(bus), hear::<PressesAsk>(bus));
    let (mut live, ((_e1, states), (_e2, asks))) = Live::start(|_| {}, listen);
    assert_eq!(states.recv_timeout(WAIT), Ok(on(false)));
    answer(&live, &asks, &states);
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
    assert!(
        !live.hook.running(),
        "a new start wants no presses until answered"
    );
    answer(&live, &asks, &states);
    assert!(
        live.hook.running(),
        "the answer brings it back with no new word"
    );
    live.bus.publish(&PressesWanted { on: false }).unwrap();
    synced(&live, &states);
    live.hook.settle(T0);
    synced(&live, &states);
    assert!(!live.hook.running(), "unwanted lets it go");
    live.stop();
}
