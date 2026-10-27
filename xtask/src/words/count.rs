//! The text counter: one pass over the text fills the counts the list's text rules read, and counts the chains of
//! `min_chain` to `max_chain` candidates within a sentence. A sentence whose letters were seen before counts once,
//! and the chain table stays under `max_live_chains` by lossy counting: past it, the chains under a rising floor go.

use std::collections::{HashMap, HashSet};
use std::hash::{DefaultHasher, Hasher};

use kx_mod_predict::pack::layout::MAX_CHAIN;
use kx_mod_predict::split::{Splitter, Token};

use super::manifest::TextRules;
use super::text::TextCounts;
use super::vocab::Vocab;

/// The most words a chain key holds.
pub(crate) const KEY_LEN: usize = MAX_CHAIN as usize;
/// A chain key's unused places.
pub(crate) const NONE: u32 = u32::MAX;
/// The weight of a chain nothing scales.
const UNSCALED: f32 = 1.0;
/// How far the floor rises at each prune: one count.
const FLOOR_STEP: f32 = 1.0;

/// A chain of candidate or word places, `NONE` after its last.
pub(crate) type ChainKey = [u32; KEY_LEN];

/// Chain counts kept under a cap by dropping the rarest.
#[derive(Debug)]
pub(crate) struct Chains {
    /// Each chain's weighted count.
    pub(crate) table: HashMap<ChainKey, f32>,
    /// The most chains kept at once.
    cap: usize,
    /// Chains counted fewer times than this were dropped at least once.
    pub(crate) floor: f32,
    /// How often the table was pruned.
    pub(crate) prunes: u32,
}

impl Chains {
    /// An empty table that keeps at most `cap` chains.
    pub(crate) fn new(cap: usize) -> Self {
        let table = HashMap::with_capacity(cap.saturating_add(1));
        Self {
            table,
            cap,
            floor: 0.0,
            prunes: 0,
        }
    }

    /// Counts `key` `weight` more times, pruning when the table passes its cap.
    pub(crate) fn add(&mut self, key: ChainKey, weight: f32) {
        *self.table.entry(key).or_default() += weight;
        while self.table.len() > self.cap {
            self.floor += FLOOR_STEP;
            let floor = self.floor;
            self.table.retain(|_, n| *n >= floor);
            self.prunes += 1;
        }
    }
}

/// The sentences seen so far, by key, so a repeat counts once.
#[derive(Debug)]
pub(crate) struct Repeats {
    /// The keys remembered.
    keys: HashSet<u64>,
    /// The most keys remembered; past it new sentences are no longer remembered.
    cap: usize,
    /// The sentences skipped as repeats.
    pub(crate) skipped: u64,
    /// The sentences read that hold letters.
    pub(crate) read: u64,
}

impl Repeats {
    /// No sentence seen yet, remembering at most `cap`.
    pub(crate) fn new(cap: usize) -> Self {
        Self {
            keys: HashSet::new(),
            cap,
            skipped: 0,
            read: 0,
        }
    }

    /// True when `sentence` is seen for the first time; a sentence without letters always is.
    pub(crate) fn first(&mut self, sentence: &str) -> bool {
        let Some(key) = key(sentence) else {
            return true;
        };
        self.read += 1;
        if self.keys.contains(&key) {
            self.skipped += 1;
            return false;
        }
        if self.keys.len() < self.cap {
            self.keys.insert(key);
        }
        true
    }

    /// How many keys are remembered.
    pub(crate) fn kept(&self) -> usize {
        self.keys.len()
    }
}

/// The key of `sentence`: its letters in order, case kept, hashed to 64 bits; `None` when it holds none.
pub(crate) fn key(sentence: &str) -> Option<u64> {
    let mut hasher = DefaultHasher::new();
    let mut letters = sentence.chars().filter(|c| c.is_alphabetic()).peekable();
    letters.peek()?;
    letters.for_each(|c| hasher.write_u32(u32::from(c)));
    Some(hasher.finish())
}

/// What the text held, as the list and the chain picker need it.
#[derive(Debug)]
pub(crate) struct Counted {
    /// The counts the list's text rules read.
    pub(crate) text: TextCounts,
    /// The chains, by candidate place.
    pub(crate) chains: Chains,
    /// How often each candidate was seen.
    pub(crate) seen: Vec<u64>,
    /// The repeated sentences skipped.
    pub(crate) repeats: Repeats,
}

/// Counts words and chains, sentence by sentence.
pub(crate) struct Counter<'a> {
    /// The candidates' spellings.
    vocab: &'a Vocab,
    /// How sentences and words split.
    split: Splitter<'a>,
    /// The chain lengths counted.
    lengths: (usize, usize),
    /// All tokens, weighted.
    total: u64,
    /// Each key's standalone sightings.
    key_alone: Vec<u64>,
    /// Each form's standalone sightings away from sentence starts.
    form_seen: Vec<u64>,
    /// Each form's sightings anywhere, by spelling, case or fold.
    form_found: Vec<u64>,
    /// Each candidate's sightings.
    seen: Vec<u64>,
    /// The chains.
    chains: Chains,
    /// The sentences seen.
    repeats: Repeats,
    /// Each candidate's factor for the scaled chains it ends.
    skew: Vec<f32>,
}

