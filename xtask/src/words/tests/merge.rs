//! The spelling-swap rule: a rejected word gives its frequency to its most frequent fold twin; prefixes never do.

use kx_module_api::text::Casing;

use super::super::fixture::{AFF, AFF_AL, AFF_ANA};
use super::{build, build_with, dict, spellings, word};

#[test]
fn a_bare_alef_word_gives_its_frequency_to_its_most_frequent_fold_twin() {
    let words = [
        ("انا", 200),
        ("أنا", 300),
        ("ان", 150),
        ("أن", 250),
        ("إن", 260),
        ("فى", 140),
        ("في", 180),
    ];
    let list = build(&words, &["أنا", "أن", "إن", "في"]);
    let expected = [
        word("في", 140),
        word("أن", 150),
        word("أنا", 200),
        word("إن", 260),
    ];
    assert_eq!(list.words, expected, "ان goes to أن only, فى to في");
    let merged = ["انا → أنا", "ان → أن", "فى → في"];
    assert_eq!(list.facts.merged.count, merged.len());
    assert_eq!(list.facts.merged.first, merged[..2]);
    assert_eq!(
        list.coverage.map(|c| c.rejected),
        Some(0),
        "a merged word is no gap"
    );
}

#[test]
fn a_bare_word_the_dictionary_knows_stays_with_its_frequency() {
    let words = [("كان", 200), ("ابن", 300), ("كأن", 400), ("أبن", 500)];
    let list = build(&words, &["كان", "ابن", "كأن", "أبن"]);
    let expected = [
        word("كان", 200),
        word("ابن", 300),
        word("كأن", 400),
        word("أبن", 500),
    ];
    assert_eq!(list.words, expected);
    assert_eq!(list.facts.merged.count, 0);
}

#[test]
fn a_rejected_word_without_a_fold_twin_stays_a_gap() {
    let list = build(&[("اذا", 200), ("أنا", 300)], &["أنا", "إذن"]);
    assert_eq!(spellings(&list), ["أنا"]);
    assert_eq!(
        list.coverage.map(|c| c.first),
        Some(vec!["اذا".to_string()])
    );
}

#[test]
fn a_twin_far_rarer_than_the_rejected_word_takes_nothing() {
    let words = [("الذى", 300), ("ألذى", 401), ("اخرى", 300), ("أخرى", 400)];
    let list = build(&words, &["ألذى", "أخرى"]);
    assert_eq!(list.words, [word("أخرى", 300), word("ألذى", 401)]);
    assert_eq!(
        list.facts.merged.first,
        ["اخرى → أخرى"],
        "100 centibels rarer merges, 101 does not"
    );
}

#[test]
fn the_most_frequent_twin_wins_whatever_the_list_order() {
    let words = [("ete", 300), ("étê", 500), ("été", 350)];
    let list = build(&words, &["étê", "été"]);
    assert_eq!(list.words, [word("été", 300), word("étê", 500)]);
}

#[test]
fn a_rejected_prefix_never_merges() {
    let dict = dict(AFF_AL, &["آل", "بيت/A"]);
    assert!(dict.is_prefix("ال") && !dict.is_prefix("آل") && !dict.is_prefix("Y"));
    let list = build_with(&[("ال", 100), ("آل", 300)], Some(&dict), None);
    assert_eq!(list.words, [word("آل", 300)]);
    assert_eq!(list.facts.merged.count, 0);
    assert_eq!(list.facts.prefixes.first, ["ال"]);
    assert_eq!(
        list.coverage.map(|c| c.rejected),
        Some(0),
        "a prefix is no gap"
    );
}

#[test]
fn a_prefix_rule_still_checks_words() {
    let with_al = dict(AFF_AL, &["بيت/A"]);
    assert_eq!(with_al.forms("البيت", Casing::root()), ["البيت"]);
    assert!(with_al.forms("الكتاب", Casing::root()).is_empty());
    assert!(
        dict(AFF, &["بيت"])
            .forms("البيت", Casing::root())
            .is_empty()
    );
}

#[test]
fn a_rejected_word_equal_to_a_suffix_still_merges() {
    let dict = dict(AFF_ANA, &["أنا", "كتب/B"]);
    assert!(!dict.is_prefix("انا"));
    assert_eq!(dict.forms("كتبانا", Casing::root()), ["كتبانا"]);
    let list = build_with(&[("انا", 200), ("أنا", 300)], Some(&dict), None);
    assert_eq!(list.words, [word("أنا", 200)]);
    assert_eq!(list.facts.merged.first, ["انا → أنا"]);
}
