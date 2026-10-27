//! An engine the hook gave up: the modes turn off for this session, so one tap on a mode key starts afresh, and held
//! latches get one fresh run at once, once until the user acts again; a late word from an older run changes nothing.
//! The user always hears why a run a mode tap or a latch held went dark, and once per program run when only quiet
//! needs held it.

use crate::live::{Live, WAIT, hear, on, synced};
use crate::rig::{A, CODE, Heard, Rig, SHIFT, T0, T1, down, shown, told, up};
use kx_mod_mouse::Msg;
use kx_module_api::ports::Button::Left;
use kx_module_api::ports::GrabState::{Armed, Off};
use kx_module_api::ports::PointerAct::{Down, KeyDown, KeyUp, Up};
use kx_module_api::ports::{AssistChanged, HookError, PressesWanted, Toggle};
use kx_module_api::{Bus, Notice, keys};

/// A rig whose hook runs only for a latched Shift, with what it said so far taken.
fn latched() -> Rig {
    let mut rig = Rig::new();
    rig.begin();
    rig.send(Msg::Latches(vec![SHIFT]));
    rig.take();
    rig
}

#[test]
fn an_engine_the_hook_gave_up_turns_the_modes_off_for_now_and_a_tap_starts_afresh() {
    let mut rig = Rig::saved_rclick();
    rig.begin();
    rig.take();
    rig.hook.give_up();
    rig.pump();
    assert_eq!(rig.take(), [shown(false, Off), told(keys::MOUSE_RESET)]);
    assert!(!rig.hook.running(), "the run is dropped");
    assert!(!rig.ended, "the actor stays live");
    for _ in 0..2 {
        rig.toggle(Toggle::Grab);
        rig.settle(T0);
    }
    assert!(
        rig.host.sink.writes().is_empty(),
        "the saved Right-click stays"
    );
    rig.toggle(Toggle::RightClick);
    rig.settle(T1);
    assert!(rig.hook.running(), "a fresh run");
    let after = [shown(false, Armed), shown(false, Off), shown(true, Off)];
    assert_eq!(rig.take(), after);
}

#[test]
fn held_latches_get_a_fresh_run_at_once_so_a_click_keeps_its_modifier() {
    let mut rig = latched();
    rig.hook.give_up();
    rig.pump();
    assert_eq!(rig.take(), [shown(false, Off), told(keys::MOUSE_RESET)]);
    assert!(rig.hook.running(), "the latch still needs the hook");
    assert_eq!(rig.hook.starts(), 2, "one fresh run");
    let _ = rig.hook.take_sent();
    assert!(
        rig.feed(down(A), T0),
        "the press goes out inside its modifier"
    );
    assert!(rig.feed(up(A), T1));
    let click = [KeyDown(SHIFT), Down(Left, A), Up(Left, A), KeyUp(SHIFT)];
    assert_eq!(rig.hook.take_sent(), click);
    assert_eq!(rig.take(), [Heard::Used], "the latch is used");
}

#[test]
fn a_fresh_run_for_latches_the_hook_refuses_is_tried_once_and_adds_no_notice() {
    let mut rig = latched();
    rig.hook.fail_next(HookError::Refused(CODE));
    rig.hook.give_up();
    rig.pump();
    assert_eq!(rig.take(), [shown(false, Off), told(keys::MOUSE_RESET)]);
    // A second try would have started, as the refusal is used up.
    assert!(!rig.hook.running(), "tried once");
    rig.send(Msg::Latches(vec![SHIFT]));
    assert!(rig.hook.running(), "the next latch change tries again");
}

