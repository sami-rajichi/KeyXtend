//! Twins, pieces and stems, which need counts from real text; without text the three rules are skipped.

use std::collections::HashMap;

use super::super::fixture::AFF;
use super::super::text::{KeyCounts, TextCounts};
use super::super::wordfreq::{bucket_of, bucket_of_share, freq, share};
use super::{build_with, dict, spellings, word};

/// One fold key's counts: the key, how often it stands alone, and each spelling's count.
type Key<'a> = (&'a str, u64, &'a [(&'a str, u64)]);

/// Text of `total` tokens where each key is counted; each spelling is found as often as it is seen.
fn text(total: u64, keys: &[Key<'_>]) -> TextCounts {
    let spellings = |seen: &[(&str, u64)]| -> HashMap<String, u64> {
        seen.iter().map(|(s, n)| ((*s).to_string(), *n)).collect()
    };
    let found = keys
        .iter()
        .flat_map(|(_, _, seen)| spellings(seen))
        .collect();
    let keys = keys.iter().map(|(key, alone, seen)| {
        let counts = KeyCounts {
            alone: *alone,
            spellings: spellings(seen),
        };
        ((*key).to_string(), counts)
    });
    TextCounts {
        total,
        keys: keys.collect(),
        found,
    }
}

#[test]
fn without_text_the_dictionary_form_wins_and_no_piece_goes() {
    let dict = dict(AFF, &["i", "I", "s", "l", "l'"]);
    let list = build_with(&[("i", 150), ("s", 160), ("l", 170)], Some(&dict), None);
    assert_eq!(spellings(&list), ["i", "s", "l"]);
    assert!(!list.text && list.text_facts.twins.count == 0 && list.text_facts.pieces.count == 0);
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
    let l = bucket_of(50, 1_000).unwrap();
    assert_eq!(list.words, [word("l'", l, false), word("I", 150, true)]);
    assert_eq!(list.text_facts.twins.first, ["i → I", "l → l'"]);
}

#[test]
fn a_twin_seen_too_rarely_leaves_the_dictionary_form() {
    let dict = dict(AFF, &["may", "May"]);
    let counts = text(1_000, &[("may", 900, &[("May", 19), ("may", 3)])]);
    let list = build_with(&[("may", 300)], Some(&dict), Some(&counts));
    assert_eq!(spellings(&list), ["may"]);
    assert_eq!(list.text_facts.twins.count, 0);
}

#[test]
fn a_twin_seen_exactly_the_minimum_wins() {
    let dict = dict(AFF, &["may", "May"]);
    let counts = text(1_000, &[("may", 900, &[("May", 20), ("may", 1)])]);
    let list = build_with(&[("may", 300)], Some(&dict), Some(&counts));
    assert_eq!(list.words, [word("May", 300, true)]);
    assert_eq!(list.text_facts.twins.first, ["may → May"]);
}

#[test]
fn a_case_twin_needs_its_ratio_but_an_elided_twin_only_needs_more() {
    let dict = dict(AFF, &["bill", "Bill", "t", "t'"]);
    let keys: [Key<'_>; 3] = [
        ("bill", 988, &[("Bill", 507), ("bill", 481)]),
        ("t'", 3_924, &[("t'", 3_924)]),
        ("t", 3_627, &[("t", 3_627)]),
    ];
    let list = build_with(
        &[("bill", 300), ("t", 200)],
        Some(&dict),
        Some(&text(1_000_000, &keys)),
    );
    assert_eq!(
        spellings(&list),
        ["t'", "bill"],
        "Bill is not seen 10 times more than bill"
    );
    assert_eq!(list.text_facts.twins.first, ["t → t'"]);
}

#[test]
fn a_spelling_with_the_apostrophe_is_never_a_piece() {
    let dict = dict(AFF, &["l", "l'", "aujourd'hui"]);
    let counts = text(
        1_000_000_000,
        &[
            ("l'", 0, &[("l'", 500)]),
            ("aujourd'hui", 0, &[("aujourd'hui", 300)]),
        ],
    );
    let words = [("l", 140), ("aujourd", 400), ("hui", 410)];
    let list = build_with(&words, Some(&dict), Some(&counts));
    let l = bucket_of(500, 1_000_000_000).unwrap();
    assert_eq!(
        list.words,
        [word("aujourd'hui", 410, false), word("l'", l, false)],
        "both stand alone 0 times; the elision keeps its text share, aujourd'hui takes all of hui's"
    );
    assert_eq!(list.text_facts.pieces.count, 0);
}

#[test]
fn a_word_its_apostrophe_words_dwarf_is_a_piece() {
    let dict = dict(
        AFF,
        &["s", "it's", "t", "don't", "dude", "dude's", "let", "let's"],
    );
    let keys: [Key<'_>; 7] = [
        ("s", 100, &[]),
        ("it's", 0, &[("it's", 2_001)]),
        ("don't", 0, &[("don't", 99)]),
        ("dude", 1, &[]),
        ("dude's", 0, &[("dude's", 3)]),
        ("let", 1_000, &[]),
        ("let's", 0, &[("let's", 1_500)]),
    ];
    let words = [
        ("s", 150),
        ("let", 160),
        ("it's", 170),
        ("don't", 180),
        ("t", 300),
        ("dude", 400),
    ];
    let list = build_with(&words, Some(&dict), Some(&text(1_000_000, &keys)));
    assert_eq!(
        spellings(&list),
        ["let", "it's", "don't", "let's", "t", "dude", "dude's"],
        "let's is seen more than let, but not 20 times more; don't is too rare to judge t"
    );
    assert_eq!(
        list.text_facts.pieces.first,
        ["s 100/3504"],
        "alone 100 times, joined 2001 + 3 + 1500"
    );
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

/// wordfreq tokens holding `it's` but neither `don't` nor `can't`, which the dictionary joins from their pieces.
const STEM_WORDS: [(&str, u32); 6] = [
    ("it", 205),
    ("it's", 230),
    ("can", 250),
    ("don", 440),
    ("t", 300),
    ("s", 310),
];

#[test]
fn an_apostrophe_word_takes_its_share_of_its_right_pieces_wordfreq_frequency() {
    let dict = dict(
        AFF,
        &["don't", "don", "t", "it", "can't", "can", "it's", "s"],
    );
    let keys: [Key<'_>; 6] = [
        ("don't", 0, &[("don't", 5_000)]),
        ("can't", 0, &[("can't", 10_000)]),
        ("it's", 0, &[("it's", 1)]),
        ("t", 1_000, &[("t", 5_000)]),
        ("can", 20_000, &[]),
        ("it", 9_000, &[]),
    ];
    let list = build_with(&STEM_WORDS, Some(&dict), Some(&text(1_000_000, &keys)));
    let part = |n: f64| bucket_of_share(share(300) * n).unwrap();
    let (dont, cant) = (part(0.25), part(0.5));
    assert_eq!(
        (dont, cant),
        (360, 330),
        "5000 and 10000 of t's 5000 sightings and its 15000 joined"
    );
    let find = |s: &str| list.words.iter().find(|w| w.spelling == s);
    assert_eq!(find("don't"), Some(&word("don't", dont, false)));
    assert_eq!(find("can't"), Some(&word("can't", cant, false)));
    assert_eq!(
        find("it's"),
        Some(&word("it's", 230, false)),
        "a wordfreq token keeps its own"
    );
    let shift = format!("can't {} → {}", freq(300), freq(cant));
    let stems = &list.text_facts.stems;
    assert!(
        stems.first.contains(&shift) && stems.count == 2,
        "{stems:?}: it's keeps its own"
    );
    assert_eq!(list.text_facts.pieces.first, ["don 0/5000"]);
    assert!(
        list.covers < 1.0,
        "the stems' shares are no wordfreq tokens'"
    );
}

#[test]
fn a_stem_text_never_shows_goes_and_one_only_text_adds_needs_text() {
    let dict = dict(AFF, &["can", "t", "can't", "won", "won't"]);
    let words = [("can", 250), ("t", 357), ("won", 400), ("won't", 380)];
    let without = build_with(&words, Some(&dict), None);
    assert_eq!(
        spellings(&without),
        ["can", "t", "won't", "won"],
        "no stem without text"
    );
    let keys: [Key<'_>; 3] = [
        ("can't", 0, &[("can't", 2_000)]),
        ("can", 3_000, &[]),
        ("won", 100, &[]),
    ];
    let with = build_with(&words, Some(&dict), Some(&text(1_000_000, &keys)));
    assert_eq!(spellings(&with), ["can", "can't", "won"], "t is a piece");
    assert_eq!(with.text_facts.unseen.first, ["won't"]);
}
