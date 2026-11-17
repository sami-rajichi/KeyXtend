//! The key and the hidden run: a key that is missing, damaged or holds no key, found as it is read
//! just before `send`, sends nothing, types nothing and keeps the recording for Try again and "Use the local model";
//! a hidden run offers no service and reads no key.

use kx_module_api::ports::{BlobName, SealedError};
use kx_platform_fake::ShelfCall;

use super::*;
use crate::keys::KEYS;

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
    set_key(&rig.fake, KEY);
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
