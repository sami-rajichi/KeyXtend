//! Picks the chains a pack keeps: chains move from candidates to the list's words, each follower's score is its
//! quantised conditional log-probability, each context keeps its best `followers`, and the least seen contexts go
//! until the chain sections fit the budget.

use std::cmp::Reverse;
use std::collections::HashMap;

use kx_mod_predict::pack::layout::{
    CHAIN_HEAD, FOLLOWER_LEN, MIN_CHAIN, SCORE_CERTAIN, STEPS_PER_ZIPF, record_len,
};

use super::count::{ChainKey, KEY_LEN, NONE};
use super::float;

/// One context a pack keeps, by word place in the list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Context {
    /// The context's words, oldest first.
    pub(crate) words: Vec<u32>,
    /// Its followers with their scores, best first.
    pub(crate) followers: Vec<(u32, u8)>,
    /// How often the context was seen, weighted.
    strength: u64,
}

/// The chains a pack keeps.
#[derive(Debug, Default)]
pub(crate) struct Picked {
    /// The kept contexts.
    pub(crate) contexts: Vec<Context>,
    /// The kept followers, all contexts together.
    pub(crate) chains: usize,
    /// The bytes their sections take.
    pub(crate) bytes: usize,
}

/// The chains of `table` (by candidate place) on the list's words, given each candidate's sightings in `seen` and
/// its word in `of_cand`; each context keeps `followers` and all fit in `budget` bytes for chains up to `longest`.
pub(crate) fn pick(
    table: HashMap<ChainKey, u32>,
    seen: &[u64],
    of_cand: &[Option<u32>],
    followers: u8,
    budget: usize,
    longest: u8,
) -> Picked {
    let words = word_seen(seen, of_cand);
    let chains = on_words(table, of_cand);
    let mut contexts = score(&chains, &words, usize::from(followers));
    let overhead = usize::from(longest.saturating_sub(MIN_CHAIN) + 1) * CHAIN_HEAD;
    let mut bytes = overhead + contexts.iter().map(cost).sum::<usize>();
    contexts.sort_by_key(|c| (c.strength, Reverse(c.words.len())));
    let drop = contexts
        .iter()
        .take_while(|c| {
            let over = bytes > budget;
            if over {
                bytes -= cost(c);
            }
            over
        })
        .count();
    contexts.drain(..drop);
    let chains = contexts.iter().map(|c| c.followers.len()).sum();
    let bytes = section_bytes(&contexts);
    Picked {
        contexts,
        chains,
        bytes,
    }
}

/// How often each word was seen: the sum of its candidates' sightings.
fn word_seen(seen: &[u64], of_cand: &[Option<u32>]) -> Vec<u64> {
    let mut words: Vec<u64> = Vec::new();
    for (count, word) in seen.iter().zip(of_cand) {
        let Some(at) = word.and_then(|w| usize::try_from(w).ok()) else {
            continue;
        };
        if words.len() <= at {
            words.resize(at + 1, 0);
        }
        if let Some(n) = words.get_mut(at) {
            *n += count;
        }
    }
    words
}

/// The chains on the list's words, sorted by length then words, those holding a word the list dropped left out.
fn on_words(table: HashMap<ChainKey, u32>, of_cand: &[Option<u32>]) -> Vec<(ChainKey, u64)> {
    let word = |cand: u32| of_cand.get(usize::try_from(cand).ok()?).copied().flatten();
    let mut chains: Vec<(ChainKey, u64)> = Vec::with_capacity(table.len());
    for (key, count) in table {
        let mut out = [NONE; KEY_LEN];
        let mapped = key
            .iter()
            .zip(out.iter_mut())
            .take_while(|(c, _)| **c != NONE);
        let whole = mapped.fold(true, |ok, (c, slot)| {
            *slot = word(*c).unwrap_or(NONE);
            ok && *slot != NONE
        });
        if whole {
            chains.push((out, u64::from(count)));
        }
    }
    chains.sort_unstable_by_key(|(key, _)| (len(key), *key));
    chains.dedup_by(|b, a| {
        let same = a.0 == b.0;
        if same {
            a.1 += b.1;
        }
        same
    });
    chains
}

