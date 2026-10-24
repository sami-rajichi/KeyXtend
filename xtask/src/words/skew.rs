//! Tatoeba's fixed names: a first pass counts each candidate's share of Tatoeba's tokens, and a word over
//! `skew_ratio` times its wordfreq share has the Tatoeba chains it ends scaled down to that ratio.

use std::collections::HashMap;

use super::cands::{Cand, Tally};
use super::count::Repeats;
use super::float;
use super::split::Splitter;
use super::vocab::Vocab;
use super::wordfreq::share;

/// Each candidate's sightings among Tatoeba's tokens, `cap` sentences remembered as `Counter` does.
pub(crate) struct Shares<'a> {
    /// The candidates' spellings.
    vocab: &'a Vocab,
    /// How sentences and words split.
    split: Splitter<'a>,
    /// Each candidate's sightings.
    counts: Vec<u64>,
    /// All tokens.
    total: u64,
    /// The sentences seen, so a repeat counts once here too.
    repeats: Repeats,
}

impl<'a> Shares<'a> {
    /// No text counted yet, for `cands` candidates of `vocab`.
    pub(crate) fn new(vocab: &'a Vocab, cands: usize, split: Splitter<'a>, cap: usize) -> Self {
        Self {
            vocab,
            split,
            counts: vec![0; cands],
            total: 0,
            repeats: Repeats::new(cap),
        }
    }

    /// Counts every token of the sentences of `text` seen for the first time.
    pub(crate) fn add(&mut self, text: &str) {
        let text = self.split.normalise(text);
        let (split, vocab) = (self.split, self.vocab);
        let knows = |t: &str| vocab.knows(t);
        for sentence in split.sentences(&text).filter(|s| self.repeats.first(s)) {
            split.words(sentence, &knows, &mut |token| {
                self.total += 1;
                let hit = vocab.find(token.text).map(|h| h.cand);
                let at = hit.and_then(|c| usize::try_from(c).ok());
                if let Some(n) = at.and_then(|i| self.counts.get_mut(i)) {
                    *n += 1;
                }
            });
        }
    }

    /// Each candidate's factor for the Tatoeba chains it ends, 1 unless its Tatoeba share passes `ratio` times its
    /// wordfreq share; with the words scaled and their ratios, most skewed first, `show` of them named. Only a
    /// wordfreq token has a share to compare: a joined stem's bucket is borrowed.
    pub(crate) fn factors(
        &self,
        cands: &[Cand],
        tokens: &HashMap<String, u32>,
        ratio: f64,
        show: usize,
    ) -> (Vec<f32>, Tally) {
        let total = float(self.total.max(1));
        let skew = |c: &Cand, n: u64| {
            let own = tokens.get(&c.word) == Some(&c.bucket);
            if own {
                float(n) / total / share(c.bucket)
            } else {
                0.0
            }
        };
        let skews: Vec<f64> = (cands.iter().zip(&self.counts))
            .map(|(c, &n)| skew(c, n))
            .collect();
        let mut over: Vec<(usize, f64)> = (skews.iter().copied().enumerate())
            .filter(|&(_, s)| s > ratio)
            .collect();
        over.sort_by(|a, b| b.1.total_cmp(&a.1));
        let mut tally = Tally::default();
        for &(i, skew) in &over {
            let word = cands.get(i).map(|c| c.word.as_str()).unwrap_or_default();
            tally.add(show, || format!("{word} {skew:.0}"));
        }
        let factor = |s: f64| if s > ratio { narrow(ratio / s) } else { 1.0 };
        (skews.into_iter().map(factor).collect(), tally)
    }
}

/// A factor from 0 to 1 as an f32.
#[allow(
    clippy::cast_possible_truncation,
    reason = "A factor from 0 to 1 loses only digits an f32 cannot hold."
)]
fn narrow(x: f64) -> f32 {
    x as f32
}
