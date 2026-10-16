//! When the engine's notes reach the bus: the fake hook settles only when the real host would wake, after an event or
//! a let-go mid-run, so a note that waits for the next health tick shows here.

use crate::rig::{A, B, Heard, Rig, T0, T1, T2, down, due, moved, shown, up};
use kx_mod_mouse::ESC;
use kx_module_api::ports::GrabState::{Carrying, Off};
use kx_module_api::ports::{AssistNote, Button, PointerAct, PointerMoved, Toggle};

#[test]
fn a_move_and_a_pick_up_reach_the_bus_with_no_wait_for_a_tick() {
    let mut rig = Rig::new();
    rig.begin();
    rig.toggle(Toggle::Grab);
    rig.settle(T0);
    rig.take();
    let _ = rig.feed(moved(A), T1);
    let badge = [Heard::Moved(PointerMoved { at: A })];
    assert_eq!(rig.take(), badge, "the badge follows the pointer at once");
    assert!(!rig.feed(down(B), T2), "the pick-up passes");
    let picked = [
        Heard::Note(AssistNote::Picked { at: B }),
        shown(false, Carrying),
    ];
    assert_eq!(rig.take(), picked, "the pick-up sounds and lights at once");
}

#[test]
fn a_let_go_mid_run_cancels_a_carry_tells_the_new_state_and_the_engine_works_on() {
    let mut rig = Rig::saved_rclick();
    rig.begin();
    rig.toggle(Toggle::Grab);
    rig.settle(T0);
    let _ = rig.feed(down(A), T1);
    let _ = rig.feed(up(A), T1);
    rig.take();
    let _ = rig.hook.take_sent();
    rig.hook.let_go(T2);
    rig.pump();
    let cancel = [
        PointerAct::KeyDown(ESC),
        PointerAct::KeyUp(ESC),
        PointerAct::Up(Button::Left, A),
    ];
    assert_eq!(rig.hook.take_sent(), cancel, "Esc first, so nothing drops");
    let told = [Heard::Note(AssistNote::Cancelled), shown(true, Off)];
    assert_eq!(rig.take(), told, "Right-click is back");
    assert!(rig.hook.running(), "the run goes on");
    let _ = rig.feed(down(B), T2);
    rig.settle(due(&rig.settings.clone(), T2));
    let click = PointerAct::Down(Button::Right, B);
    assert!(
        rig.hook.sent().contains(&click),
        "a still hold right-clicks again"
    );
}
