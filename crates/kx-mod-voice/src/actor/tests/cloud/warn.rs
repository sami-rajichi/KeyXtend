//! The card's warning through the actor (ruling 11): the first `ElevenLabs` dictation shows it on the bar while it
//! listens and saves it seen at the row's terms number, nothing waiting on it; the next shows none, and a Groq
//! dictation shows only its row's own. Newer terms, once confirmed, show it again.

use super::port::eleven;
use super::*;
use crate::cloud::Cloud;
use crate::services::tests::keyed::raised;
use crate::services::tests::shipped::warn_id;
use crate::services::tests::{LANGS, OWN, TEXT};
use crate::services::{self, Services};
use crate::settings::SERVICES;

/// The id of the warning the bar shows while it listens; `None` for none.
fn warn(rig: &Rig) -> Option<String> {
    match rig.bar() {
        VoiceBar::Listening(listening) => listening.warn.map(|w| w.id),
        other => panic!("listening, not {other:?}"),
    }
}

/// The terms number `id`'s entry saw its warning at, as the actor follows it.
fn seen_at(rig: &Rig, id: &str) -> u16 {
    rig.actor.settings().services[id].warned
}

/// Shipped row `id`'s terms number.
fn terms(id: &str) -> u16 {
    services::table().unwrap().get(id).unwrap().terms
}

#[test]
fn the_first_dictation_shows_the_warning_and_saves_it_seen_and_the_next_shows_none() {
    let mut rig = Rig::build(eleven);
    let answers = || vec![CloudAnswer::text("hello"), CloudAnswer::text("world")];
    rig.fake.cloud.script(answers());
    rig.talk();
    assert_eq!(
        warn(&rig).as_deref(),
        Some(warn_id("elevenlabs")),
        "while it listens"
    );
    let at = terms("elevenlabs");
    assert_eq!(seen_at(&rig, "elevenlabs"), at, "seen at its terms number");
    assert_eq!(rig.saved().services["elevenlabs"].warned, at, "and saved");
    rig.key();
    rig.tell(SpeechNote::Done(done(0)));
    assert_eq!(rig.typed_texts(), ["hello world"], "nothing waited on it");
    rig.fake.cloud.script(answers());
    rig.talk();
    assert_eq!(warn(&rig), None, "the next dictation");
}

#[test]
fn a_groq_dictation_shows_its_rows_warning_alone_and_notes_none_without_one() {
    let mut rig = Rig::build(groq);
    rig.talk();
    let own = services::table()
        .unwrap()
        .get("groq")
        .unwrap()
        .facts
        .warn
        .as_ref();
    assert_eq!(warn(&rig), own.map(|f| f.id.clone()));
    let noted = if own.is_some() { terms("groq") } else { 0 };
    assert_eq!(seen_at(&rig, "groq"), noted, "noted only with one");
}

/// The shipped table with `ElevenLabs`' terms raised by one, kept for the run.
fn newer() -> &'static Services {
    let table = Services::parse(TEXT, &raised(OWN, "elevenlabs"), LANGS).unwrap();
    let raised = table.get("elevenlabs").unwrap().terms;
    assert_eq!(raised, terms("elevenlabs") + 1, "its own row");
    Box::leak(Box::new(table))
}

#[test]
fn newer_terms_once_confirmed_show_the_warning_again_and_save_it_at_their_number() {
    let mut rig = Rig::build(eleven);
    let answers = || vec![CloudAnswer::text("hello"), CloudAnswer::text("world")];
    rig.fake.cloud.script(answers());
    rig.dictate();
    assert_eq!(
        seen_at(&rig, "elevenlabs"),
        terms("elevenlabs"),
        "seen at the first terms"
    );
    let table = newer();
    rig.actor.held.table = Some(table);
    let mut next = rig.actor.settings().clone();
    let today = rig.actor.ports.calendar.now().date;
    assert!(Cloud::new(table, false).confirm(&mut next, "elevenlabs", today));
    rig.actor.set(SERVICES, next.services_value().unwrap());
    rig.fake.cloud.script(answers());
    rig.talk();
    assert_eq!(
        warn(&rig).as_deref(),
        Some(warn_id("elevenlabs")),
        "again, at the new terms"
    );
    let new = table.get("elevenlabs").unwrap().terms;
    assert_eq!(seen_at(&rig, "elevenlabs"), new, "seen at the new number");
}
