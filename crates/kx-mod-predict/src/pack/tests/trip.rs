//! A small pack with French, English and Arabic words round-trips through the writer and the reader.

use std::collections::BTreeSet;

use super::super::*;
use crate::fold::fold;

/// The sample pack's language.
const ISO: &str = "fr";
/// Followers kept per context; the sample gives `the` three, so the lowest goes.
const FOLLOWERS: u8 = 2;
/// The sample's words: spelling, frequency bucket and whether it is always capitalised.
pub(super) const WORDS: &[(&str, u8, bool)] = &[
    ("école", 200, false),
    ("économie", 150, false),
    ("à", 250, false),
    ("a", 255, false),
    ("the", 255, false),
    ("London", 180, true),
    ("I", 240, true),
    ("أن", 240, false),
    ("إن", 230, false),
    ("أنا", 220, false),
    ("على", 250, false),
    ("إلى", 245, false),
    ("المدرسة", 150, false),
];
/// The sample's chains: the words before, the next word and its score.
const CHAINS: &[(&[&str], &str, u8)] = &[
    (&["the"], "London", 90),
    (&["the"], "école", 20),
    (&["the"], "a", 50),
    (&["إلى"], "المدرسة", 200),
    (&["أنا"], "على", 30),
    (&["أنا", "على"], "المدرسة", 77),
];

/// The sample pack's bytes.
pub(super) fn sample() -> Vec<u8> {
    let flags = Flags {
        checked: true,
        chains: true,
    };
    let mut writer = PackWriter::new(ISO, flags, FOLLOWERS).unwrap();
    for (spelling, freq, capital) in WORDS {
        writer.add_word(spelling, *freq, *capital).unwrap();
    }
    for (context, next, score) in CHAINS {
        writer.add_chain(context, next, *score).unwrap();
    }
    writer.finish().unwrap()
}

/// The id of `spelling` in `pack`, found through its folded key's run.
pub(super) fn id(pack: &Pack<'_>, spelling: &str) -> WordId {
    let run = pack.run(&fold(spelling));
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
fn every_word_round_trips_with_its_frequency_and_capital() {
    let bytes = sample();
    let pack = Pack::parse(&bytes).unwrap();
    for (spelling, freq, capital) in WORDS {
        let id = id(&pack, spelling);
        assert_eq!(pack.freq(id), *freq, "{spelling}");
        assert_eq!(pack.capital(id), *capital, "{spelling}");
    }
}

#[test]
fn one_folded_key_is_one_run_highest_frequency_first() {
    let bytes = sample();
    let pack = Pack::parse(&bytes).unwrap();
    assert_eq!(spellings(&pack, pack.run("a")), ["a", "à"]);
    assert_eq!(spellings(&pack, pack.run(&fold("إن"))), ["أن", "إن"]);
    assert!(pack.run("zz").is_empty());
    let keys: Vec<String> = (0..pack.word_count())
        .map(|id| fold(pack.spelling(id)))
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
