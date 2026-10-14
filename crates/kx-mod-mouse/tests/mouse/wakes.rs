//! When the engine's notes reach the bus: the fake hook settles only when the real host would wake, so a note that
//! waits for the next health tick shows here.

use crate::rig::{A, B, Heard, Rig, T0, T1, T2, down, moved, shown};
use kx_module_api::ports::GrabState::Carrying;
use kx_module_api::ports::{AssistNote, PointerMoved, Toggle};

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
