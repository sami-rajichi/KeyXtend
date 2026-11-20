//! Removing a key through the actor (voice-a I1, final review I1): a removal the shelf refuses keeps the key listed,
//! shown and in use, and says so; at the start, the keys of rows no longer offered are deleted, but never in a hidden
//! run.

use kx_module_api::ports::{BarHint, BlobName, CloudButton, SealedError, VoiceClick, VoiceTab};
use kx_platform_fake::ShelfCall;

use super::*;
use crate::actor::Msg;
use crate::keys::KEYS;
use crate::services::tests::NOT_OFFERED;
use crate::services::tests::shipped::first;

/// Groq on with its key, on its key page.
fn on_lines() -> Rig {
    let mut rig = Rig::build(groq);
    rig.send(Msg::Open);
    rig.act(VoiceClick::Open(VoiceTab::Cloud));
    rig.act(VoiceClick::Row(0));
    rig.act(VoiceClick::Service(CloudButton::Lines));
    rig
}

/// The note band and the key's dots as last drawn.
fn shown(rig: &Rig) -> (Option<BarHint>, u32) {
    let panel = rig.views().last().and_then(|v| v.panel.clone()).unwrap();
    let dots = match panel.sheet {
        kx_module_api::ports::VoiceSheet::Service(sheet) => *sheet.key.expose(),
        other => panic!("not a key page: {other:?}"),
    };
    (panel.note, dots)
}

#[test]
fn a_removal_the_shelf_refuses_keeps_the_key_shown_listed_and_in_use_and_says_so() {
    let mut rig = on_lines();
    let dots = u32::try_from(key().len()).unwrap();
    assert_eq!(shown(&rig), (None, dots));
    rig.fake.shelf.fail(ShelfCall::Remove, SealedError::Io);
    rig.act(VoiceClick::Service(CloudButton::Remove));
    assert_eq!(shown(&rig), (Some(BarHint::Fault), dots));
    assert!(rig.actor.held.keyed.contains("groq"));
    let keys = Keys::new(Arc::new(rig.fake.shelf.clone()), false);
    assert!(keys.has("groq").unwrap(), "the blob is still there");
}

#[test]
fn a_removal_that_works_clears_the_dots_and_the_listing() {
    let mut rig = on_lines();
    rig.act(VoiceClick::Service(CloudButton::Remove));
    assert_eq!(shown(&rig), (None, 0));
    assert!(!rig.actor.held.keyed.contains("groq"));
    let folder = BlobName::new(KEYS).unwrap();
    assert_eq!(
        rig.fake
            .shelf
            .blob(&folder, &BlobName::new("groq").unwrap()),
        None
    );
}

/// Plants a key blob for each of `ids`, as an earlier run sealed them.
fn plant(fake: &Fakes, ids: &[&str]) {
    let folder = BlobName::new(KEYS).unwrap();
    for id in ids {
        let name = BlobName::new(id).unwrap();
        fake.shelf.plant(&folder, &name, key().into_bytes());
    }
}

/// The services whose key blob the shelf still holds, read without a call.
fn left(rig: &Rig, ids: &[&str]) -> Vec<String> {
    let folder = BlobName::new(KEYS).unwrap();
    let held = |id: &&&str| (rig.fake.shelf.blob(&folder, &BlobName::new(id).unwrap())).is_some();
    ids.iter().filter(held).map(|id| (*id).to_owned()).collect()
}

#[test]
fn the_start_deletes_the_keys_of_rows_no_longer_offered_and_keeps_the_rest() {
    let hidden = NOT_OFFERED[0];
    let ids = [first().id.as_str(), hidden, "gone-row"];
    let rig = Rig::build(|fake, _| plant(fake, &ids));
    assert_eq!(
        left(&rig, &ids),
        [ids[0]],
        "only the offered row's key stays"
    );
}

#[test]
fn a_hidden_run_deletes_no_key_at_its_start() {
    let ids = [NOT_OFFERED[0], "gone-row"];
    let rig = Rig::hidden_with(|fake, _| plant(fake, &ids));
    assert_eq!(left(&rig, &ids), ids);
    assert_eq!(rig.fake.shelf.calls(ShelfCall::Remove), 0);
}
