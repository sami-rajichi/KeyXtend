//! The writer: what it refuses, how it merges repeats, and a pack of words without chains.

use super::super::layout::{MAX_CHAIN, MAX_ISO, MAX_SPELLING};
use super::super::*;
use super::trip::id;

/// A pack's language in these tests.
const ISO: &str = "en";
/// Flags for a pack with chains.
const FULL: Flags = Flags {
    checked: true,
    chains: true,
};
/// Flags for a word list.
const LIST: Flags = Flags {
    checked: false,
    chains: false,
};
/// Followers kept per context in these tests.
const FOLLOWERS: u8 = 3;

/// A writer for `flags` holding `words`, each with frequency 1.
fn writer(flags: Flags, words: &[&str]) -> PackWriter {
    let mut writer = PackWriter::new(ISO, flags, FOLLOWERS).unwrap();
    for word in words {
        writer.add_word(word, 1, false).unwrap();
    }
    writer
}

#[test]
fn a_bad_language_code_is_refused() {
    let long = "x".repeat(MAX_ISO + 1);
    for iso in ["", "e n", "é", long.as_str()] {
        assert_eq!(
            PackWriter::new(iso, LIST, FOLLOWERS).unwrap_err(),
            WriteError::BadIso,
            "{iso}"
        );
    }
}

#[test]
fn a_bad_spelling_is_refused() {
    let long = "a".repeat(MAX_SPELLING + 1);
    let mut writer = writer(LIST, &[]);
    for spelling in ["", "\u{064E}\u{0640}", long.as_str()] {
        assert_eq!(
            writer.add_word(spelling, 1, false).unwrap_err(),
            WriteError::BadSpelling
        );
    }
}

#[test]
fn a_chain_needs_known_words_and_a_fitting_length() {
    let mut writer = writer(FULL, &["a", "b"]);
    let unknown = writer.add_chain(&["a"], "c", 1).unwrap_err();
    assert_eq!(unknown, WriteError::UnknownWord("c".to_owned()));
    assert!(matches!(
        writer.add_chain(&["z"], "a", 1),
        Err(WriteError::UnknownWord(_))
    ));
    assert_eq!(
        writer.add_chain(&[], "a", 1).unwrap_err(),
        WriteError::BadChain
    );
    let long = vec!["a"; usize::from(MAX_CHAIN)];
    assert_eq!(
        writer.add_chain(&long, "b", 1).unwrap_err(),
        WriteError::BadChain
    );
}

#[test]
fn the_chains_flag_must_match_the_chains_added() {
    let without = writer(FULL, &["a"]);
    assert_eq!(without.finish().unwrap_err(), WriteError::ChainsFlag);
    let mut with = writer(LIST, &["a", "b"]);
    with.add_chain(&["a"], "b", 1).unwrap();
    assert_eq!(with.finish().unwrap_err(), WriteError::ChainsFlag);
}

#[test]
fn repeats_keep_the_highest_frequency_and_score() {
    let mut writer = writer(FULL, &["a", "b"]);
    writer.add_word("a", 9, true).unwrap();
    writer.add_word("a", 5, false).unwrap();
    writer.add_chain(&["a"], "b", 40).unwrap();
    writer.add_chain(&["a"], "b", 10).unwrap();
    let bytes = writer.finish().unwrap();
    let pack = Pack::parse(&bytes).unwrap();
    let a = id(&pack, "a");
    assert_eq!(
        (pack.word_count(), pack.freq(a), pack.capital(a)),
        (2, 9, true)
    );
    let found: Vec<_> = pack.followers(&[a]).collect();
    assert_eq!(found, [(id(&pack, "b"), 40)]);
}

#[test]
fn a_longer_chain_alone_still_gives_every_shorter_section() {
    let mut writer = writer(FULL, &["a", "b", "c"]);
    writer.add_chain(&["a", "b"], "c", 7).unwrap();
    let bytes = writer.finish().unwrap();
    let pack = Pack::parse(&bytes).unwrap();
    let ids = [id(&pack, "a"), id(&pack, "b")];
    assert_eq!(pack.max_chain(), 3);
    assert_eq!(
        pack.followers(&ids).collect::<Vec<_>>(),
        [(id(&pack, "c"), 7)]
    );
    assert_eq!(pack.followers(&ids[1..]).count(), 0);
}

#[test]
fn a_word_list_has_no_chains() {
    let bytes = writer(LIST, &["b", "a"]).finish().unwrap();
    let pack = Pack::parse(&bytes).unwrap();
    assert_eq!((pack.max_chain(), pack.flags()), (1, LIST));
    assert_eq!(pack.followers(&[id(&pack, "a")]).count(), 0);
    assert_eq!(pack.spelling(0), "a");
}

#[test]
fn an_empty_pack_round_trips() {
    let bytes = writer(LIST, &[]).finish().unwrap();
    let pack = Pack::parse(&bytes).unwrap();
    assert_eq!(pack.word_count(), 0);
    assert_eq!(pack.completions("").count(), 0);
    assert_eq!(pack.spelling(0), "");
}
