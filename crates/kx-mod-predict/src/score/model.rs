//! One reading's model: stupid backoff (Brants et al., 2007) over the seed's chains and over the learned chains, each
//! step down to a shorter context multiplied by `backoff`, the two parts blended by the learned weight.

use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};

use super::best::Best;
use super::case::Case;
use super::{Learned, ScoreConfig, float};
use crate::fold::fold;
use crate::pack::layout::{SCORE_CERTAIN, STEPS_PER_ZIPF, ZIPF_OFFSET};
use crate::pack::{Pack, WordId};
use crate::track::Query;

/// Where one reading's candidates go: prefixed by its lead, cased as its prefix asks and weighed by its lead.
pub(super) struct Sink<'s> {
    /// The best texts so far, across readings.
    pub(super) best: &'s mut Best,
    /// The query, so a guess that changes nothing is dropped.
    pub(super) query: &'s Query,
    /// The typed elisions each text starts with.
    pub(super) lead: &'s str,
    /// The case each candidate takes.
    pub(super) case: Case,
    /// The lead's chance, which every score of the reading is multiplied by.
    pub(super) scale: f64,
}

impl Sink<'_> {
    /// True when a candidate scoring `score` could enter.
    fn wants(&self, score: f64) -> bool {
        self.best.wants(score * self.scale)
    }

    /// Keeps `word` at `score` when it can enter and picking it changes the text.
    fn take(&mut self, word: &str, score: f64) {
        let score = score * self.scale;
        if self.best.wants(score) {
            let text = format!("{}{}", self.lead, self.case.apply(word));
            if self.query.changes(&text) {
                self.best.offer(text, score);
            }
        }
    }
}

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
}

impl<'m> Model<'m> {
    /// The model of `context` over `seed` and `learned`.
    pub(super) fn new(
        seed: Option<&'m Pack<'m>>,
        learned: &'m dyn Learned,
        context: &'m [&'m str],
        cfg: &'m ScoreConfig,
    ) -> Self {
        let seed = seed.map(|pack| Seed {
            pack,
            levels: levels(pack, context),
        });
        let weight = cfg.weight(learned.uses());
        Self {
            seed,
            learned,
            context,
            cfg,
            weight,
        }
    }

    /// Offers `sink` every candidate for `folded`: its completions, seed and learned, or the next-word candidates
    /// for an empty prefix.
    pub(super) fn offer(&self, folded: &str, fill: bool, sink: &mut Sink<'_>) {
        if folded.is_empty() {
            self.offer_next(fill, sink);
            return;
        }
        self.learned
            .starting(folded, &mut |w| self.offer_learned(w, sink));
        if let Some(seed) = &self.seed {
            for id in seed.pack.completions(folded) {
                self.offer_seed(seed, id, sink);
            }
        }
    }

    /// Offers the next-word candidates: the context's followers, seed and learned, and with `fill` the most frequent
    /// seed words and every learned word, as backoff reaches them all.
    fn offer_next(&self, fill: bool, sink: &mut Sink<'_>) {
        for start in 0..self.context.len() {
            let context = self.context.get(start..).unwrap_or_default();
            self.learned
                .following(context, &mut |w| self.offer_learned(w, sink));
        }
        if fill {
            self.learned
                .starting("", &mut |w| self.offer_learned(w, sink));
        }
        let Some(seed) = &self.seed else {
            return;
        };
        let top = if fill {
            self.top(seed, sink.best.limit())
        } else {
            Vec::new()
        };
        let followers = seed.levels.iter().flat_map(HashMap::keys).copied();
        let mut ids: Vec<WordId> = followers.chain(top).collect();
        ids.sort_unstable();
        ids.dedup();
        for id in ids {
            self.offer_seed(seed, id, sink);
        }
    }

    /// Offers the seed word `id` at its seed score, unless the user forgot it.
    fn offer_seed(&self, seed: &Seed<'_>, id: WordId, sink: &mut Sink<'_>) {
        let score = (1.0 - self.weight) * self.seed_score(seed, id);
        if !sink.wants(score) {
            return;
        }
        let word = seed.pack.spelling(id);
        if !word.is_empty() && !self.learned.blocked(word) {
            sink.take(word, score);
        }
    }

    /// Offers the learned `word`: a seed word blends both parts unless forgotten; another word needs `min_uses`.
    fn offer_learned(&self, word: &str, sink: &mut Sink<'_>) {
        let seed = self
            .seed
            .as_ref()
            .and_then(|s| Some((s, exact(s.pack, word)?)));
        let learned = self.weight * self.learned_score(word);
        let score = match seed {
            Some(_) if self.learned.blocked(word) => return,
            Some((s, id)) => (1.0 - self.weight) * self.seed_score(s, id) + learned,
            None if self.learned.count(word) < self.cfg.min_uses => return,
            None => learned,
        };
        sink.take(word, score);
    }

    /// The blended score of `word` after the context, as `offer` would weigh it.
    pub(super) fn score(&self, word: &str) -> f64 {
        let seed = self
            .seed
            .as_ref()
            .and_then(|s| Some(self.seed_score(s, id_of(s.pack, word)?)));
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
        step * share(seed.pack.freq(id))
    }

    /// The learned backoff score of `word`: its share after the longest context it followed, else of all uses.
    fn learned_score(&self, word: &str) -> f64 {
        let uses = self.learned.uses();
        if uses == 0 {
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
        step * f64::from(self.learned.count(word)) / float(uses)
    }

    /// The `limit` most frequent seed words the user did not forget, lower ids first among equals.
    fn top(&self, seed: &Seed<'_>, limit: usize) -> Vec<WordId> {
        let mut heap = BinaryHeap::with_capacity(limit + 1);
        for id in 0..seed.pack.word_count() {
            let freq = seed.pack.freq(id);
            let full = heap.len() == limit;
            if full && heap.peek().is_none_or(|Reverse((min, _))| freq <= *min) {
                continue;
            }
            if self.learned.blocked(seed.pack.spelling(id)) {
                continue;
            }
            heap.push(Reverse((freq, Reverse(id))));
            if heap.len() > limit {
                heap.pop();
            }
        }
        heap.into_iter()
            .map(|Reverse((_, Reverse(id)))| id)
            .collect()
    }
}

/// The followers of each context suffix the pack's chains can hold, longest first; an unknown word leaves its levels
/// empty, so they back off.
fn levels(pack: &Pack<'_>, context: &[&str]) -> Vec<HashMap<WordId, u8>> {
    let longest = usize::from(pack.max_chain().saturating_sub(1)).min(context.len());
    let ids: Vec<Option<WordId>> = context.iter().map(|w| id_of(pack, w)).collect();
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
fn id_of(pack: &Pack<'_>, word: &str) -> Option<WordId> {
    let run = pack.run(&fold(word));
    let spelt = |w: &str| run.clone().find(|&id| pack.spelling(id) == w);
    spelt(word)
        .or_else(|| spelt(&word.to_lowercase()))
        .or_else(|| run.clone().next())
}

/// The id of the seed word spelt exactly `word`.
fn exact(pack: &Pack<'_>, word: &str) -> Option<WordId> {
    pack.run(&fold(word)).find(|&id| pack.spelling(id) == word)
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
