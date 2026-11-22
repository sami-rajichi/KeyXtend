//! The bar names the model on this PC (owner, 2026-11-22): while a local dictation prepares, listens or writes, its
//! chip carries the model's short name, and it goes once the text or a note shows.

use kx_module_api::ports::{BarHint, BarMark, SpeechNote, SpeechStage, VoiceBar};
use kx_module_api::{Redacted, Zeroizing};

use super::rig::{Rig, done, english};

/// The chip of the last view told.
fn mark(rig: &Rig) -> Option<BarMark> {
    rig.views().last().and_then(|v| v.mark.clone())
}

/// English's default model as the bar names it.
fn device() -> BarMark {
    BarMark::Device(Redacted::new(english().short.clone()))
}

#[test]
fn a_local_dictation_names_its_model_from_start_to_its_text() {
    let mut rig = Rig::new();
    assert_eq!(mark(&rig), None, "no dictation, no chip");
    rig.key();
    assert_eq!(
        (rig.bar(), mark(&rig)),
        (VoiceBar::Hidden, Some(device())),
        "preparing"
    );
    rig.tell(SpeechNote::Stage(SpeechStage::Open));
    rig.tell(SpeechNote::Heard);
    assert!(matches!(rig.bar(), VoiceBar::Listening(_)));
    assert_eq!(mark(&rig), Some(device()), "listening");
    rig.key();
    assert_eq!(
        (rig.bar(), mark(&rig)),
        (VoiceBar::Transcribing, Some(device()))
    );
    let text = Redacted::new(Zeroizing::new("hello".to_owned()));
    rig.tell(SpeechNote::Piece { index: 0, text });
    rig.tell(SpeechNote::Done(done(1)));
    assert!(matches!(rig.bar(), VoiceBar::Text(_)));
    assert_eq!(mark(&rig), None, "the text shows alone");
}

#[test]
fn a_local_note_shows_no_chip() {
    let mut rig = Rig::new();
    rig.listen();
    rig.key();
    rig.tell(SpeechNote::Done(done(0)));
    assert!(matches!(rig.bar(), VoiceBar::Note(n) if n.hint == BarHint::Nothing));
    assert_eq!(mark(&rig), None);
}
