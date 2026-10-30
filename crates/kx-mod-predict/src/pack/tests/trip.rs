//! A small pack with French, English and Arabic words round-trips through the writer and the reader.

use std::collections::BTreeSet;

use kx_module_api::text::{Casing, fold};

use super::super::*;

/// The sample pack's language.
const ISO: &str = "fr";
/// Followers kept per context; the sample gives `the` three, so the lowest goes.
const FOLLOWERS: u8 = 2;
/// The sample's words: spelling, with its capitals, and frequency bucket.
const WORDS: &[Word] = &[
    ("école", 200),
    ("économie", 150),
    ("à", 250),
    ("a", 255),
    ("the", 255),
    ("London", 180),
    ("I", 240),
    ("أن", 240),
    ("إن", 230),
    ("أنا", 220),
    ("على", 250),
    ("إلى", 245),
    ("المدرسة", 150),
];
/// The sample's chains: the words before, the next word and its score.
const CHAINS: &[Chain] = &[
    (&["the"], "London", 90),
    (&["the"], "école", 20),
    (&["the"], "a", 50),
    (&["إلى"], "المدرسة", 200),
    (&["أنا"], "على", 30),
    (&["أنا", "على"], "المدرسة", 77),
];

/// A word as the sample lists it.
type Word = (&'static str, u8);
/// A chain as the sample lists it.
type Chain = (&'static [&'static str], &'static str, u8);

/// The sample pack's bytes.
pub(super) fn sample() -> Vec<u8> {
    written(WORDS.iter(), CHAINS.iter())
}

/// The bytes of a pack holding `words`, then `chains`, added in the order given.
fn written<'w>(
    words: impl Iterator<Item = &'w Word>,
    chains: impl Iterator<Item = &'w Chain>,
) -> Vec<u8> {
    let flags = Flags {
        checked: true,
        chains: true,
    };
    let mut writer = PackWriter::new(ISO, flags, FOLLOWERS).unwrap();
    for (spelling, freq) in words {
        writer.add_word(spelling, *freq).unwrap();
    }
    for (context, next, score) in chains {
        writer.add_chain(context, next, *score).unwrap();
    }
    writer.finish().unwrap()
}

#[test]
fn the_same_words_and_chains_in_another_order_give_the_same_bytes() {
    assert_eq!(written(WORDS.iter().rev(), CHAINS.iter().rev()), sample());
    let half = WORDS.len() / 2;
    let turned = WORDS[half..].iter().chain(&WORDS[..half]);
    assert_eq!(written(turned, CHAINS.iter().rev()), sample());
}

/// The id of `spelling` in `pack`, found through its folded key's run.
pub(super) fn id(pack: &Pack<'_>, spelling: &str) -> WordId {
    let run = pack.run(&fold(spelling, Casing::of(ISO)));
    run.into_iter()
        .find(|&id| pack.spelling(id) == spelling)
        .unwrap()
}

/// The spellings of `ids`.
fn spellings<'a>(pack: &Pack<'a>, ids: impl Iterator<Item = WordId>) -> Vec<&'a str> {
    ids.map(|id| pack.spelling(id)).collect()
}

#[test]
fn the_header_round_trips() {
    let bytes = sample();
    let pack = Pack::parse(&bytes).unwrap();
    assert_eq!(pack.iso(), ISO);
    assert_eq!(pack.word_count() as usize, WORDS.len());
    assert_eq!(pack.max_chain(), 3);
    assert_eq!(
        pack.flags(),
        Flags {
            checked: true,
            chains: true
        }
    );
}

#[test]
fn every_word_round_trips_with_its_capitals_and_frequency() {
    let bytes = sample();
    let pack = Pack::parse(&bytes).unwrap();
    for (spelling, freq) in WORDS {
        let id = id(&pack, spelling);
        assert_eq!(pack.spelling(id), *spelling);
        assert_eq!(pack.freq(id), *freq, "{spelling}");
    }
}

#[test]
fn one_folded_key_is_one_run_highest_frequency_first() {
    let bytes = sample();
    let pack = Pack::parse(&bytes).unwrap();
    assert_eq!(spellings(&pack, pack.run("a")), ["a", "à"]);
    assert_eq!(
        spellings(&pack, pack.run(&fold("إن", Casing::of(ISO)))),
        ["أن", "إن"]
    );
    assert!(pack.run("zz").is_empty());
    let keys: Vec<String> = (0..pack.word_count())
        .map(|id| fold(pack.spelling(id), Casing::of(ISO)))
        .collect();
    assert!(keys.is_sorted(), "the word table is sorted by folded key");
}

#[test]
fn prefix_completions_find_every_folded_match() {
    let bytes = sample();
    let pack = Pack::parse(&bytes).unwrap();
    let eco: BTreeSet<_> = spellings(&pack, pack.completions("eco"))
        .into_iter()
        .collect();
    assert_eq!(eco, BTreeSet::from(["école", "économie"]));
    let an = spellings(&pack, pack.completions("ان"));
    for want in ["أن", "إن", "أنا"] {
        assert!(an.contains(&want), "{want}");
    }
    assert_eq!(pack.completions("").count(), WORDS.len());
    assert_eq!(pack.completions("zz").count(), 0);
}

#[test]
fn followers_come_highest_score_first_and_only_the_kept_ones() {
    let bytes = sample();
    let pack = Pack::parse(&bytes).unwrap();
    let the = id(&pack, "the");
    let named = |found: Vec<(WordId, u8)>| -> Vec<(&str, u8)> {
        found
            .into_iter()
            .map(|(id, score)| (pack.spelling(id), score))
            .collect()
    };
    assert_eq!(
        named(pack.followers(&[the]).collect()),
        [("London", 90), ("a", 50)]
    );
    let ila = id(&pack, "إلى");
    assert_eq!(named(pack.followers(&[ila]).collect()), [("المدرسة", 200)]);
    let two = [id(&pack, "أنا"), id(&pack, "على")];
    assert_eq!(named(pack.followers(&two).collect()), [("المدرسة", 77)]);
}

#[test]
fn an_unknown_or_unfit_context_has_no_followers() {
    let bytes = sample();
    let pack = Pack::parse(&bytes).unwrap();
    let the = id(&pack, "the");
    assert_eq!(pack.followers(&[id(&pack, "London")]).count(), 0);
    assert_eq!(pack.followers(&[]).count(), 0);
    assert_eq!(
        pack.followers(&[the, the, the]).count(),
        0,
        "longer than the longest chain"
    );
    assert_eq!(pack.followers(&[WordId::MAX]).count(), 0);
}
