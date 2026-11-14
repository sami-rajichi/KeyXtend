//! The microphone's faults (edge case 12) and the worker's death (13): each has its note, the key's light goes off,
//! the pieces already written are offered with Copy, and the bar never hangs.

use super::*;
use kx_module_api::ports::{BarText, CardId, CardUse};

/// The Listen again note for `hint`.
fn listen(hint: BarHint) -> (BarHint, Option<BarButton>) {
    (hint, Some(BarButton::Listen))
}

#[test]
fn each_microphone_fault_at_open_has_its_note() {
    let faults = [
        (SpeechError::NoMic, BarHint::NoMic),
        (SpeechError::MicBusy, BarHint::MicBusy),
        (SpeechError::MicRefused, BarHint::MicRefused),
    ];
    for (error, hint) in faults {
        let mut rig = Rig::new();
        rig.key();
        let out = rig.tell(SpeechNote::Fault(error));
        assert_eq!(calls(&out), [RunCall::Close], "{error:?}");
        assert_eq!(note(&rig.shown()), Some(listen(hint)), "{error:?}");
        assert!(!rig.lit, "{error:?}");
    }
}

#[test]
fn an_unplugged_microphone_turns_the_light_off_with_its_note() {
    let mut rig = Rig::new();
    rig.key();
    rig.speak();
    assert!(rig.lit);
    let out = rig.tell(SpeechNote::Fault(SpeechError::Unplugged));
    assert_eq!(calls(&out), [RunCall::Close]);
    assert!(!rig.lit);
    assert_eq!(note(&rig.shown()), Some(listen(BarHint::Unplugged)));
    let out = rig.bar(BarButton::Listen);
    assert_eq!(opened(&out).len(), 1);
}

#[test]
fn a_death_while_transcribing_offers_the_pieces_with_copy() {
    let mut rig = Rig::new();
    rig.key();
    rig.speak();
    rig.tell(SpeechNote::Piece {
        index: 0,
        text: secret("le début"),
    });
    rig.bar(BarButton::Stop);
    let out = rig.tell(SpeechNote::Fault(SpeechError::Gone));
    assert_eq!(calls(&out), [RunCall::Close]);
    assert!(ask(&out).is_none());
    let VoiceBar::Text(BarText { hint, undo, .. }) = rig.shown() else {
        panic!("no Text bar: {:?}", rig.shown());
    };
    assert_eq!(hint, Some(BarHint::Pieces));
    assert!(!undo);
    assert_eq!(text(&rig.shown()).as_deref(), Some("le début"));
    assert!(!rig.flow.can_again());
    let out = rig.bar(BarButton::Copy);
    assert!(out.contains(&Out::Copy(secret("le début"))));
}

#[test]
fn a_fault_after_pieces_keeps_them_under_its_line() {
    let mut rig = Rig::new();
    rig.key();
    rig.speak();
    rig.tell(SpeechNote::Piece {
        index: 0,
        text: secret("le début"),
    });
    let out = rig.tell(SpeechNote::Fault(SpeechError::Unplugged));
    assert_eq!(calls(&out), [RunCall::Close]);
    assert!(!rig.lit && ask(&out).is_none());
    assert_eq!(kept(&out).len(), 1);
    let VoiceBar::Text(BarText { hint, .. }) = rig.shown() else {
        panic!("no Text bar: {:?}", rig.shown());
    };
    assert_eq!(hint, Some(BarHint::Unplugged));
    let out = rig.bar(BarButton::Copy);
    assert!(out.contains(&Out::Copy(secret("le début"))));
}

#[test]
fn a_death_during_try_again_keeps_the_whole_first_text_in_history() {
    let mut rig = Rig::new();
    rig.dictate("bonjour tout le monde");
    rig.again();
    rig.tell(SpeechNote::Piece {
        index: 0,
        text: secret("bonjour"),
    });
    let out = rig.tell(SpeechNote::Fault(SpeechError::Gone));
    assert!(kept(&out).is_empty());
    assert_eq!(text(&rig.shown()).as_deref(), Some("bonjour"));
}

#[test]
fn a_death_while_listening_with_nothing_written_shows_the_fault() {
    let mut rig = Rig::new();
    rig.key();
    rig.speak();
    let out = rig.tell(SpeechNote::Fault(SpeechError::Timeout));
    assert_eq!(calls(&out), [RunCall::Close]);
    assert!(!rig.lit);
    assert_eq!(note(&rig.shown()), Some((BarHint::Fault, None)));
    assert!(rig.flow.deadline().is_some());
}

#[test]
fn a_death_after_the_text_dims_try_again_and_keeps_the_text() {
    let mut rig = Rig::new();
    rig.dictate("bonjour");
    let out = rig.tell(SpeechNote::Fault(SpeechError::Gone));
    assert!(calls(&out).is_empty());
    assert!(!rig.flow.can_again());
    assert_eq!(text(&rig.shown()).as_deref(), Some("bonjour"));
}

#[test]
fn notes_of_a_closed_dictation_are_dropped() {
    let mut rig = Rig::new();
    rig.dictate("bonjour");
    let old = rig.take;
    rig.bar(BarButton::Close);
    let note = SpeechNote::Stage(SpeechStage::Open);
    rig.on(In::Note { take: old, note });
    assert!(!rig.lit);
    assert_eq!(rig.shown(), VoiceBar::Hidden);
}

#[test]
fn a_failed_card_lets_the_processor_write_with_a_note() {
    let mut rig = Rig::new();
    let card = CardUse {
        id: CardId::new("Vulkan0").unwrap(),
        need_mb: 960,
        margin_mb: 512,
    };
    let mut on_card = ready("parakeet", "fr");
    on_card.card = Some(card.clone());
    rig.book.0.insert("fr".into(), Ok(on_card));
    rig.key();
    rig.speak();
    rig.bar(BarButton::Stop);
    let out = rig.tell(SpeechNote::Fault(SpeechError::CardFailed));
    assert!(out.contains(&Out::Fell(card.id)));
    assert_eq!(rig.shown(), VoiceBar::Transcribing);
    let out = rig.write(&["bonjour"]);
    assert!(ask(&out).is_some());
    let VoiceBar::Text(BarText { hint, .. }) = rig.shown() else {
        panic!("no Text bar");
    };
    assert_eq!(hint, Some(BarHint::Fell));
}
