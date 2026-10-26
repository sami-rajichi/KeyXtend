//! One reading's model: stupid backoff (Brants et al., 2007) over the seed's chains and over the learned chains, each
//! step down to a shorter context multiplied by `backoff`, the two parts blended by the learned weight.

use std::collections::HashMap;

use super::sink::Sink;
use super::{Learned, ScoreConfig, float};
use crate::casing::Casing;
use crate::fold::fold;
use crate::pack::layout::{SCORE_CERTAIN, STEPS_PER_ZIPF, ZIPF_OFFSET};
use crate::pack::{Pack, WordId};

mod rank;

/// The seed's part of a model.
struct Seed<'m> {
    /// The pack.
    pack: &'m Pack<'m>,
    /// The context's followers per level, longest context first.
    levels: Vec<HashMap<WordId, u8>>,
}

/// The scores of one reading: its context, the seed and the learned counts.
pub(super) struct Model<'m> {
    /// The seed, when the language has a pack.
    seed: Option<Seed<'m>>,
    /// The learned counts.
    learned: &'m dyn Learned,
    /// The words before the prefix, oldest first.
    context: &'m [&'m str],
    /// The settings.
    cfg: &'m ScoreConfig,
    /// The learned part's weight.
    weight: f64,
    /// How the language cases its words.
    casing: &'static Casing,
}

impl<'m> Model<'m> {
    /// The model of `context` over `seed` and `learned`, folding with `casing`, the language in front's.
    pub(super) fn new(
        seed: Option<&'m Pack<'m>>,
        learned: &'m dyn Learned,
        context: &'m [&'m str],
        cfg: &'m ScoreConfig,
        casing: &'static Casing,
    ) -> Self {
        let seed = seed.map(|pack| Seed {
            pack,
            levels: levels(pack, context, casing),
        });
        let weight = cfg.weight(learned.uses());
        Self {
            seed,
            learned,
            context,
            cfg,
            weight,
            casing,
        }
    }

    /// Offers only the context's followers, seed and learned, that start with `folded`; false when none does, so the
    /// word list fills in for a typed prefix.
    pub(super) fn offer_followers(&self, folded: &str, sink: &mut Sink<'_>) -> bool {
        let fits = |w: &str| fold(w, self.casing).starts_with(folded) && self.offerable(w);
        let mut matched = false;
        self.each_follower(&fits, &mut |w| {
            matched = true;
            self.offer_learned(w, sink);
        });
        matched
    }

    /// Calls `each` with every follower of the context that `fits` takes, learned then seed, never an empty spelling.
    pub(super) fn each_follower(&self, fits: &dyn Fn(&str) -> bool, each: &mut dyn FnMut(&str)) {
        let mut take = |w: &str| {
            if !w.is_empty() && fits(w) {
                each(w);
            }
        };
        for start in 0..self.context.len() {
            let context = self.context.get(start..).unwrap_or_default();
            self.learned.following(context, &mut take);
        }
        let Some(seed) = &self.seed else {
            return;
        };
        for id in followers(seed) {
            take(seed.pack.spelling(id));
        }
    }

    /// Offers the learned `word`: a seed word blends both parts unless forgotten; another word needs `min_uses`.
    fn offer_learned(&self, word: &str, sink: &mut Sink<'_>) {
        let seed = (self.seed.as_ref()).and_then(|s| Some((s, exact(s.pack, word, self.casing)?)));
        let learned = self.weight * self.learned_score(word);
        let score = match seed {
            Some(_) if self.learned.blocked(word) => return,
            Some((s, id)) => (1.0 - self.weight) * self.seed_score(s, id) + learned,
            None if self.learned.count(word) < self.cfg.min_uses => return,
            None => learned,
        };
        sink.take(word, score);
    }

    /// True when `word` may be offered: a seed word the user did not forget, or another word used `min_uses` times.
    fn offerable(&self, word: &str) -> bool {
        let seeded =
            (self.seed.as_ref()).is_some_and(|s| exact(s.pack, word, self.casing).is_some());
        if seeded {
            !self.learned.blocked(word)
        } else {
            self.learned.count(word) >= self.cfg.min_uses
        }
    }

    /// The blended score of `word` after the context, as `offer` would weigh it.
    pub(super) fn score(&self, word: &str) -> f64 {
        let seed = (self.seed.as_ref())
            .and_then(|s| Some(self.seed_score(s, id_of(s.pack, word, self.casing)?)));
        (1.0 - self.weight) * seed.unwrap_or_default() + self.weight * self.learned_score(word)
    }

    /// The seed's backoff score of `id`: its chance after the longest context that it followed, else its frequency.
    fn seed_score(&self, seed: &Seed<'_>, id: WordId) -> f64 {
        let mut step = 1.0;
        for level in &seed.levels {
            if let Some(&score) = level.get(&id) {
                return step * chance(score);
            }
            step *= self.cfg.backoff;
        }
        self.steps(seed.levels.len()) * share(seed.pack.freq(id))
    }

    /// The learned backoff score of `word`: its share after the longest context it followed, else of all uses.
    fn learned_score(&self, word: &str) -> f64 {
        if self.learned.uses() == 0 {
            return 0.0;
        }
        let mut step = 1.0;
        for start in 0..self.context.len() {
            let context = self.context.get(start..).unwrap_or_default();
            let count = self.learned.chain(context, word);
            if let (true, Some((last, rest))) = (count > 0, context.split_last()) {
                let total = if rest.is_empty() {
                    self.learned.count(last)
                } else {
                    self.learned.chain(rest, last)
                };
                return step * f64::from(count) / f64::from(total.max(count));
            }
            step *= self.cfg.backoff;
        }
        self.unseen(self.learned.count(word))
    }

    /// The learned score of a word used `count` times that followed no part of the context: its share of all uses
    /// after every backoff step.
    fn unseen(&self, count: u32) -> f64 {
        let uses = self.learned.uses();
        if uses == 0 {
            return 0.0;
        }
        self.steps(self.context.len()) * f64::from(count) / float(uses)
    }

    /// The most a word that follows nothing scores with `count` learned uses and seed frequency `freq`: the very sum
    /// `offer_learned` makes, so it bounds every such word with no more of either.
    fn ceiling(&self, count: u32, freq: Option<u8>) -> f64 {
        let levels = self.seed.as_ref().map(|s| s.levels.len());
        let seed = levels
            .zip(freq)
            .map_or(0.0, |(n, f)| self.steps(n) * share(f));
        (1.0 - self.weight) * seed + self.weight * self.unseen(count)
    }

    /// `backoff` multiplied in `n` times, in the order the scores multiply it.
    fn steps(&self, n: usize) -> f64 {
        (0..n).fold(1.0, |step, _| step * self.cfg.backoff)
    }
}

