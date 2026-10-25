//! The learner with a pack: seed words count at once, and informal spellings count toward the pack's.

use super::*;
use crate::pack::{Flags, PackWriter};

/// A pack of `words`, each a spelling with its capitals and its frequency byte, and no chains.
pub(super) fn pack(iso: &str, words: &[(&str, u8)]) -> Vec<u8> {
    let flags = Flags::default();
    let mut writer = PackWriter::new(iso, flags, 0).unwrap();
    for (spelling, freq) in words {
        writer.add_word(spelling, *freq).unwrap();
    }
    writer.finish().unwrap()
}

/// English: `US` and `us` share a key, `us` more frequent; `London` keeps its capital.
pub(super) fn english() -> Vec<u8> {
    pack(
        "en",
        &[
            ("the", 180),
            ("in", 170),
            ("us", 160),
            ("US", 150),
            ("London", 130),
            ("theory", 110),
        ],
    )
}

/// French: an elision and accented words.
pub(super) fn french() -> Vec<u8> {
    pack("fr", &[("l'", 181), ("école", 150), ("écoute", 140)])
}

/// Arabic: the hamza forms of `أن` and `إن`, and `أنا`.
pub(super) fn arabic() -> Vec<u8> {
    pack("ar", &[("أن", 175), ("إن", 156), ("أنا", 147)])
}

#[test]
fn a_seed_words_first_use_counts_at_once() {
    let bytes = english();
    let en = Pack::parse(&bytes).unwrap();
    let mut l = learner("en");
    teach(&mut l, Some(&en), "in theory", DAY);
    let view = l.view(Some(&en));
    assert_eq!(view.count("theory"), 1);
    assert_eq!(view.chain(&["in"], "theory"), 1);
    assert_eq!(followers(&l, Some(&en), &["in"]), ["theory"]);
}

#[test]
fn informal_spellings_count_toward_the_pack_spelling() {
    let bytes = arabic();
    let ar = Pack::parse(&bytes).unwrap();
    let mut l = learner("ar");
    teach(&mut l, Some(&ar), "انا", DAY);
    assert_eq!(listed(&l, "انا"), ["أنا"]);
    assert_eq!(l.view(Some(&ar)).count("أنا"), 1);
    assert_eq!(l.view(Some(&ar)).count("انا"), 1);
    let bytes = french();
    let fr = Pack::parse(&bytes).unwrap();
    let mut l = learner("fr");
    teach(&mut l, Some(&fr), "ecole Ecole", DAY);
    assert_eq!(listed(&l, "ecole"), ["école"]);
    assert_eq!(l.view(Some(&fr)).count("école"), 2);
}

#[test]
fn a_sentence_start_counts_toward_the_lower_spelling_and_an_exact_one_as_itself() {
    let bytes = english();
    let en = Pack::parse(&bytes).unwrap();
    let mut l = learner("en");
    teach(&mut l, Some(&en), "The london US Us", DAY);
    assert_eq!(listed(&l, "the"), ["the"]);
    assert_eq!(listed(&l, "lo"), ["London"]);
    let mut us = listed(&l, "us");
    us.sort();
    assert_eq!(us, ["US", "us"]);
    let view = l.view(Some(&en));
    assert_eq!((view.count("US"), view.count("us")), (1, 1));
    assert_eq!(view.chain(&["The"], "London"), 1);
}

#[test]
fn an_elision_reaches_its_learned_follower() {
    let bytes = french();
    let fr = Pack::parse(&bytes).unwrap();
    let mut l = learner("fr");
    l.learn(Some(&fr), "l'", &[], DAY);
    l.learn(Some(&fr), "écoute", &["l'"], DAY);
    assert_eq!(followers(&l, Some(&fr), &["l'"]), ["écoute"]);
}
