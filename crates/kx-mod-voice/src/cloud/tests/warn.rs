//! The card's warning as a reminder (ruling 11): a row with `warn` carries it on its first dictation at the row's terms
//! number, not on the next, again once newer terms are confirmed; a row without one never does, and any row with one
//! does.

use super::*;
use crate::services::tests::{LANGS, OWN};

/// The `ElevenLabs` model the tests pick.
const SCRIBE: &str = "scribe_v3";
/// Groq's facts' last line, which a warning follows in the test that gives it one.
const GROQ_LAST: &str = "bills = { id = \"least\", count = { seconds = 10 } }";

/// The defaults with cloud on, `ElevenLabs` confirmed under `rules` and picked for English.
fn eleven_for_english(rules: Cloud<'_>) -> VoiceSettings {
    let mut s = on();
    assert!(rules.confirm(&mut s, "elevenlabs", today()));
    assert!(rules.pick(&mut s, "en", "elevenlabs", SCRIBE));
    s
}

/// The id of the warning that `lang`'s choice carries under `rules`.
fn warns(rules: Cloud<'_>, s: &VoiceSettings, lang: &str) -> Option<String> {
    rules.choice(s, lang).and_then(|c| c.warn).map(|w| w.id)
}

#[test]
fn the_first_dictation_carries_the_cards_warning_and_the_next_does_not() {
    let mut s = eleven_for_english(rules());
    let card = rules().card(&s, row("elevenlabs"), today()).warn;
    let choice = rules().choice(&s, "en").and_then(|c| c.warn);
    assert_eq!(choice, card, "the card's own warning");
    assert!(rules().warned(&mut s, "elevenlabs"));
    assert_eq!(
        s.services["elevenlabs"].warned, 1,
        "seen at its terms number"
    );
    assert_eq!(warns(rules(), &s, "en"), None, "the next dictation");
    assert!(!rules().warned(&mut on(), "elevenlabs"), "a service not on");
}

#[test]
fn newer_terms_confirmed_bring_the_warning_back() {
    let mut s = eleven_for_english(rules());
    assert!(rules().warned(&mut s, "elevenlabs"));
    let own = OWN.replacen("terms = 1", "terms = 2", 1);
    let newer = Services::parse(TEXT, &own, LANGS).unwrap();
    let rules = Cloud::new(&newer, false);
    assert!(rules.choice(&s, "en").is_some_and(|c| c.ask), "asks first");
    assert!(rules.confirm(&mut s, "elevenlabs", today()));
    assert_eq!(warns(rules, &s, "en").as_deref(), Some("trains"));
    assert!(rules.warned(&mut s, "elevenlabs"));
    assert_eq!(s.services["elevenlabs"].warned, 2);
}

#[test]
fn a_row_without_a_warning_never_carries_one_and_any_row_with_one_does() {
    let s = groq_for_french();
    assert_eq!(warns(rules(), &s, "fr"), None, "Groq has none");
    let warn = "warn = { id = \"trains\", text = { switch = \"Data use\" } }";
    let text = TEXT.replacen(GROQ_LAST, &format!("{GROQ_LAST}\n{warn}"), 1);
    let warned = parse(&text).unwrap();
    let rules = Cloud::new(&warned, false);
    assert_eq!(
        warns(rules, &s, "fr").as_deref(),
        Some("trains"),
        "from its row alone"
    );
}
