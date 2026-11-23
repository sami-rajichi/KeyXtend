//! The owner's case from task 24's check through the actor: a management key's test fails with its own answer and
//! keeps the key for a new one to replace, and its dictation is refused as a bad key, so it never reads as working.

use kx_module_api::ports::{CloudButton, KeyCheck, ServiceMark, VoiceSheet, VoiceTab};

use super::*;
use crate::actor::Msg;

/// The API key the fake service takes in place of the management one, built as the test runs.
fn api_key() -> String {
    ["kx", "api", "key"].join("-")
}

#[test]
fn a_management_keys_test_fails_on_its_own_and_its_dictation_reads_as_a_refused_key() {
    let mut rig = Rig::build(groq);
    rig.send(Msg::Open);
    rig.act(VoiceClick::Open(VoiceTab::Cloud));
    rig.act(VoiceClick::Row(0));
    rig.act(VoiceClick::Service(CloudButton::Lines));
    // Groq stands in for `OpenRouter`; the worker answers a management key's test so.
    rig.fake.cloud.script_test(Err(CloudFault::Admin));
    rig.act(VoiceClick::Service(CloudButton::Test));
    let panel = rig.views().last().and_then(|v| v.panel.clone());
    let Some(VoiceSheet::Service(page)) = panel.map(|p| p.sheet) else {
        panic!("the key page shows");
    };
    let dots = u32::try_from(key().len()).unwrap();
    assert_eq!(
        (page.check, page.mark, *page.key.expose()),
        (KeyCheck::Admin, ServiceMark::Ready, dots),
        "the key stays"
    );
    rig.send(Msg::Close);
    // The service takes only an API key, so it refuses the management key's dictation.
    rig.fake.cloud.take_key(Some(secret(&api_key())));
    rig.dictate();
    assert_eq!(rig.fake.cloud.calls().runs.len(), 1);
    assert!(rig.asks().is_empty(), "nothing typed");
    assert_eq!(rig.bar(), cloud_note(CloudWhy::Key));
}
