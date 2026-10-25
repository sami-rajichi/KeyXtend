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

#[test]
fn an_entry_learned_before_the_pack_merges_into_the_pack_spelling() {
    let bytes = french();
    let fr = Pack::parse(&bytes).unwrap();
    let mut l = learner("fr");
    teach(&mut l, None, "ecole ecole", DAY);
    assert_eq!(l.view(Some(&fr)).count("ecole"), 0, "read through the pack");
    l.learn(Some(&fr), "ecole", &[], DAY + 1);
    assert_eq!(listed(&l, "ec"), ["école"]);
    assert_eq!(l.view(Some(&fr)).count("école"), 3);
    assert!(!guessed(&l, Some(&fr), &[], "e").contains(&"ecole".to_owned()));
}

#[test]
fn a_merge_moves_the_counts_and_re_keys_the_chains() {
    let bytes = french();
    let fr = Pack::parse(&bytes).unwrap();
    let mut l = learner("fr");
    teach(&mut l, None, "l' ecole l' ecole l' école", DAY);
    l.learn(Some(&fr), "école", &["l'"], DAY + 1);
    assert_eq!(listed(&l, "ec"), ["école"]);
    let view = l.view(Some(&fr));
    assert_eq!((view.count("école"), view.chain(&["l'"], "école")), (4, 4));
    assert_eq!(view.chain(&["école"], "l'"), 2);
    assert_eq!(
        view.chain(&["école", "l'"], "école"),
        2,
        "merged keys add up"
    );
    assert_eq!(lengths(&l), [2, 2]);
}

#[test]
fn forgetting_a_pack_spelling_takes_its_strays_and_undo_restores_them() {
    let bytes = french();
    let fr = Pack::parse(&bytes).unwrap();
    let mut l = learner("fr");
    teach(&mut l, None, "l' ecole l' école", DAY);
    let before = l.clone();
    assert!(l.forget(Some(&fr), "école", DAY));
    assert_eq!((listed(&l, "ec"), lengths(&l)), (vec![], vec![0, 0]));
    assert!(l.view(Some(&fr)).blocked("école"));
    assert!(l.undo_forget());
    assert_eq!(l, before);
}

#[test]
fn a_merged_chain_keeps_the_later_day() {
    let bytes = french();
    let fr = Pack::parse(&bytes).unwrap();
    let mut l = learner("fr");
    teach(&mut l, None, "l' école", DAY);
    teach(&mut l, None, "l' ecole", DAY + 5);
    l.learn(Some(&fr), "ecole", &[], DAY + 5);
    l.purge(DAY + CFG.forget_days + 1);
    assert_eq!(l.view(Some(&fr)).chain(&["l'"], "école"), 2);
}

#[test]
fn a_stray_takes_the_pack_spelling_with_its_capitals() {
    let bytes = english();
    let en = Pack::parse(&bytes).unwrap();
    let mut l = learner("en");
    teach(&mut l, None, "london london", DAY);
    l.learn(Some(&en), "london", &[], DAY);
    assert_eq!(listed(&l, "lo"), ["London"]);
    assert_eq!(l.view(Some(&en)).count("London"), 3);
}
