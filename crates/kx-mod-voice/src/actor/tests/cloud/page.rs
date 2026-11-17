//! The key page's effects through the actor (ADR-0047): a set key is sealed and shown as dots, a removed one stops the
//! upload under way (carry 7), the key's test runs in a job that reads the key there and whose late answer is dropped
//! (carry 3), Paste asks the clipboard history as Quick-fill does (carry 9), and a card's link opens (edge 23).

use kx_module_api::ports::{
    ClipTake, ClipTaken, ClipText, CloudButton, KeyCheck, MAX_KEY_BYTES, ServiceMark, ServiceSheet,
    VoiceClick, VoiceSheet, VoiceTab,
};

use super::super::taps::{Heard, held, release, waiting};
use super::*;
use crate::actor::Msg;
use crate::panel::Do;

/// A key-like text, built as the test runs.
fn made_key() -> String {
    ["kx", "page", &"5".repeat(20)].join("-")
}

/// Groq's key as the shelf holds it.
fn sealed(rig: &Rig) -> Option<String> {
    let keys = Keys::new(Arc::new(rig.fake.shelf.clone()), false);
    keys.get("groq").unwrap().map(|k| k.expose().to_string())
}

/// Groq on with its terms confirmed and no key, on its key page.
fn on_lines() -> Rig {
    let mut rig = Rig::build(|_, host| {
        host.put("cloud", true.into());
        host.put("services", table("groq = { on = 2026-11-01, terms = 1 }"));
    });
    rig.send(Msg::Open);
    rig.act(VoiceClick::Open(VoiceTab::Cloud));
    rig.act(VoiceClick::Row(0));
    rig.act(VoiceClick::Service(CloudButton::Lines));
    rig
}

/// The key page as last drawn.
fn lines(rig: &Rig) -> ServiceSheet {
    let panel = rig.views().last().and_then(|v| v.panel.clone());
    match panel.map(|p| p.sheet) {
        Some(VoiceSheet::Service(sheet)) => sheet,
        other => panic!("not a key page: {other:?}"),
    }
}

#[test]
fn a_removed_key_stops_the_upload_under_way_and_nothing_is_typed() {
    let mut rig = Rig::build(groq);
    rig.dictate();
    assert_eq!(rig.fake.cloud.calls().runs[0].state, CloudState::Sending);
    rig.actor.key_do(Do::RemoveKey("groq".into()));
    assert_eq!(rig.fake.cloud.calls().runs[0].state, CloudState::Dropped);
    assert_eq!(sealed(&rig), None);
    assert!(rig.typed_texts().is_empty());
}

#[test]
fn a_key_set_on_its_page_is_sealed_shown_as_dots_and_tested_with_the_key_read_in_the_job() {
    let mut rig = on_lines();
    let key = made_key();
    rig.act(VoiceClick::SetKey(secret(&format!("{key} "))));
    assert_eq!(sealed(&rig).as_deref(), Some(key.as_str()));
    let sheet = lines(&rig);
    let dots = u32::try_from(key.len()).unwrap();
    assert_eq!(
        (sheet.key, sheet.mark),
        (Redacted::new(dots), ServiceMark::Ready)
    );
    rig.fake.cloud.take_key(Some(secret(&key)));
    rig.act(VoiceClick::Service(CloudButton::Test));
    let tests = rig.fake.cloud.calls().tests;
    assert_eq!(tests.len(), 1);
    assert_eq!(
        tests[0].url.expose(),
        "https://api.groq.com/openai/v1/models"
    );
    assert_eq!(lines(&rig).check, KeyCheck::Works);
}

#[test]
fn a_test_runs_off_the_actor_and_its_answer_after_the_key_changed_is_dropped() {
    let mut rig = on_lines();
    rig.act(VoiceClick::SetKey(secret(&made_key())));
    rig.actor.ports.spawn = held;
    rig.act(VoiceClick::Service(CloudButton::Test));
    assert!(
        waiting() && rig.fake.cloud.calls().tests.is_empty(),
        "the test waits in its job"
    );
    assert_eq!(lines(&rig).check, KeyCheck::Testing);
    rig.act(VoiceClick::SetKey(secret(&made_key())));
    release();
    rig.pump();
    assert_eq!(rig.fake.cloud.calls().tests.len(), 1);
    assert_eq!(
        lines(&rig).check,
        KeyCheck::Untested,
        "the answer came for the old key"
    );
}

#[test]
fn paste_asks_the_clipboard_history_and_seals_its_text() {
    let mut rig = on_lines();
    rig.act(VoiceClick::Service(CloudButton::Paste));
    let take = rig.heard().into_iter().find_map(|h| match h {
        Heard::Take(take) => Some(take),
        _ => None,
    });
    let take = take.expect("an ask for the clipboard's text");
    assert_eq!(take, ClipTake::new(take.round, MAX_KEY_BYTES));
    let key = made_key();
    let text = ClipText::Text(secret(&key));
    rig.send(Msg::Taken(ClipTaken {
        round: take.round,
        text,
    }));
    assert_eq!(sealed(&rig).as_deref(), Some(key.as_str()));
    assert!(rig.views().iter().all(|v| !format!("{v:?}").contains(&key)));
}

#[test]
fn a_cards_link_opens_its_facts_page_in_the_browser() {
    let mut rig = on_lines();
    rig.act(VoiceClick::Service(CloudButton::Facts));
    rig.act(VoiceClick::Service(CloudButton::Link));
    let opened = rig.fake.link.opened();
    assert_eq!(opened.len(), 1);
    assert!(opened[0].expose().starts_with("https://"));
}

#[test]
fn switching_the_service_off_stops_the_upload_under_way_and_keeps_its_key() {
    let mut rig = Rig::build(groq);
    rig.dictate();
    assert_eq!(rig.fake.cloud.calls().runs[0].state, CloudState::Sending);
    let off = toml::Value::Table(toml::Table::new());
    rig.actor.dos(vec![Do::Set(crate::settings::SERVICES, off)]);
    assert_eq!(rig.fake.cloud.calls().runs[0].state, CloudState::Dropped);
    assert_eq!(sealed(&rig), Some(key()));
}
