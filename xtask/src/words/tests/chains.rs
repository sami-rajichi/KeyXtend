//! Picking chains: followers come scored in order, chains move onto the list's words, and a tight budget keeps only
//! the most seen contexts, which the written pack's chain sections then fit.

use std::collections::HashMap;

use kx_mod_predict::Pack;
use kx_mod_predict::pack::layout::{
    AT_SECTIONS, CHAIN_HEAD, CHAINS_BASE, ENTRY_AT_LEN, ENTRY_LEN, FOLLOWER_LEN, HEAD_LEN,
    record_len, u8_at, usize_at,
};

use super::super::WordsError;
use super::super::cands::Facts;
use super::super::chains::{Context, Picked, pick, top};
use super::super::count::{ChainKey, KEY_LEN, NONE};
use super::super::list::{TextFacts, Word, WordList};
use super::super::out::write_pack;

/// The chain key of places `ids`.
fn key(ids: &[u32]) -> ChainKey {
    let mut key = [NONE; KEY_LEN];
    key.iter_mut().zip(ids).for_each(|(slot, id)| *slot = *id);
    key
}

/// A chain table from `(places, count)` pairs.
fn table(chains: &[(&[u32], u16)]) -> HashMap<ChainKey, f32> {
    chains
        .iter()
        .map(|(ids, n)| (key(ids), f32::from(*n)))
        .collect()
}

/// The followers of `context` in `picked`.
fn followers<'a>(picked: &'a Picked, context: &[u32]) -> Option<&'a [(u32, u8)]> {
    let found = picked.contexts.iter().find(|c| c.words == context);
    found.map(|c: &Context| c.followers.as_slice())
}

/// Four candidates on four words, seen 100, 50, 30 and 20 times, with chains after word 0, word 1 and both.
fn sample() -> (HashMap<ChainKey, f32>, Vec<u64>) {
    let chains = table(&[
        (&[0, 1], 50),
        (&[0, 2], 25),
        (&[0, 3], 5),
        (&[1, 2], 10),
        (&[0, 1, 2], 20),
    ]);
    (chains, vec![100, 50, 30, 20])
}

/// Every candidate on its own word.
const SAME: [Option<u32>; 4] = [Some(0), Some(1), Some(2), Some(3)];

#[test]
fn followers_come_best_first_scored_by_their_chance_after_the_context() {
    let (chains, seen) = sample();
    let picked = pick(chains, &seen, &SAME, 2, usize::MAX, 3);
    assert_eq!(
        followers(&picked, &[0]),
        Some(&[(1, 247), (2, 240)][..]),
        "0.5 and 0.25 of 100"
    );
    assert_eq!(followers(&picked, &[1]), Some(&[(2, 238)][..]), "10 of 50");
    assert_eq!(
        followers(&picked, &[0, 1]),
        Some(&[(2, 245)][..]),
        "20 of the 50 chains 0 1"
    );
    assert_eq!((picked.contexts.len(), picked.chains), (3, 4));
}

#[test]
fn the_top_contexts_are_the_most_seen_one_word_ones_with_their_first_followers() {
    let (chains, seen) = sample();
    let picked = pick(chains, &seen, &SAME, 2, usize::MAX, 3);
    assert_eq!(top(&picked, 5, 5), [(0, vec![1, 2]), (1, vec![2])]);
    assert_eq!(top(&picked, 1, 1), [(0, vec![1])]);
}

#[test]
fn chains_move_onto_words_and_lose_the_ones_holding_a_dropped_word() {
    let chains = table(&[(&[0, 1], 5), (&[0, 2], 7), (&[0, 3], 9)]);
    let of_cand = [Some(0), Some(1), Some(1), None];
    let picked = pick(chains, &[20, 5, 7, 9], &of_cand, 8, usize::MAX, 3);
    assert_eq!(
        followers(&picked, &[0]),
        Some(&[(1, 249)][..]),
        "12 of 20 once both share word 1"
    );
}

#[test]
fn a_tight_budget_keeps_only_the_most_seen_contexts() {
    let (chains, seen) = sample();
    let strongest = record_len(2) + 2 * FOLLOWER_LEN;
    let budget = 2 * CHAIN_HEAD + strongest;
    let picked = pick(chains, &seen, &SAME, 2, budget, 3);
    let kept: Vec<&[u32]> = picked.contexts.iter().map(|c| c.words.as_slice()).collect();
    assert_eq!(kept, [&[0][..]]);
    assert_eq!(picked.bytes, CHAIN_HEAD + strongest);
    assert!(picked.bytes <= budget);
}

#[test]
fn the_written_pack_reads_back_its_followers_and_its_chain_sections_fit_the_budget() {
    let (chains, seen) = sample();
    let budget = 2 * CHAIN_HEAD + 2 * record_len(2) + 3 * FOLLOWER_LEN;
    let picked = pick(chains, &seen, &SAME, 2, budget, 3);
    let words = ["the", "cat", "sat", "down"].map(|w| Word {
        spelling: w.into(),
        freq: 100,
        capital: false,
    });
    let list = WordList {
        words: words.to_vec(),
        read: 4,
        coverage: None,
        facts: Facts::default(),
        text_facts: TextFacts::default(),
        of_cand: SAME.to_vec(),
        text: true,
        covers: 1.0,
        cut: (0, usize::MAX),
    };
    let bytes = write_pack("en", &list, &picked, 2).unwrap();
    let pack = Pack::parse(&bytes).unwrap();
    let the = pack.run("the").start;
    let next: Vec<&str> = pack
        .followers(&[the])
        .map(|(id, _)| pack.spelling(id))
        .collect();
    assert_eq!(next, ["cat", "sat"]);
    let scores: Vec<u8> = pack.followers(&[the]).map(|(_, s)| s).collect();
    assert!(scores.is_sorted_by(|a, b| a >= b), "{scores:?}");
    assert!(
        chain_bytes(&bytes) <= budget,
        "{} > {budget}",
        chain_bytes(&bytes)
    );
    assert_eq!(chain_bytes(&bytes), picked.bytes);
}

#[test]
fn a_chain_on_a_place_past_the_list_is_refused_not_shortened() {
    let (chains, seen) = sample();
    let of_cand = [Some(0), Some(1), Some(2), Some(7)];
    let picked = pick(chains, &seen, &of_cand, 3, usize::MAX, 3);
    let list = WordList {
        words: vec![
            Word {
                spelling: "the".into(),
                freq: 100,
                capital: false,
            };
            4
        ],
        read: 4,
        coverage: None,
        facts: Facts::default(),
        text_facts: TextFacts::default(),
        of_cand: of_cand.to_vec(),
        text: true,
        covers: 1.0,
        cut: (0, usize::MAX),
    };
    let refused = write_pack("en", &list, &picked, 3);
    assert!(matches!(refused, Err(WordsError::Pack(why)) if why.contains("place 7")));
}

/// The bytes of a pack's chain sections, from its section table.
fn chain_bytes(pack: &[u8]) -> usize {
    let sections = usize::from(u8_at(pack, AT_SECTIONS).unwrap());
    let entry = |i: usize| HEAD_LEN + i * ENTRY_LEN;
    let chains = (0..sections).filter(|&i| u8_at(pack, entry(i)).unwrap() > CHAINS_BASE);
    chains
        .map(|i| usize_at(pack, entry(i) + ENTRY_AT_LEN).unwrap())
        .sum()
}
