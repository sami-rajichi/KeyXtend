//! The memory figure at the spec's caps: 10,000 words and 30,000 chains of each length, with the default two and the
//! most, four, previous words.

use std::time::Instant;

use super::super::clean::{CLEAN_AT_TENTHS, TENTHS, peak};
use super::*;

/// Words learned: enough for every table to pass its peak and clean.
const LEARNS: usize = 60_000;
/// Learns per day, so the least recently used differ.
const PER_DAY: usize = 1_000;
/// Knuth's MMIX multiplier, for a repeatable word order.
const MUL: u64 = 6_364_136_223_846_793_005;
/// Knuth's MMIX increment.
const INC: u64 = 1_442_695_040_888_963_407;
/// The high bits an MMIX step keeps, as its low bits repeat quickly.
const SHIFT: u32 = 33;
/// The spec's memory figure per language, which the byte form stays within.
const FIGURE: usize = 2 * 1024 * 1024;

/// A learner with the default caps and `context_words`, after `LEARNS` words in a repeatable order from as many
/// words as the word table's peak, so it fills without a clean-up taking chains with it.
fn filled(context_words: usize) -> Learner {
    let cfg = LearnConfig {
        context_words,
        ..CFG
    };
    let mut l = Learner::new("en", cfg).unwrap();
    let vocab: Vec<String> = (0..peak(cfg.max_words)).map(word).collect();
    let (mut state, mut before) = (1u64, Vec::new());
    for n in 0..LEARNS {
        state = state.wrapping_mul(MUL).wrapping_add(INC);
        let next = &vocab[usize::try_from(state >> SHIFT).unwrap() % vocab.len()];
        let context: Vec<&str> = before.iter().map(String::as_str).collect();
        l.learn(
            None,
            next,
            &context,
            DAY + u32::try_from(n / PER_DAY).unwrap(),
        );
        before.push(next.clone());
        if before.len() > cfg.context_words {
            before.remove(0);
        }
    }
    l
}

#[test]
#[ignore = "fills the full caps; run with --release for the size report"]
fn full_caps_stay_near_the_memory_figure() {
    for context_words in [CFG.context_words, usize::from(MAX_CHAIN) - 1] {
        let start = Instant::now();
        let l = filled(context_words);
        let spent = start.elapsed();
        let chains: Vec<usize> = l.chains.iter().map(Chains::len).collect();
        let heap = l.words.heap_bytes() + l.chains.iter().map(Chains::heap_bytes).sum::<usize>();
        let bytes = l.to_bytes().len();
        let words = l.words.len();
        eprintln!(
            "context {context_words}: words {words} chains {chains:?} heap {heap} bytes {bytes} in {spent:?}"
        );
        assert!(words >= CFG.max_words && chains.iter().all(|&n| n >= CFG.max_chains));
        assert!(
            heap <= FIGURE * CLEAN_AT_TENTHS / TENTHS && bytes <= FIGURE,
            "{heap} {bytes}"
        );
    }
}