/// The followers of every level of `seed`, once each, in id order.
fn followers(seed: &Seed<'_>) -> Vec<WordId> {
    let mut ids: Vec<WordId> = seed
        .levels
        .iter()
        .flat_map(HashMap::keys)
        .copied()
        .collect();
    ids.sort_unstable();
    ids.dedup();
    ids
}

/// The followers of each context suffix the pack's chains can hold, longest first; an unknown word leaves its levels
/// empty, so they back off.
fn levels(pack: &Pack<'_>, context: &[&str], casing: &Casing) -> Vec<HashMap<WordId, u8>> {
    let longest = usize::from(pack.max_chain().saturating_sub(1)).min(context.len());
    let ids: Vec<Option<WordId>> = context.iter().map(|w| id_of(pack, w, casing)).collect();
    (1..=longest)
        .rev()
        .map(|n| {
            let tail = ids.get(ids.len() - n..).unwrap_or_default();
            let tail: Option<Vec<WordId>> = tail.iter().copied().collect();
            tail.map_or_else(HashMap::new, |t| pack.followers(&t).collect())
        })
        .collect()
}

/// The id of `word` as the pack builder found it: its spelling, then in lower case, then its fold's most frequent
/// spelling.
fn id_of(pack: &Pack<'_>, word: &str, casing: &Casing) -> Option<WordId> {
    let run = pack.run(&fold(word, casing));
    let spelt = |w: &str| run.clone().find(|&id| pack.spelling(id) == w);
    spelt(word)
        .or_else(|| spelt(&casing.lower(word)))
        .or_else(|| run.clone().next())
}

/// The id of the seed word spelt exactly `word`.
fn exact(pack: &Pack<'_>, word: &str, casing: &Casing) -> Option<WordId> {
    pack.run(&fold(word, casing))
        .find(|&id| pack.spelling(id) == word)
}

/// A follower's chance after its context, from its score byte.
fn chance(score: u8) -> f64 {
    let steps = f64::from(score) - f64::from(SCORE_CERTAIN);
    10f64.powf(steps / f64::from(STEPS_PER_ZIPF))
}

/// A word's share of all words, from its frequency byte: 10 to the power of its Zipf value less `ZIPF_OFFSET`.
fn share(freq: u8) -> f64 {
    let zipf = f64::from(freq) / f64::from(STEPS_PER_ZIPF);
    10f64.powf(zipf - f64::from(ZIPF_OFFSET))
}
