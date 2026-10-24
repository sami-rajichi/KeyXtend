//! The text counter: one pass over the text fills the counts the list's text rules read, and counts the chains of
//! `min_chain` to `max_chain` candidates within a sentence. The chain table stays under `max_live_chains` by lossy
//! counting: past it, the chains under a rising floor go.

use std::collections::HashMap;

use kx_mod_predict::pack::layout::MAX_CHAIN;

use super::manifest::TextRules;
use super::split::{Splitter, Token};
use super::text::TextCounts;
use super::vocab::Vocab;

/// The most words a chain key holds.
pub(crate) const KEY_LEN: usize = MAX_CHAIN as usize;
/// A chain key's unused places.
pub(crate) const NONE: u32 = u32::MAX;

/// A chain of candidate or word places, `NONE` after its last.
pub(crate) type ChainKey = [u32; KEY_LEN];

/// Chain counts kept under a cap by dropping the rarest.
#[derive(Debug)]
pub(crate) struct Chains {
    /// Each chain's weighted count.
    pub(crate) table: HashMap<ChainKey, u32>,
    /// The most chains kept at once.
    cap: usize,
    /// Chains counted fewer times than this were dropped at least once.
    pub(crate) floor: u32,
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
            floor: 0,
            prunes: 0,
        }
    }

    /// Counts `key` `weight` more times, pruning when the table passes its cap.
    pub(crate) fn add(&mut self, key: ChainKey, weight: u32) {
        let count = self.table.entry(key).or_default();
        *count = count.saturating_add(weight);
        while self.table.len() > self.cap {
            self.floor += 1;
            let floor = self.floor;
            self.table.retain(|_, n| *n >= floor);
            self.prunes += 1;
        }
    }
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
        }
    }

    /// Counts every sentence of `text`, each sighting `weight` times.
    pub(crate) fn add(&mut self, text: &str, weight: u32) {
        let text = self.split.normalise(text);
        let (split, vocab) = (self.split, self.vocab);
        let knows = |t: &str| vocab.knows(t);
        let mut run: Vec<u32> = Vec::with_capacity(KEY_LEN);
        for sentence in split.sentences(&text) {
            split.words(sentence, &knows, &mut |token| {
                self.token(token, weight, &mut run);
            });
        }
    }

    /// Counts one token and the chains it ends; `run` holds the candidates of the chain so far, and a sentence's first
    /// word is never joined, so it starts a new one.
    fn token(&mut self, token: Token<'_>, weight: u32, run: &mut Vec<u32>) {
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
        if !token.joined {
            run.clear();
        }
        if run.len() == self.lengths.1 {
            run.remove(0);
        }
        run.push(hit.cand);
        self.chains_ending(run, weight);
    }

    /// Counts each chain of a counted length that ends `run`'s last candidate.
    fn chains_ending(&mut self, run: &[u32], weight: u32) {
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
        }
    }
}