impl<'a> Counter<'a> {
    /// A counter over `vocab`'s `cands` candidates, as `rules` say, with `apostrophe` for splitting.
    pub(crate) fn new(
        vocab: &'a Vocab,
        cands: usize,
        rules: &'a TextRules,
        apostrophe: char,
    ) -> Self {
        let (forms, keys) = vocab.sizes();
        Self {
            vocab,
            split: Splitter {
                ends: &rules.sentence_ends,
                apostrophe,
            },
            lengths: (usize::from(rules.min_chain), usize::from(rules.max_chain)),
            total: 0,
            key_alone: vec![0; keys],
            form_seen: vec![0; forms],
            form_found: vec![0; forms],
            seen: vec![0; cands],
            chains: Chains::new(rules.max_live_chains),
            repeats: Repeats::new(rules.max_seen_sentences),
            skew: Vec::new(),
        }
    }

    /// Sets each candidate's factor for the chains it ends in scaled text; a candidate not named keeps 1.
    pub(crate) fn scale(&mut self, skew: Vec<f32>) {
        self.skew = skew;
    }

    /// Counts every sentence of `text` seen for the first time, each sighting `weight` times, its chains scaled by
    /// the candidates' factors when `scaled`; true when any word counted.
    pub(crate) fn add(&mut self, text: &str, weight: u32, scaled: bool) -> bool {
        let text = self.split.normalise(text);
        let (split, vocab) = (self.split, self.vocab);
        let knows = |t: &str| vocab.knows(t);
        let mut run: Vec<u32> = Vec::with_capacity(KEY_LEN);
        let mut counted = false;
        for sentence in split.sentences(&text) {
            if self.repeats.first(sentence) {
                split.words(sentence, &knows, &mut |token| {
                    counted = true;
                    self.token(token, weight, scaled, &mut run);
                });
            }
        }
        counted
    }

    /// Counts one token and the chains it ends; `run` holds the candidates of the chain so far. A sentence start, an
    /// unknown word or a word not standing alone (`est-ce`, `site.com`) ends it, as the tracker ends its context.
    fn token(&mut self, token: Token<'_>, weight: u32, scaled: bool, run: &mut Vec<u32>) {
        let w = u64::from(weight);
        self.total += w;
        let Some(hit) = self.vocab.find(token.text) else {
            run.clear();
            return;
        };
        let bump = |counts: &mut Vec<u64>, at: u32| {
            if let Some(n) = usize::try_from(at).ok().and_then(|i| counts.get_mut(i)) {
                *n += w;
            }
        };
        bump(&mut self.form_found, hit.found);
        bump(&mut self.seen, hit.cand);
        if token.alone {
            bump(&mut self.key_alone, hit.key);
        }
        if let (Some(form), false, true) = (hit.form, token.first, token.alone) {
            bump(&mut self.form_seen, form);
        }
        if !token.alone || !token.joined {
            run.clear();
        }
        if !token.alone {
            return;
        }
        if run.len() == self.lengths.1 {
            run.remove(0);
        }
        run.push(hit.cand);
        let factor = scaled.then(|| self.factor(hit.cand));
        self.chains_ending(run, weight_of(weight) * factor.unwrap_or(UNSCALED));
    }

    /// The factor of candidate `cand` for the scaled chains it ends.
    fn factor(&self, cand: u32) -> f32 {
        let at = usize::try_from(cand).ok();
        at.and_then(|i| self.skew.get(i))
            .copied()
            .unwrap_or(UNSCALED)
    }

    /// Counts each chain of a counted length that ends `run`'s last candidate.
    fn chains_ending(&mut self, run: &[u32], weight: f32) {
        let (shortest, longest) = self.lengths;
        for n in shortest..=longest.min(run.len()) {
            let mut key = [NONE; KEY_LEN];
            let tail = run.get(run.len() - n..).unwrap_or_default();
            for (slot, cand) in key.iter_mut().zip(tail) {
                *slot = *cand;
            }
            self.chains.add(key, weight);
        }
    }

    /// The counts, once every text is in.
    pub(crate) fn finish(self) -> Counted {
        let text = self.vocab.counts(
            self.total,
            &self.key_alone,
            &self.form_seen,
            &self.form_found,
        );
        Counted {
            text,
            chains: self.chains,
            seen: self.seen,
            repeats: self.repeats,
        }
    }
}

/// A source's weight as a chain count.
#[allow(
    clippy::cast_precision_loss,
    reason = "Source weights are small whole numbers, exact in an f32."
)]
fn weight_of(weight: u32) -> f32 {
    weight as f32
}
