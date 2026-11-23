//! Preparing (part 2's edges 26 and 27): from Start until the microphone opens no bar shows and the key glows amber; a
//! loading model is no error, the key then cancels with nothing sent, and a microphone that never opens within the
//! job's wait ends in its note with the light out.

use super::*;

/// The run's notes while a fresh job loads its model: the stage, then a card's warm-up.
fn loading() -> [SpeechNote; 2] {
    [
        SpeechNote::Stage(SpeechStage::Loading),
        SpeechNote::Warming { percent: 40 },
    ]
}

/// Waits out the key's debounce, so the next click counts.
fn past_debounce(rig: &Rig) {
    rig.clock
        .advance(VoiceSettings::from_defaults().unwrap().debounce());
}

#[test]
fn until_the_microphone_opens_no_bar_shows_and_the_key_glows_amber() {
    let mut rig = Rig::new();
    let out = rig.key();
    assert_eq!(opened(&out).len(), 1, "the run opens at once");
    assert_eq!(rig.shown(), VoiceBar::Hidden);
    assert_eq!(rig.light, VoiceLit::Prep);
    assert!(rig.flow.dictating(), "busy, so the gear and Compare see it");
    assert_eq!(rig.flow.deadline(), None, "no hint and no fade yet");
    rig.wait(5_000);
    rig.tell(SpeechNote::Stage(SpeechStage::Open));
    assert!(matches!(rig.shown(), VoiceBar::Listening(l) if l.elapsed_ms == 0));
    assert_eq!(rig.light, VoiceLit::Rec);
}

#[test]
fn a_loading_model_is_no_error_and_the_key_stays_amber() {
    let mut rig = Rig::new();
    rig.key();
    let mut out = Vec::new();
    for note in loading() {
        out.extend(rig.tell(note));
    }
    out.extend(rig.tick_after(60_000));
    assert!(calls(&out).is_empty(), "the run goes on");
    assert_eq!(rig.shown(), VoiceBar::Hidden, "no note");
    assert_eq!(rig.light, VoiceLit::Prep);
    rig.tell(SpeechNote::Stage(SpeechStage::Open));
    assert!(matches!(rig.shown(), VoiceBar::Listening(_)));
}

#[test]
fn the_key_while_preparing_cancels_with_nothing_sent() {
    let mut rig = Rig::new();
    rig.key();
    past_debounce(&rig);
    let out = rig.key();
    assert_eq!(calls(&out), [RunCall::Cancel, RunCall::Close]);
    assert!(opened(&out).is_empty() && ask(&out).is_none() && kept(&out).is_empty());
    assert_eq!((rig.shown(), rig.light), (VoiceBar::Hidden, VoiceLit::Off));
    assert!(!rig.flow.dictating());
    rig.speak();
    let out = rig.write(&["trop tard"]);
    assert!(ask(&out).is_none() && kept(&out).is_empty(), "a late note");
    assert!(!out.iter().any(|o| matches!(o, Out::Count(_))));
    assert_eq!((rig.shown(), rig.light), (VoiceBar::Hidden, VoiceLit::Off));
}

#[test]
fn a_microphone_that_never_opens_ends_in_its_note_and_the_light_goes_out() {
    let faults = [
        (SpeechError::Timeout, (BarHint::Fault, None)),
        (
            SpeechError::NoMic,
            (BarHint::NoMic, Some(BarButton::Listen)),
        ),
        (SpeechError::Gone, (BarHint::Fault, None)),
    ];
    for (error, want) in faults {
        let mut rig = Rig::new();
        rig.key();
        rig.tell(SpeechNote::Stage(SpeechStage::Loading));
        let out = rig.tell(SpeechNote::Fault(error));
        assert_eq!(calls(&out), [RunCall::Close], "{error:?}");
        assert_eq!(note(&rig.shown()), Some(want), "{error:?}");
        assert_eq!(rig.light, VoiceLit::Off, "{error:?}: never stuck amber");
        assert!(rig.flow.deadline().is_some(), "{error:?}: the note fades");
    }
}

#[test]
fn listen_again_prepares_too_and_the_bar_waits_for_the_microphone() {
    let mut rig = Rig::new();
    rig.key();
    rig.tell(SpeechNote::Fault(SpeechError::MicBusy));
    let out = rig.bar(BarButton::Listen);
    assert_eq!(opened(&out).len(), 1);
    assert_eq!((rig.shown(), rig.light), (VoiceBar::Hidden, VoiceLit::Prep));
    rig.tell(SpeechNote::Stage(SpeechStage::Open));
    assert!(matches!(rig.shown(), VoiceBar::Listening(_)) && rig.lit);
}

#[test]
fn a_late_starter_hears_the_amber_light_again() {
    let mut rig = Rig::new();
    rig.key();
    let out = rig.on(In::Ask);
    assert!(out.contains(&Out::Lit(VoiceLit::Prep)), "{out:?}");
}
