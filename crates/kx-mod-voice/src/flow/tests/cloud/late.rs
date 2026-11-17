//! A hand-over that comes after "Use the local model": the pick left while the run still held its speech, the user
//! chose the local model, then the run's end arrives late. That end is taken in quietly: nothing is counted, closed or
//! sent, and the local model's text is typed once with the recording kept.

use super::*;

/// The model ids counted among `out`.
fn counted(out: &[Out]) -> Vec<String> {
    let one = |o: &Out| match o {
        Out::Count(count) => Some(count.model.expose().clone()),
        _ => None,
    };
    out.iter().filter_map(one).collect()
}

/// The pick leaves French while `rig`'s dictation runs, so the bar says Off before the run hands its speech over.
fn pick_leaves(rig: &mut Rig) {
    rig.book.1.clear();
    let out = rig.on(In::Settings(Box::new(
        VoiceSettings::from_defaults().unwrap(),
    )));
    assert!(uploads(&out).is_empty());
    assert_eq!(
        rig.shown(),
        cloud_note(CloudWhy::Off, LocalUse::Ready, true)
    );
}

/// "Use the local model", then the late hand-over, then the local model writes: checked as the module promises.
fn local_then_late_hand_over(rig: &mut Rig) {
    let out = rig.click(VoiceClick::Local);
    assert!(matches!(calls(&out)[..], [RunCall::Again(_)]));
    let late = rig.tell(SpeechNote::Done(done(0)));
    assert!(counted(&late).is_empty(), "the hand-over counts nothing");
    assert!(calls(&late).is_empty() && uploads(&late).is_empty());
    assert_eq!(rig.shown(), VoiceBar::Transcribing);
    let out = rig.write(&["en local"]);
    assert_eq!(
        counted(&out),
        ["parakeet"],
        "counted once, under the local model"
    );
    assert!(ask(&out).is_some());
    assert_eq!(text(&rig.shown()).as_deref(), Some("en local"));
    assert!(rig.flow.can_again(), "the recording stays");
}

#[test]
fn a_hand_over_after_the_pick_left_while_listening_keeps_the_local_models_text() {
    let mut rig = Rig::cloud();
    rig.listen();
    pick_leaves(&mut rig);
    local_then_late_hand_over(&mut rig);
}

#[test]
fn a_hand_over_after_the_pick_left_after_stop_keeps_the_local_models_text() {
    let mut rig = Rig::cloud();
    rig.listen();
    rig.wait(1_000);
    assert_eq!(calls(&rig.key()), [RunCall::Stop]);
    pick_leaves(&mut rig);
    local_then_late_hand_over(&mut rig);
}
