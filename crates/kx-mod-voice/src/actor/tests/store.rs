//! The store on the voice shelf: a text and its count are kept, a recording with Keep recordings, nothing on a locked
//! shelf, a damaged text is dropped with its note, and a failed save, History's or a count's, tells once and is tried
//! again at the stop.

use std::sync::Arc;

use kx_module_api::keys;
use kx_module_api::ports::{
    BlobName, MAX_BLOB_BYTES, PanelButton, SealedError, VoiceBar, VoiceClick, VoiceTab,
};
use kx_platform_fake::ShelfCall;

use super::Msg;
use super::rig::Rig;
use crate::store::{Opened, Store};

/// What a later run would read from the rig's shelf.
fn reopen(rig: &Rig) -> Opened {
    Store::open(Arc::new(rig.fake.shelf.clone()), MAX_BLOB_BYTES).unwrap()
}

#[test]
fn a_dictated_text_and_its_count_are_kept_on_the_shelf() {
    let mut rig = Rig::new();
    rig.speak("hello");
    let opened = reopen(&rig);
    let texts: Vec<&str> = (opened.history.entries().iter())
        .map(|e| e.text.expose().as_str())
        .collect();
    assert_eq!(texts, ["hello"]);
    assert_eq!(opened.counts.rows().len(), 1);
}

#[test]
fn keep_recordings_hands_the_sound_to_the_store() {
    let mut rig = Rig::build(|_, host| host.put("keep_voice", true.into()));
    rig.fake.speech.set_sound(vec![vec![1, 2, 3, 4]]);
    rig.speak("hello");
    assert_eq!(rig.fake.speech.runs()[0].keeps, 1);
    let opened = reopen(&rig);
    assert!(opened.history.entries()[0].sound.is_some());
}

#[test]
fn a_locked_shelf_keeps_nothing_and_says_so() {
    let mut rig = Rig::build(|fake, host| {
        fake.shelf.fail(ShelfCall::Load, SealedError::Locked);
        host.put("keep_voice", true.into());
    });
    assert_eq!(rig.notices(), [keys::VOICE_LOCKED]);
    rig.speak("hello");
    assert_eq!(
        rig.fake.speech.runs()[0].keeps,
        0,
        "the sound stays in the worker"
    );
    assert!(reopen(&rig).history.is_empty());
}

#[test]
fn a_damaged_text_is_dropped_with_its_note() {
    let rig = Rig::build(|fake, _| {
        let (texts, one) = (BlobName::new("texts").unwrap(), BlobName::new("1").unwrap());
        fake.shelf.plant(&texts, &one, vec![0xFF; 3]);
    });
    assert_eq!(rig.notices(), [keys::VOICE_DAMAGED]);
}

#[test]
fn a_failed_save_tells_once_and_is_tried_again_at_the_stop() {
    let mut rig = Rig::new();
    rig.speak("hello");
    rig.send(Msg::Open);
    rig.act(VoiceClick::Open(VoiceTab::History));
    rig.act(VoiceClick::Row(0));
    rig.fake.shelf.fail(ShelfCall::Save, SealedError::Io);
    rig.act(VoiceClick::Button(PanelButton::Pin));
    assert_eq!(rig.notices(), [keys::VOICE_UNSAVED]);
    assert!(!reopen(&rig).history.entries()[0].pinned);
    rig.send(Msg::Quit);
    assert!(reopen(&rig).history.entries()[0].pinned);
    assert_eq!(rig.notices(), [keys::VOICE_UNSAVED]);
}

#[test]
fn a_count_that_failed_to_save_is_saved_again_at_the_stop() {
    let mut rig = Rig::build(|_, host| host.put("history", false.into()));
    rig.fake.shelf.fail(ShelfCall::Save, SealedError::Io);
    rig.speak("hello");
    assert!(reopen(&rig).counts.rows().is_empty());
    rig.send(Msg::Quit);
    assert_eq!(reopen(&rig).counts.rows().len(), 1);
    assert_eq!(rig.notices(), [keys::VOICE_UNSAVED]);
}

#[test]
fn a_hidden_run_keeps_no_text_or_count() {
    let mut rig = Rig::hidden();
    rig.speak("hello");
    assert!(matches!(rig.bar(), VoiceBar::Text(_)));
    let opened = reopen(&rig);
    assert!(opened.history.is_empty() && opened.counts.rows().is_empty());
}
