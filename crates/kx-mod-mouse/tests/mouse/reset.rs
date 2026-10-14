//! An engine the hook gave up (ruling 39): the modes turn off for this session, held latches get one fresh run at
//! once, and a late word from an older run changes nothing.

use crate::rig::{A, Heard, Rig, T0, T1, down, shown, told, up};
use kx_mod_mouse::Msg;
use kx_module_api::keys;
use kx_module_api::ports::Button::Left;
use kx_module_api::ports::GrabState::{Armed, Off};
use kx_module_api::ports::PointerAct::{Down, KeyDown, KeyUp, Up};
use kx_module_api::ports::{HookError, Scan, Toggle};

/// Left Shift.
const SHIFT: Scan = Scan::new(0x2A);
/// The OS error code a refused hook carries.
const CODE: i32 = 1428;

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
