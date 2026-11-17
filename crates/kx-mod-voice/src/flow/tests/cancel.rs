//! Cancel (edge case 5): it types nothing and offers "Transcribe anyway" until the bar closes; Cancel while
//! transcribing drops the writing, and the recording stays for Transcribe anyway.

use super::*;

/// The Cancelled note with Transcribe anyway.
fn cancelled() -> (BarHint, Option<BarButton>) {
    (BarHint::Cancelled, Some(BarButton::Anyway))
}

#[test]
fn cancel_while_listening_types_nothing_and_keeps_the_recording() {
    let mut rig = Rig::new();
    rig.key();
    rig.speak();
    let out = rig.bar(BarButton::Cancel);
    assert_eq!(calls(&out), [RunCall::Cancel]);
    assert!(ask(&out).is_none() && kept(&out).is_empty());
    assert!(!rig.lit);
    assert_eq!(note(&rig.shown()), Some(cancelled()));
}

#[test]
fn transcribe_anyway_writes_the_same_recording_and_types_it() {
    let mut rig = Rig::new();
    let first = opened(&rig.key()).remove(0);
    rig.speak();
    rig.bar(BarButton::Cancel);
    let out = rig.bar(BarButton::Anyway);
    assert_eq!(calls(&out), [RunCall::Again(first)]);
    assert_eq!(rig.shown(), VoiceBar::Transcribing);
    let out = rig.write(&["quand même"]);
    let typed = ask(&out).unwrap();
    assert_eq!(typed.erase, 0);
    assert_eq!(typed.text.expose().as_str(), suffixed("quand même"));
}

#[test]
fn cancel_with_no_speech_offers_nothing_to_write() {
    let mut rig = Rig::new();
    rig.key();
    rig.tell(SpeechNote::Stage(SpeechStage::Open));
    rig.bar(BarButton::Cancel);
    assert_eq!(note(&rig.shown()), Some((BarHint::Cancelled, None)));
    assert!(calls(&rig.bar(BarButton::Anyway)).is_empty());
}

#[test]
fn cancel_while_transcribing_drops_the_writing() {
    let mut rig = Rig::new();
    rig.key();
    rig.speak();
    rig.bar(BarButton::Stop);
    rig.tell(SpeechNote::Piece {
        index: 0,
        text: secret("début"),
    });
    let out = rig.bar(BarButton::Cancel);
    assert_eq!(calls(&out), [RunCall::Cancel]);
    assert_eq!(note(&rig.shown()), Some(cancelled()));
    let out = rig.write(&["trop tard"]);
    assert!(ask(&out).is_none() && kept(&out).is_empty());
    assert_eq!(note(&rig.shown()), Some(cancelled()));
}

#[test]
fn anyway_after_cancel_while_transcribing_writes_afresh() {
    let mut rig = Rig::new();
    rig.key();
    rig.speak();
    rig.bar(BarButton::Stop);
    rig.tell(SpeechNote::Piece {
        index: 0,
        text: secret("début"),
    });
    rig.bar(BarButton::Cancel);
    rig.bar(BarButton::Anyway);
    let out = rig.write(&["tout le texte"]);
    let typed = ask(&out).unwrap();
    assert_eq!(typed.text.expose().as_str(), suffixed("tout le texte"));
}

#[test]
fn closing_the_cancelled_bar_wipes_the_recording() {
    let mut rig = Rig::new();
    rig.key();
    rig.speak();
    rig.bar(BarButton::Cancel);
    let out = rig.bar(BarButton::Close);
    assert_eq!(calls(&out), [RunCall::Close]);
    assert_eq!(rig.shown(), VoiceBar::Hidden);
}
