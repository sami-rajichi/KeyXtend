//! The scorer on tiny packs written by `PackWriter`: backoff, folding, capitals, the learned blend and its gates.

#![cfg(feature = "build")]

use std::collections::HashSet;

use kx_module_api::Redacted;
use kx_module_api::ports::TypingMode;

use super::fake::Fake;
use super::*;
use crate::pack::{Flags, PackWriter};
use crate::track::Reading;

mod config;
mod elisions;
mod learned;
mod ranked;
mod scripts;
mod shift;
mod stored;

/// The followers each test pack keeps per context.
const FOLLOWERS: u8 = 8;
/// The chips a test asks for.
const LIMIT: usize = 5;
/// One test word: its spelling, with its capitals, and its frequency byte.
type Word<'a> = (&'a str, u8);
/// One test chain: its context, next word and score.
type Chain<'a> = (&'a [&'a str], &'a str, u8);

/// A pack of `words` and `chains`.
fn pack(iso: &str, words: &[Word<'_>], chains: &[Chain<'_>]) -> Vec<u8> {
    let flags = Flags {
        checked: false,
        chains: !chains.is_empty(),
    };
    let mut writer = PackWriter::new(iso, flags, FOLLOWERS).unwrap();
    for (spelling, freq) in words {
        writer.add_word(spelling, *freq).unwrap();
    }
    for (context, next, score) in chains {
        writer.add_chain(context, next, *score).unwrap();
    }
    writer.finish().unwrap()
}

/// English: `park` is more frequent than `the`, yet `the` follows `in` more often; `walk` follows `in` with a chance
/// between `of`'s share and that share times the backoff.
fn english() -> Vec<u8> {
    let words = [
        ("of", 195),
        ("park", 190),
        ("the", 180),
        ("in", 170),
        ("they", 165),
        ("there", 160),
        ("then", 158),
        ("walk", 140),
        ("run", 140),
        ("fast", 130),
        ("London", 130),
        ("theory", 110),
    ];
    let chains: [Chain<'_>; 4] = [
        (&["in"], "the", 240),
        (&["in"], "walk", 220),
        (&["in"], "park", 200),
        (&["walk", "in"], "park", 250),
    ];
    pack("en", &words, &chains)
}

