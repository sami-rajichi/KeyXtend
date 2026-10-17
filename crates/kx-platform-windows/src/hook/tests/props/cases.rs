//! Runs that random ones once failed, kept by name so each stays checked whatever the strategies become.

use kx_module_api::ports::{Button, PointerAct};

use super::super::CFG;
use super::steps::{ENGINE_AT, Script, Step};
use super::{
    End, Gesture, common, in_hand_order, no_press_after_its_release,
    nothing_the_hand_let_go_stays_down, play,
};

/// Checks what every calm run must keep, for a run with no swallowed or missed echo.
fn calm(end: &End) {
    assert!(in_hand_order(end), "{:?}", end.apps);
    assert_eq!(end.lost, 0, "every echo came back");
    assert!(nothing_the_hand_let_go_stays_down(end), "{:?}", end.apps);
    common(end).unwrap();
}

/// Lagging hooks go back while the hand's press and release still wait in the stream, and the reinstall's batch, sent
/// with our hooks out, presses that button. `SendInput` returns only once that press passed, so the hand's release
/// lets it go, and nothing of ours stays held after the engine's release panics.
#[test]
fn a_press_sent_with_our_hooks_out_passes_before_the_hands_waiting_input() {
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

/// Lagging hooks go back while the hand's press waits in the stream, and its release lands right after the
/// reinstall's batch. That press still reaches the hooks put back, so the engine, which drops every event, drops both.
#[test]
fn the_hands_press_waiting_at_a_reinstall_reaches_the_hooks_put_back() {
    let release = vec![PointerAct::Move(ENGINE_AT)];
    let steps = [
        Step::Pass(CFG.health_slack_ms + 1),
        Step::Hand(Gesture::Flip(Button::Left)),
        Step::Slip(Gesture::Flip(Button::Left)),
        Step::Check,
        Step::Plan(Script {
            drop: true,
            release,
            ..Script::default()
        }),
        Step::Check,
        Step::Hand(Gesture::Flip(Button::Left)),
        Step::Hand(Gesture::Flip(Button::Left)),
    ];
    for inside in [false, true] {
        calm(&play(&steps, inside));
    }
}

/// Our press waits in the stream past its echo's wait, then lagging hooks go back and the engine's release panics, so
/// the reinstall lets that press go with our hooks out. That Up never overtakes our own press, so nothing stays held.
#[test]
fn a_let_go_sent_with_our_hooks_out_follows_our_own_waiting_press() {
    let stall = Step::Stall(CFG.echo_ms + 1);
    let steps = [
        Step::Plan(Script {
            on: vec![PointerAct::Move(ENGINE_AT)],
            settle: vec![PointerAct::Move(ENGINE_AT)],
            ..Script::default()
        }),
        Step::Plan(Script {
            settle: vec![PointerAct::Down(Button::Left, ENGINE_AT)],
            ..Script::default()
        }),
        Step::Turn,
        stall.clone(),
        Step::Panic(true),
        Step::Plan(Script::default()),
        stall,
        Step::Flood(1),
        Step::Check,
        Step::Check,
    ];
    let end = play(&steps, false);
    assert!(no_press_after_its_release(&end), "{:?}", end.apps);
    assert!(nothing_the_hand_let_go_stays_down(&end), "{:?}", end.apps);
    common(&end).unwrap();
}