#[test]
fn a_give_up_that_repeats_restarts_for_latches_once_until_a_latch_changes() {
    let mut rig = latched();
    let reset = [shown(false, Off), told(keys::MOUSE_RESET)];
    for _ in 0..2 {
        rig.hook.give_up();
        rig.pump();
        assert_eq!(rig.take(), reset, "a held latch always hears why");
    }
    assert!(!rig.hook.running(), "the second give-up starts nothing");
    assert_eq!(rig.hook.starts(), 2, "one fresh run, never a loop");
    rig.send(Msg::Latches(vec![SHIFT]));
    assert!(rig.hook.running(), "a latch change starts afresh");
    rig.hook.give_up();
    rig.pump();
    assert_eq!(rig.hook.starts(), 4, "and earns one more fresh run");
}

#[test]
fn a_mode_tap_after_a_repeated_give_up_earns_the_latches_one_more_fresh_run() {
    let mut rig = latched();
    for _ in 0..2 {
        rig.hook.give_up();
        rig.pump();
    }
    rig.toggle(Toggle::RightClick);
    assert_eq!(rig.hook.starts(), 3, "the tap starts a run");
    rig.hook.give_up();
    rig.pump();
    assert!(rig.hook.running(), "the latch still gets its fresh run");
    assert_eq!(rig.hook.starts(), 4);
}

#[test]
fn a_fault_that_turns_a_latch_dark_leaves_the_quiet_allowance_unused() {
    let mut rig = latched();
    rig.hook.give_up();
    rig.pump();
    rig.send(Msg::Latches(Vec::new()));
    rig.settle(T0);
    assert!(!rig.hook.running(), "the fresh run ended well");
    rig.send(Msg::Presses(true));
    rig.take();
    rig.hook.give_up();
    rig.pump();
    let reset = [shown(false, Off), told(keys::MOUSE_RESET)];
    assert_eq!(rig.take(), reset, "the first quiet fault still tells");
}

#[test]
fn a_fault_after_a_mode_tap_the_engine_never_noted_still_tells() {
    let mut rig = Rig::new();
    rig.begin();
    rig.send(Msg::Presses(true));
    rig.hook.give_up();
    rig.pump();
    rig.take();
    // The tap reaches the fresh run, which faults before it notes the mode.
    rig.toggle(Toggle::RightClick);
    rig.hook.give_up();
    rig.pump();
    let reset = [shown(false, Off), told(keys::MOUSE_RESET)];
    assert_eq!(
        rig.take(),
        reset,
        "the quiet allowance is spent, yet the tap hears why"
    );
}

#[test]
fn a_late_word_from_an_older_runs_engine_leaves_the_live_run_alone() {
    let mut rig = Rig::new();
    rig.begin();
    rig.toggle(Toggle::RightClick);
    rig.settle(T0);
    rig.toggle(Toggle::RightClick);
    rig.settle(T0);
    rig.toggle(Toggle::RightClick);
    rig.take();
    // The first run's engine, dropped with its run, says so again as the second runs.
    rig.send(Msg::Lost(1));
    assert!(rig.hook.running() && !rig.ended);
    assert!(rig.take().is_empty());
}

#[test]
fn a_quiet_fault_tells_once_per_program_run_even_across_module_restarts() {
    let listen = |bus: &Bus| (hear::<AssistChanged>(bus), hear::<Notice>(bus));
    let (mut live, ((_e1, states), (_e2, notices))) = Live::start(|_| {}, listen);
    for round in 0..2 {
        assert_eq!(states.recv_timeout(WAIT), Ok(on(false)), "live");
        live.bus.publish(&PressesWanted { on: true }).unwrap();
        synced(&live, &states);
        live.hook.give_up();
        assert_eq!(states.recv_timeout(WAIT), Ok(on(false)), "the fault");
        synced(&live, &states);
        let heard: Vec<_> = notices.try_iter().map(|n| n.key).collect();
        let want = if round == 0 {
            vec![keys::MOUSE_RESET]
        } else {
            Vec::new()
        };
        assert_eq!(heard, want, "round {round}");
        live.stop();
        assert_eq!(states.try_recv().map(|s| s.live), Ok(false), "stopped");
        live.restart();
    }
    live.stop();
}
