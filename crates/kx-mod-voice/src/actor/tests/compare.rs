//! Compare models on the actor: its run borrows the microphone, so the bar listens and the key glows and stops it; then
//! each ready model writes the recording again, and only the numbers are kept on the shelf. The gear cancels it.

use std::sync::Arc;

use kx_module_api::ports::{
    MAX_BLOB_BYTES, PanelButton, SpeechNote, SpeechStage, VoiceBar, VoiceClick, VoiceLit,
    VoiceSheet, VoiceTab,
};
use kx_module_api::{Redacted, Zeroizing};
use kx_platform_fake::SpeechCall;

use super::Msg;
use super::rig::{Rig, done};
use crate::store::Store;

/// The set text the ui would hand over.
const SET: &str = "hello world";

/// A rig on Compare's page, its recording started.
fn started() -> Rig {
    let mut rig = Rig::new();
    rig.send(Msg::Open);
    rig.act(VoiceClick::Open(VoiceTab::Compare));
    let open = vec![SpeechNote::Stage(SpeechStage::Open)];
    rig.fake.speech.on(SpeechCall::Open, open);
    rig.act(VoiceClick::Compare(Redacted::new(SET.to_owned())));
    rig
}

#[test]
fn compares_recording_listens_on_the_bar_and_the_key_stops_it() {
    let mut rig = started();
    let runs = rig.fake.speech.runs();
    assert!(
        runs.len() == 1 && runs[0].ask.model.is_none(),
        "a listen-only run"
    );
    assert!(matches!(rig.bar(), VoiceBar::Listening(_)));
    assert_eq!(rig.lits().last(), Some(&VoiceLit::Rec));
    rig.tell(SpeechNote::Heard);
    rig.key();
    assert_eq!(rig.fake.speech.runs()[0].stops, 1);
    assert_eq!(rig.lits().last(), Some(&VoiceLit::Off));
    assert_eq!(rig.bar(), VoiceBar::Hidden, "no bar while the models write");
}

#[test]
fn each_model_writes_the_recording_and_only_the_numbers_are_kept() {
    let mut rig = started();
    rig.tell(SpeechNote::Heard);
    rig.key();
    rig.tell(SpeechNote::Done(done(0)));
    assert_eq!(rig.fake.speech.runs()[0].agains.len(), 1);
    let text = Redacted::new(Zeroizing::new(SET.to_owned()));
    rig.tell(SpeechNote::Piece { index: 0, text });
    rig.tell(SpeechNote::Done(done(1)));
    assert!(rig.fake.speech.runs()[0].closed);
    let shelf = Arc::new(rig.fake.shelf.clone());
    let opened = Store::open(shelf, MAX_BLOB_BYTES).unwrap();
    assert_eq!(opened.scores.boards().len(), 1);
    assert!(opened.history.is_empty(), "Compare keeps no text");
}

#[test]
fn the_gear_while_compare_records_ends_it_as_compares_cancel_does() {
    let (mut geared, mut cancelled) = (started(), started());
    geared.tell(SpeechNote::Heard);
    cancelled.tell(SpeechNote::Heard);
    geared.send(Msg::Open);
    cancelled.act(VoiceClick::Button(PanelButton::Cancel));
    let runs = geared.fake.speech.runs();
    assert_eq!(runs, cancelled.fake.speech.runs());
    assert_eq!((runs[0].cancels, runs[0].stops), (1, 0));
    assert!(
        runs[0].closed && runs[0].agains.is_empty(),
        "no model writes it"
    );
    assert!(geared.asks().is_empty(), "nothing is typed");
    let shelf = Arc::new(geared.fake.shelf.clone());
    let opened = Store::open(shelf, MAX_BLOB_BYTES).unwrap();
    assert!(opened.scores.boards().is_empty(), "nothing is scored");
    assert_eq!(geared.lits().last(), Some(&VoiceLit::Off));
    let (page, theirs) = (
        geared.views().pop().unwrap(),
        cancelled.views().pop().unwrap(),
    );
    assert_eq!((&page.bar, &page.panel), (&theirs.bar, &theirs.panel));
    let sheet = page.panel.map(|p| p.sheet);
    assert!(
        matches!(sheet, Some(VoiceSheet::Compare(_))),
        "the panel stays on Compare"
    );
}
