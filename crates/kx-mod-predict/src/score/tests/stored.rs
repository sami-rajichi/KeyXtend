//! Each guess carries the spelling it was stored under, which a forget passes: never the chip's capitals or lead.

use super::*;

/// The guess for `prefix` with nothing learned, from `bytes`'s pack.
fn guess(bytes: &[u8], prefix: &str, mode: TypingMode) -> Option<Guess> {
    let pack = Pack::parse(bytes).unwrap();
    asked(&pack, &NoLearned, &ask(&[], prefix), mode)
        .into_iter()
        .next()
}

/// A guess's text and stored spelling.
fn both(guess: &Guess) -> (&str, &str) {
    (guess.text.expose(), guess.stored.expose())
}

#[test]
fn a_capitalised_chip_keeps_the_lower_case_spelling_it_was_stored_under() {
    let en = english();
    let shift = TypingMode {
        shift: true,
        ..PLAIN
    };
    let next = guess(&en, "", shift).unwrap();
    assert_eq!(both(&next), ("Of", "of"));
    let caps = guess(&en, "TH", PLAIN).unwrap();
    assert_eq!(both(&caps), ("THE", "the"));
    let kept = guess(&en, "lo", PLAIN).unwrap();
    assert_eq!(both(&kept), ("London", "London"));
}

#[test]
fn an_elided_chip_stores_the_word_without_its_lead() {
    let fr = french();
    let mut query = ask(&[], "l'éc");
    query.readings.push(Reading {
        context: Redacted::new(vec!["l'".to_owned()]),
        lead: Redacted::new("l'".to_owned()),
        prefix: Redacted::new("éc".to_owned()),
    });
    let pack = Pack::parse(&fr).unwrap();
    let found = asked(&pack, &NoLearned, &query, PLAIN);
    let first = found.first().unwrap();
    assert_eq!(both(first), ("l'école", "école"));
}
