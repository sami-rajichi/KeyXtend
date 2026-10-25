//! The writer: what it refuses, how it merges repeats, and a pack of words without chains.

use kx_module_api::ports::{MAX_ISO_LEN, RESERVED_NAMES};

use super::super::layout::{MAX_CHAIN, MAX_SPELLING};
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
        writer.add_word(word, 1).unwrap();
    }
    writer
}

#[test]
fn a_bad_language_code_is_refused() {
    let long = "x".repeat(MAX_ISO_LEN + 1);
    let device = RESERVED_NAMES[0];
    for iso in ["", "e n", "é", "EN", "../x", device, long.as_str()] {
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
            writer.add_word(spelling, 1).unwrap_err(),
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
fn case_twins_stay_two_words_in_one_run_each_with_its_own_capitals() {
    let mut writer = writer(LIST, &[]);
    writer.add_word("polish", 7).unwrap();
    writer.add_word("Polish", 9).unwrap();
    let bytes = writer.finish().unwrap();
    let pack = Pack::parse(&bytes).unwrap();
    let run: Vec<_> = pack
        .run("polish")
        .map(|id| (pack.spelling(id), pack.freq(id)))
        .collect();
    assert_eq!(run, [("Polish", 9), ("polish", 7)]);
}

#[test]
fn a_turkish_pack_keys_dotless_i_apart_and_an_english_pack_does_not() {
    let keys = |iso: &str| {
        let mut writer = PackWriter::new(iso, LIST, 0).unwrap();
        writer.add_word("ılık", 7).unwrap();
        writer.add_word("Ilık", 6).unwrap();
        writer.add_word("ilik", 5).unwrap();
        let bytes = writer.finish().unwrap();
        let pack = Pack::parse(&bytes).unwrap();
        let run: Vec<String> = pack
            .run("ilik")
            .map(|id| pack.spelling(id).to_owned())
            .collect();
        run
    };
    assert_eq!(keys("tr"), ["ilik"]);
    assert_eq!(keys("en"), ["ılık", "Ilık", "ilik"]);
}

#[test]
fn repeats_keep_the_highest_frequency_and_score() {
    let mut writer = writer(FULL, &["a", "b"]);
    writer.add_word("a", 9).unwrap();
    writer.add_word("a", 5).unwrap();
    writer.add_chain(&["a"], "b", 40).unwrap();
    writer.add_chain(&["a"], "b", 10).unwrap();
    let bytes = writer.finish().unwrap();
    let pack = Pack::parse(&bytes).unwrap();
    let a = id(&pack, "a");
    assert_eq!((pack.word_count(), pack.freq(a)), (2, 9));
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
