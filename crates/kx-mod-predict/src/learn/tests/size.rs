//! The memory figure at the spec's caps: 10,000 words and 30,000 chains of each length, with the default two and the
//! most, four, previous words.

use std::time::Instant;

use super::super::clean::{CLEAN_AT_TENTHS, TENTHS, peak};
use super::*;

/// Words learned: enough for every table to pass its peak and clean.
const LEARNS: usize = 60_000;
/// Learns per day, so the least recently used differ.
const PER_DAY: usize = 1_000;
/// Knuth's MMIX multiplier, for a repeatable order.
pub(super) const MUL: u64 = 6_364_136_223_846_793_005;
/// Knuth's MMIX increment.
pub(super) const INC: u64 = 1_442_695_040_888_963_407;
/// The high bits an MMIX step keeps, as its low bits repeat quickly.
pub(super) const SHIFT: u32 = 33;
/// The spec's memory figure per language, which the byte form stays within.
const FIGURE: usize = 2 * 1024 * 1024;
/// The heap's limit: the figure plus the room tables grow into before their clean-up, 1.2 ×.
const HEAP_FIGURE: usize = FIGURE * CLEAN_AT_TENTHS / TENTHS;

/// A learner for `iso` with `cfg` after `LEARNS` words of `vocab` in a repeatable order, each after the words before
/// it, mapped through `seed`; gives their places in `vocab` in the order learned. A vocabulary as large as the word
/// table's peak fills every table without a clean-up taking chains with it.
pub(super) fn filled(
    iso: &str,
    cfg: LearnConfig,
    seed: Option<&Pack<'_>>,
    vocab: &[String],
) -> (Learner, Vec<usize>) {
    let mut l = Learner::new(iso, cfg).unwrap();
    let (mut state, mut order) = (1u64, Vec::with_capacity(LEARNS));
    for n in 0..LEARNS {
        state = state.wrapping_mul(MUL).wrapping_add(INC);
        let next = usize::try_from(state >> SHIFT).unwrap() % vocab.len();
        let from = order.len().saturating_sub(cfg.context_words);
        let context: Vec<&str> = order[from..]
            .iter()
            .map(|&at: &usize| vocab[at].as_str())
            .collect();
        let day = DAY + u32::try_from(n / PER_DAY).unwrap();
        l.learn(seed, &vocab[next], &context, day);
        order.push(next);
    }
    (l, order)
}

#[test]
#[ignore = "fills the full caps; run with --release for the size report"]
fn full_caps_stay_near_the_memory_figure() {
    for context_words in [CFG.context_words, usize::from(MAX_CHAIN) - 1] {
        let cfg = LearnConfig {
            context_words,
            ..CFG
        };
        let vocab: Vec<String> = (0..peak(cfg.max_words)).map(word).collect();
        let start = Instant::now();
        let (l, _) = filled("en", cfg, None, &vocab);
        let spent = start.elapsed();
        let chains: Vec<usize> = l.chains.iter().map(Chains::len).collect();
        let heap = l.words.heap_bytes() + l.chains.iter().map(Chains::heap_bytes).sum::<usize>();
        let bytes = l.to_bytes().len();
        let words = l.words.len();
        eprintln!(
            "context {context_words}: words {words} chains {chains:?} heap {heap} bytes {bytes} in {spent:?}"
        );
        assert!(words >= CFG.max_words && chains.iter().all(|&n| n >= CFG.max_chains));
        assert!(heap <= HEAP_FIGURE && bytes <= FIGURE, "{heap} {bytes}");
    }
}
