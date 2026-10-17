//! Runs that random ones once failed, kept by name so each stays checked whatever the strategies become.

use kx_module_api::ports::{Button, PointerAct};

use super::super::CFG;
use super::steps::{ENGINE_AT, Script, Step};
use super::{Gesture, common, play};

/// Lagging hooks go back while the hand's press and release still wait ahead in the stream, and the reinstall's
/// batch, sent with our hooks out, presses that button. `SendInput` returns only once that press passed, after the
/// hand's, so nothing of ours stays held after the engine's release panics.
#[test]
fn a_press_sent_with_our_hooks_out_passes_after_the_hands_waiting_input() {
    let release = vec![PointerAct::Down(Button::Right, ENGINE_AT)];
    let steps = [
        Step::Pass(CFG.health_slack_ms + 1),
        Step::Hand(Gesture::Flip(Button::Right)),
        Step::Hand(Gesture::Flip(Button::Right)),
        Step::Check,
        Step::Plan(Script {
            release,
            ..Script::default()
        }),
        Step::Check,
        Step::Panic(true),
        Step::Rit,
        Step::Rit,
        Step::Slip(Gesture::Flip(Button::Left)),
    ];
    for inside in [false, true] {
        common(&play(&steps, inside)).unwrap();
    }
}
