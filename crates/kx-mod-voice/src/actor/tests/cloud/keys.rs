//! The key and the hidden run: a key that is missing, damaged or holds no key, found as it is read
//! just before `send`, sends nothing, types nothing and keeps the recording for Try again and "Use the local model";
//! a hidden run, or a saved pick of a row not offered (ADR-0050), offers no service and reads no key.

use kx_module_api::ports::{BlobName, SealedError};
use kx_platform_fake::ShelfCall;

use super::*;
use crate::cloud::pick_text;
use crate::keys::KEYS;
use crate::services::{self, tests::NOT_OFFERED};

#[test]
fn a_damaged_key_sends_nothing_types_nothing_and_keeps_the_recording() {
    let mut rig = Rig::build(groq);
    rig.fake.shelf.fail(ShelfCall::Load, SealedError::Damaged);
    rig.dictate();
    assert!(
        rig.fake.cloud.calls().runs.is_empty(),
        "no request, so never an empty key"
    );
    assert!(rig.asks().is_empty());
    assert_eq!(rig.bar(), cloud_note(CloudWhy::Key));
    assert!(rig.fake.speech.runs()[0].held, "the recording stays");
    // The shelf set the damaged blob aside, so Try again finds no key until one is set again.
    rig.act(VoiceClick::Bar(BarButton::Again));
    assert!(rig.fake.cloud.calls().runs.is_empty());
    set_key(&rig.fake, &key());
    rig.fake
        .cloud
        .script(vec![CloudAnswer::text("hello"), CloudAnswer::text("world")]);
    rig.act(VoiceClick::Bar(BarButton::Again));
    assert_eq!(rig.typed_texts(), ["hello world"]);
}

#[test]
fn a_blob_that_holds_no_key_sends_nothing_and_the_local_model_writes_the_speech() {
    let mut rig = Rig::build(groq);
    let (folder, name) = (BlobName::new(KEYS).unwrap(), BlobName::new("groq").unwrap());
    rig.fake
        .shelf
        .plant(&folder, &name, b"not a key\n".to_vec());
    rig.dictate();
    assert!(rig.fake.cloud.calls().runs.is_empty());
    assert_eq!(rig.bar(), cloud_note(CloudWhy::Key));
    rig.act(VoiceClick::Local);
    let agains = rig.fake.speech.runs()[0].agains.clone();
    assert!(
        agains.len() == 1 && agains[0].model.is_some(),
        "the local model writes it"
    );
    let text = secret("hello");
    rig.tell(SpeechNote::Piece { index: 0, text });
    rig.tell(SpeechNote::Done(done(1)));
    assert_eq!(rig.typed_texts(), ["hello"]);
    assert!(rig.fake.cloud.calls().runs.is_empty());
}

#[test]
fn a_saved_pick_of_a_row_not_offered_dictates_locally_reads_no_key_and_sends_nothing() {
    for id in NOT_OFFERED {
        let row = services::table().unwrap().get(id).unwrap();
        let model = row.models.iter().find(|m| m.code("en").is_some()).unwrap();
        let mut rig = Rig::build(|fake, host| {
            host.put("cloud", true.into());
            host.put(
                "services",
                table(&format!("{id} = {{ on = 2026-11-17, terms = 1 }}")),
            );
            let pick = pick_text(id, &model.id);
            host.put("picks", table(&format!("en = \"{pick}\"")));
            let keys = Keys::new(Arc::new(fake.shelf.clone()), false);
            keys.set(id, secret(&key())).unwrap();
        });
        let loads = rig.fake.shelf.calls(ShelfCall::Load);
        rig.talk();
        let local = rig.fake.speech.runs()[0].ask.model.is_some();
        assert!(
            local && rig.mark().is_none(),
            "{id}: the local model listens"
        );
        rig.key();
        rig.tell(SpeechNote::Done(done(1)));
        assert!(rig.fake.cloud.calls().runs.is_empty(), "{id}: nothing sent");
        assert_eq!(
            rig.fake.shelf.calls(ShelfCall::Load),
            loads,
            "{id}: no key read"
        );
    }
}

#[test]
fn a_hidden_run_offers_no_service_and_reads_no_key() {
    let mut rig = Rig::hidden_with(groq);
    let reads = |rig: &Rig| [ShelfCall::Load, ShelfCall::Names].map(|c| rig.fake.shelf.calls(c));
    let before = reads(&rig);
    rig.talk();
    assert!(
        rig.fake.speech.runs()[0].ask.model.is_some(),
        "the local model listens"
    );
    assert_eq!(rig.mark(), None);
    rig.key();
    rig.tell(SpeechNote::Piece {
        index: 0,
        text: secret("hello"),
    });
    rig.tell(SpeechNote::Done(done(1)));
    assert_eq!(rig.typed_texts(), ["hello"]);
    assert!(rig.fake.cloud.calls().runs.is_empty());
    assert_eq!(reads(&rig), before, "no key was listed or read");
}

#[test]
fn a_failed_key_listing_is_no_missing_key_so_the_dictation_goes_on() {
    let mut rig = Rig::build(groq);
    rig.fake.shelf.fail(ShelfCall::Names, SealedError::Io);
    rig.fake
        .cloud
        .script(vec![CloudAnswer::text("hello"), CloudAnswer::text("world")]);
    rig.dictate();
    assert_ne!(
        rig.bar(),
        cloud_note(CloudWhy::Key),
        "a listing fault is no missing key"
    );
    assert_eq!(rig.typed_texts(), ["hello world"]);
}