/// The contexts of the sorted `chains`, each with its best `keep` followers scored against how often the context
/// was seen: `words` for one word, the shorter chain for more.
fn score(chains: &[(ChainKey, u64)], words: &[u64], keep: usize) -> Vec<Context> {
    let mut contexts = Vec::new();
    for run in chains.chunk_by(|a, b| len(&a.0) == len(&b.0) && context(&a.0) == context(&b.0)) {
        let Some((first, _)) = run.first() else {
            continue;
        };
        let words_of = context(first).to_vec();
        let total: u64 = run.iter().map(|(_, n)| n).sum();
        let strength = seen_context(&words_of, chains, words).max(total);
        let mut next: Vec<(u32, u64)> = run
            .iter()
            .filter_map(|(k, n)| Some((last(k)?, *n)))
            .collect();
        next.sort_by_key(|&(w, n)| (Reverse(n), w));
        next.truncate(keep);
        let followers = next
            .iter()
            .map(|&(w, n)| (w, quantise(n, strength)))
            .collect();
        contexts.push(Context {
            words: words_of,
            followers,
            strength,
        });
    }
    contexts
}

/// How often `context` was seen: its word's sightings for one word, else its own chain's count.
fn seen_context(context: &[u32], chains: &[(ChainKey, u64)], words: &[u64]) -> u64 {
    if let [word] = context {
        return usize::try_from(*word)
            .ok()
            .and_then(|w| words.get(w))
            .copied()
            .unwrap_or_default();
    }
    let mut key = [NONE; KEY_LEN];
    for (slot, w) in key.iter_mut().zip(context) {
        *slot = *w;
    }
    let found = chains.binary_search_by_key(&(context.len(), key), |(k, _)| (len(k), *k));
    found
        .ok()
        .and_then(|i| chains.get(i))
        .map_or(0, |(_, n)| *n)
}

/// The score of a follower seen `count` times after a context seen `context` times: `SCORE_CERTAIN` plus
/// `STEPS_PER_ZIPF` times the log10 of their ratio, clamped to a byte.
fn quantise(count: u64, context: u64) -> u8 {
    let log = (float(count) / float(context.max(1))).log10();
    let steps = f64::from(STEPS_PER_ZIPF) * log + f64::from(SCORE_CERTAIN);
    byte(steps)
}

/// `x` rounded and clamped to a byte.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "The value is clamped to a byte's range first."
)]
fn byte(x: f64) -> u8 {
    x.round().clamp(0.0, f64::from(u8::MAX)) as u8
}

/// The bytes one context takes in its section: its record and its followers.
fn cost(context: &Context) -> usize {
    let n = u8::try_from(context.words.len() + 1).unwrap_or(u8::MAX);
    record_len(n) + context.followers.len() * FOLLOWER_LEN
}

/// The bytes the chain sections of `contexts` take, each section from the shortest chain to their longest.
fn section_bytes(contexts: &[Context]) -> usize {
    let longest = contexts.iter().map(|c| c.words.len() + 1).max();
    let sections = longest.map_or(0, |n| n + 1 - usize::from(MIN_CHAIN));
    sections * CHAIN_HEAD + contexts.iter().map(cost).sum::<usize>()
}

/// The words of `key` before its last.
fn context(key: &ChainKey) -> &[u32] {
    key.get(..len(key).saturating_sub(1)).unwrap_or_default()
}

/// How many words `key` holds.
fn len(key: &ChainKey) -> usize {
    key.iter().take_while(|&&w| w != NONE).count()
}

/// The last word of `key`.
fn last(key: &ChainKey) -> Option<u32> {
    key.get(len(key).checked_sub(1)?).copied()
}
