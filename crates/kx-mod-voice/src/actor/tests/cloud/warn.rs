//! The card's warning through the actor (ruling 11): the first `ElevenLabs` dictation shows it on the bar while it
//! listens and saves it seen at the row's terms number, nothing waiting on it; the next shows none, and a Groq
//! dictation never does.

use super::port::eleven;
use super::*;

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

#[test]
fn the_first_dictation_shows_the_warning_and_saves_it_seen_and_the_next_shows_none() {
    let mut rig = Rig::build(eleven);
    let answers = || vec![CloudAnswer::text("hello"), CloudAnswer::text("world")];
    rig.fake.cloud.script(answers());
    rig.talk();
    assert_eq!(warn(&rig).as_deref(), Some("trains"), "while it listens");
    assert_eq!(seen_at(&rig, "elevenlabs"), 1, "seen at its terms number");
    assert_eq!(rig.saved().services["elevenlabs"].warned, 1, "and saved");
    rig.key();
    rig.tell(SpeechNote::Done(done(0)));
    assert_eq!(rig.typed_texts(), ["hello world"], "nothing waited on it");
    rig.fake.cloud.script(answers());
    rig.talk();
    assert_eq!(warn(&rig), None, "the next dictation");
}

#[test]
fn a_service_without_a_warning_never_shows_one() {
    let mut rig = Rig::build(groq);
    rig.talk();
    assert_eq!(warn(&rig), None);
    assert_eq!(seen_at(&rig, "groq"), 0, "nothing noted");
}
