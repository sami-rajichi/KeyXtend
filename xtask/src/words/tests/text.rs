//! Twins and pieces, which need counts from real text; without text both rules are skipped.

use std::collections::HashMap;

use super::super::fixture::AFF;
use super::super::text::{KeyCounts, TextCounts};
use super::{build_with, dict, spellings, word};

/// One fold key's counts: the key, how often it stands alone, and each spelling's count.
type Key<'a> = (&'a str, u64, &'a [(&'a str, u64)]);

/// Text of `total` tokens where each key is counted.
fn text(total: u64, keys: &[Key<'_>]) -> TextCounts {
    let keys = keys.iter().map(|(key, alone, seen)| {
        let spellings = seen.iter().map(|(s, n)| ((*s).to_string(), *n)).collect();
        (
            (*key).to_string(),
            KeyCounts {
                alone: *alone,
                spellings,
            },
        )
    });
    TextCounts {
        total,
        keys: keys.collect::<HashMap<_, _>>(),
    }
}

#[test]
fn without_text_the_dictionary_form_wins_and_no_piece_goes() {
    let dict = dict(AFF, &["i", "I", "s", "l", "l'"]);
    let list = build_with(&[("i", 150), ("s", 160), ("l", 170)], Some(&dict), None);
    assert_eq!(spellings(&list), ["i", "s", "l"]);
    assert!(!list.text && list.twins.count == 0 && list.pieces.count == 0);
}

#[test]
fn a_twin_seen_often_away_from_sentence_starts_wins() {
    let dict = dict(AFF, &["i", "I", "l", "l'"]);
    let counts = text(
        1_000,
        &[
            ("i", 900, &[("I", 100), ("i", 5)]),
            ("l'", 0, &[("l'", 50)]),
        ],
    );
    let list = build_with(&[("i", 150), ("l", 170)], Some(&dict), Some(&counts));
    assert_eq!(list.words, [word("I", 150, true), word("l'", 170, false)]);
    assert_eq!(list.twins.first, ["i → I", "l → l'"]);
}

#[test]
fn a_twin_seen_too_rarely_leaves_the_dictionary_form() {
    let dict = dict(AFF, &["may", "May"]);
    let counts = text(1_000, &[("may", 900, &[("May", 19), ("may", 3)])]);
    let list = build_with(&[("may", 300)], Some(&dict), Some(&counts));
    assert_eq!(spellings(&list), ["may"]);
    assert_eq!(list.twins.count, 0);
}

#[test]
fn a_twin_seen_exactly_the_minimum_wins() {
    let dict = dict(AFF, &["may", "May"]);
    let counts = text(1_000, &[("may", 900, &[("May", 20), ("may", 3)])]);
    let list = build_with(&[("may", 300)], Some(&dict), Some(&counts));
    assert_eq!(list.words, [word("May", 300, true)]);
    assert_eq!(list.twins.first, ["may → May"]);
}

#[test]
fn a_spelling_with_the_apostrophe_is_never_a_piece() {
    let dict = dict(AFF, &["l", "l'", "s", "aujourd'hui"]);
    let counts = text(
        1_000_000_000,
        &[("l'", 0, &[("l'", 500)]), ("s", 1_000, &[])],
    );
    let words = [("l", 140), ("s", 150), ("aujourd", 400), ("hui", 410)];
    let list = build_with(&words, Some(&dict), Some(&counts));
    assert_eq!(
        spellings(&list),
        ["l'", "aujourd'hui"],
        "both stand alone 0 times"
    );
    assert_eq!(list.pieces.first, ["s"]);
}

#[test]
fn a_word_standing_alone_far_less_than_wordfreq_says_is_a_piece() {
    let dict = dict(AFF, &["s", "the", "rare"]);
    let counts = text(1_000_000, &[("s", 1_000, &[]), ("the", 40_000, &[])]);
    let list = build_with(
        &[("the", 130), ("s", 150), ("rare", 600)],
        Some(&dict),
        Some(&counts),
    );
    assert_eq!(
        spellings(&list),
        ["the", "rare"],
        "rare is too rare to judge"
    );
    assert_eq!(list.pieces.first, ["s"]);
}

#[test]
fn counts_are_found_through_the_fold_key() {
    let counts = text(10, &[("ete", 4, &[("été", 3)])]);
    assert_eq!(
        (counts.alone("Été"), counts.seen("été"), counts.seen("ete")),
        (4, 3, 0)
    );
    assert_eq!((counts.alone("x"), counts.seen("x")), (0, 0));
}