/// French: accented words, elisions with their followers, and whole words holding an apostrophe; `et` and `unis` are
/// frequent words that never follow an elision.
fn french() -> Vec<u8> {
    let words = [
        ("et", 190),
        ("un", 185),
        ("à", 181),
        ("l'", 181),
        ("a", 175),
        ("au", 169),
        ("école", 150),
        ("unis", 150),
        ("quelqu'un", 145),
        ("écoute", 140),
        ("homme", 140),
        ("jusqu'à", 137),
        ("économie", 130),
        ("jusqu'au", 124),
        ("vois", 120),
        ("écologie", 110),
        ("quelqu'", 100),
        ("jusqu'", 99),
        ("lorsqu'", 98),
    ];
    let chains: [Chain<'_>; 2] = [(&["l'"], "école", 230), (&["quelqu'"], "un", 240)];
    pack("fr", &words, &chains)
}

/// Arabic: the hamza forms of `أن` and `إن`, and words that fold alike.
fn arabic() -> Vec<u8> {
    let words = [
        ("من", 187),
        ("أن", 175),
        ("إن", 156),
        ("أنا", 147),
        ("انتظر", 120),
    ];
    pack("ar", &words, &[])
}

/// A query with one reading: `prefix` after `context`.
fn ask(context: &[&str], prefix: &str) -> Query {
    let reading = Reading {
        context: Redacted::new(context.iter().map(|w| (*w).to_owned()).collect()),
        lead: Redacted::default(),
        prefix: Redacted::new(prefix.to_owned()),
    };
    Query {
        word: Redacted::new(prefix.to_owned()),
        readings: vec![reading],
    }
}

/// The guesses' texts.
fn texts(guesses: &[Guess]) -> Vec<String> {
    guesses.iter().map(|g| g.text.expose().clone()).collect()
}

/// The guesses asked of `pack` in its language with `learned`, for `query` in `mode`.
fn asked(pack: &Pack<'_>, learned: &dyn Learned, query: &Query, mode: TypingMode) -> Vec<Guess> {
    guesses(pack.iso(), Some(pack), learned, query, mode, LIMIT, &CFG)
}

/// The texts guessed from `bytes`'s pack with `learned`, for `query` in `mode`.
fn run(bytes: &[u8], learned: &dyn Learned, query: &Query, mode: TypingMode) -> Vec<String> {
    texts(&asked(&Pack::parse(bytes).unwrap(), learned, query, mode))
}

/// The first text guessed for `prefix` after `context` with nothing learned.
fn first(bytes: &[u8], context: &[&str], prefix: &str) -> Option<String> {
    run(
        bytes,
        &NoLearned,
        &ask(context, prefix),
        TypingMode::default(),
    )
    .into_iter()
    .next()
}

#[test]
fn the_after_in_beats_a_rarer_follower_that_is_more_frequent_alone() {
    let en = english();
    let after_in = run(&en, &NoLearned, &ask(&["in"], ""), TypingMode::default());
    let at = |w: &str| after_in.iter().position(|t| t == w);
    assert_eq!(after_in.first().map(String::as_str), Some("the"));
    assert!(
        at("the") < at("park") && at("park").is_some(),
        "{after_in:?}"
    );
    assert_eq!(first(&en, &["in"], "th").as_deref(), Some("the"));
}

#[test]
fn an_unseen_two_word_context_backs_off_to_one_word_then_to_none() {
    let en = english();
    assert_eq!(first(&en, &["walk", "in"], "").as_deref(), Some("park"));
    assert_eq!(first(&en, &["run", "in"], "").as_deref(), Some("the"));
    assert_eq!(first(&en, &["run", "fast"], "").as_deref(), Some("of"));
    assert_eq!(first(&en, &[], "").as_deref(), Some("of"));
}

#[test]
fn the_backoff_weight_lets_a_follower_beat_a_more_frequent_word() {
    let after_in = run(
        &english(),
        &NoLearned,
        &ask(&["in"], ""),
        TypingMode::default(),
    );
    assert_eq!(
        after_in.get(..3),
        Some(&["the", "walk", "of"].map(str::to_owned)[..])
    );
}

#[test]
fn eco_gives_ecole_first_in_its_correct_spelling() {
    let fr = french();
    let found = run(&fr, &NoLearned, &ask(&[], "eco"), TypingMode::default());
    assert_eq!(found, ["école", "écoute", "économie", "écologie"]);
}

#[test]
fn an_gives_the_hamza_forms_first_and_they_read_right_to_left() {
    let ar = arabic();
    let pack = Pack::parse(&ar).unwrap();
    let found = asked(&pack, &NoLearned, &ask(&[], "ان"), TypingMode::default());
    assert_eq!(texts(&found), ["أن", "إن", "أنا", "انتظر"]);
    assert!(found.iter().all(|g| g.rtl));
    let bytes = english();
    let en = Pack::parse(&bytes).unwrap();
    let latin = asked(&en, &NoLearned, &ask(&[], "th"), TypingMode::default());
    assert!(latin.iter().all(|g| !g.rtl));
}

#[test]
fn a_typed_capital_capitalises_and_a_capitalised_word_keeps_its_form() {
    let en = english();
    assert_eq!(first(&en, &[], "Th").as_deref(), Some("The"));
    assert_eq!(first(&en, &[], "lo").as_deref(), Some("London"));
    assert_eq!(first(&en, &[], "Lo").as_deref(), Some("London"));
    let fr = french();
    assert_eq!(first(&fr, &[], "Eco").as_deref(), Some("École"));
}

#[test]
fn caps_lock_or_two_typed_capitals_with_no_lower_case_give_all_capitals() {
    let en = english();
    let caps = TypingMode {
        caps: true,
        ..TypingMode::default()
    };
    let upper = |found: &[String]| found.iter().all(|t| t.chars().all(char::is_uppercase));
    for prefix in ["", "T", "TH"] {
        let found = run(&en, &NoLearned, &ask(&["in"], prefix), caps);
        assert!(!found.is_empty() && upper(&found), "{prefix:?}: {found:?}");
    }
    let shifted = run(&en, &NoLearned, &ask(&[], "th"), caps);
    assert!(!upper(&shifted), "letters typed in lower case: {shifted:?}");
    assert_eq!(first(&en, &[], "TH").as_deref(), Some("THE"));
    assert_eq!(first(&en, &[], "LO").as_deref(), Some("LONDON"));
}

#[test]
fn two_capitals_beside_a_lower_case_letter_keep_the_words_own_capitals() {
    let en = pack(
        "en",
        &[("McDonald", 120), ("USA", 120), ("usage", 110)],
        &[],
    );
    assert_eq!(first(&en, &[], "McD").as_deref(), Some("McDonald"));
    let us = run(&en, &NoLearned, &ask(&[], "US"), TypingMode::default());
    assert_eq!(us, ["USA", "USAGE"]);
    let nl = pack("nl", &[("IJsselmeer", 120)], &[]);
    assert_eq!(first(&nl, &[], "IJs").as_deref(), Some("IJsselmeer"));
}

#[test]
fn readings_merge_and_each_text_shows_once() {
    let fr = french();
    let mut query = ask(&[], "jusqu'a");
    query.readings.push(Reading {
        context: Redacted::new(vec!["jusqu'".to_owned()]),
        lead: Redacted::new("jusqu'".to_owned()),
        prefix: Redacted::new("a".to_owned()),
    });
    let found = run(&fr, &NoLearned, &query, TypingMode::default());
    assert_eq!(
        found.iter().filter(|t| *t == "jusqu'à").count(),
        1,
        "{found:?}"
    );
    assert!(found.contains(&"jusqu'au".to_owned()), "{found:?}");
}

#[test]
fn the_limit_holds_and_the_texts_are_distinct() {
    let en = english();
    let pack = Pack::parse(&en).unwrap();
    for limit in 0..=LIMIT {
        let found = texts(&guesses(
            pack.iso(),
            Some(&pack),
            &NoLearned,
            &ask(&[], ""),
            TypingMode::default(),
            limit,
            &CFG,
        ));
        assert_eq!(found.len(), limit);
        let unique: HashSet<&String> = found.iter().collect();
        assert_eq!(unique.len(), found.len());
    }
}
