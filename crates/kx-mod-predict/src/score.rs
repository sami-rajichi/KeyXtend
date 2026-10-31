//! The scorer (spec §6 "Engine", ADR-0032 decision 1): stupid backoff over the context, the seed's chains and
//! frequencies blended with the learned counts, and capitals that follow what was typed.

use kx_module_api::Redacted;
use kx_module_api::ports::TypingMode;
use kx_module_api::text::{Casing, fold, is_apostrophe};

use crate::pack::Pack;
use crate::script::Script;
use crate::track::{Query, Reading};

mod best;
mod case;
mod config;
mod model;
mod sink;

use best::Best;
use case::Case;
use model::Model;
use sink::Sink;

#[cfg(test)]
pub(crate) use config::CFG;
pub use config::{BACKOFFS, ScoreConfig, ScoreConfigError, WEIGHTS};

/// One guess, as its chip shows and types it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Guess {
    /// The text that replaces the open word.
    pub text: Redacted<String>,
    /// The word as the pack or the learner stores it, without the lead or the typed capitals; a forget passes it.
    pub stored: Redacted<String>,
}

/// What the learner tells the scorer; the learner implements it.
pub trait Learned {
    /// How many times `word` was learned.
    fn count(&self, word: &str) -> u32;
    /// How many times `word` was learned after `context`, its previous words oldest first.
    fn chain(&self, context: &[&str], word: &str) -> u32;
    /// How many words were learned in all.
    fn uses(&self) -> u64;
    /// True when `word` is a seed word the user forgot.
    fn blocked(&self, word: &str) -> bool;
    /// Calls `each` with every learned word whose folded key starts with `folded_prefix`, both folded with the casing
    /// of the language in front (`Casing::of(iso)`).
    fn starting(&self, folded_prefix: &str, each: &mut dyn FnMut(&str));
    /// Calls `each` with every learned word that followed `context`.
    fn following(&self, context: &[&str], each: &mut dyn FnMut(&str));
    /// Every learned word whose folded key starts with `folded_prefix`, most used first: the words `starting` lists,
    /// each with the uses `count` gives it. The early stop relies on both, as it stops once no word left can win.
    fn by_count<'s>(&'s self, folded_prefix: &str)
    -> Box<dyn Iterator<Item = (&'s str, u32)> + 's>;
}

/// A learner that knows nothing, for a language with learning off or before the learner loads.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoLearned;

impl Learned for NoLearned {
    fn count(&self, _: &str) -> u32 {
        0
    }

    fn chain(&self, _: &[&str], _: &str) -> u32 {
        0
    }

    fn uses(&self) -> u64 {
        0
    }

    fn blocked(&self, _: &str) -> bool {
        false
    }

    fn starting(&self, _: &str, _: &mut dyn FnMut(&str)) {}

    fn following(&self, _: &[&str], _: &mut dyn FnMut(&str)) {}

    fn by_count<'s>(&'s self, _: &str) -> Box<dyn Iterator<Item = (&'s str, u32)> + 's> {
        Box::new(std::iter::empty())
    }
}

/// True when `seed` holds `word` in any case or accents; false without a pack. The tracker splits apostrophes by it.
#[must_use]
pub fn knows(seed: Option<&Pack<'_>>, word: &str) -> bool {
    seed.is_some_and(|pack| !pack.run(&fold(word, pack.casing())).is_empty())
}

/// At most `limit` distinct guesses for `query`, best first, from the `seed` pack and the `learned` counts of `iso`,
/// the language in front; capitals follow each reading's typed prefix and `mode`.
#[must_use]
pub fn guesses(
    iso: &str,
    seed: Option<&Pack<'_>>,
    learned: &dyn Learned,
    query: &Query,
    mode: TypingMode,
    limit: usize,
    cfg: &ScoreConfig,
) -> Vec<Guess> {
    Ask::new(iso, seed, learned, query, mode, cfg).guesses(limit)
}

/// How a reading's word list is read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Scan {
    /// Best first, stopping once no word left can win.
    Ranked,
    /// Every word, the reference the early stop must match.
    #[cfg(all(test, feature = "build"))]
    Every,
}

/// What every reading of one query is scored against.
struct Ask<'a> {
    /// The pack, when the language has one.
    seed: Option<&'a Pack<'a>>,
    /// The learned counts.
    learned: &'a dyn Learned,
    /// The query, so a guess that changes nothing is dropped.
    query: &'a Query,
    /// The latches.
    mode: TypingMode,
    /// The settings.
    cfg: &'a ScoreConfig,
    /// How the language in front cases its words, with or without a pack.
    casing: &'static Casing,
    /// How each word list is read.
    scan: Scan,
}

impl<'a> Ask<'a> {
    /// The ask for `query` in `iso`, folding with its casing; a pack of another language is ignored.
    fn new(
        iso: &str,
        seed: Option<&'a Pack<'a>>,
        learned: &'a dyn Learned,
        query: &'a Query,
        mode: TypingMode,
        cfg: &'a ScoreConfig,
    ) -> Self {
        Self {
            seed: seed.filter(|pack| pack.iso() == iso),
            learned,
            query,
            mode,
            cfg,
            casing: Casing::of(iso),
            scan: Scan::Ranked,
        }
    }

    /// At most `limit` distinct guesses, best first, over every reading. None at once for settings `check` refuses, or
    /// when the pack's script spaces no words, as those wait for the CJK input mode.
    fn guesses(&self, limit: usize) -> Vec<Guess> {
        let script = self.seed.and_then(Pack::script);
        if !script.is_none_or(Script::spaced) || self.cfg.check().is_err() {
            return Vec::new();
        }
        let mut best = Best::new(limit);
        for reading in &self.query.readings {
            self.offer(reading, &mut best);
        }
        let guess = |(text, stored): (String, String)| Guess {
            text: Redacted::new(text),
            stored: Redacted::new(stored),
        };
        best.into_texts().into_iter().map(guess).collect()
    }

    /// Offers `best` every candidate of `reading`, prefixed by its lead, in the case its prefix asks for, and weighed
    /// by its elisions' chance. After an elision its followers come first, and the word list only for a typed prefix
    /// none fits.
    fn offer(&self, reading: &Reading, best: &mut Best) {
        let context: Vec<&str> = reading
            .context
            .expose()
            .iter()
            .map(String::as_str)
            .collect();
        let (lead, prefix) = (reading.lead.expose(), reading.prefix.expose());
        let elisions = lead.chars().filter(|&c| is_apostrophe(c)).count();
        let kept = elisions.min(context.len());
        let mut sink = Sink {
            best,
            query: self.query,
            lead,
            case: Case::of(prefix, self.mode),
            casing: self.casing,
            scale: self.lead_weight(&context, kept),
        };
        let model = Model::new(self.seed, self.learned, &context, self.cfg, self.casing);
        let folded = fold(prefix, self.casing);
        if kept == 0 || (!model.offer_followers(&folded, &mut sink) && !folded.is_empty()) {
            match self.scan {
                Scan::Ranked => model.offer(&folded, &mut sink),
                #[cfg(all(test, feature = "build"))]
                Scan::Every => model.offer_every(&folded, &mut sink),
            }
        }
    }

    /// The chance of the last `kept` words of `context`, the lead's elisions, each after the words before it.
    fn lead_weight(&self, context: &[&str], kept: usize) -> f64 {
        let first = context.len().saturating_sub(kept);
        let each = |at: usize| {
            let (before, word) = (context.get(..at)?, context.get(at)?);
            let model = Model::new(self.seed, self.learned, before, self.cfg, self.casing);
            Some(model.score(word))
        };
        (first..context.len()).filter_map(each).product()
    }
}

/// `n` as a float, for the learned shares.
#[allow(
    clippy::cast_precision_loss,
    reason = "Word counts stay far below 2^52, where an f64 is still exact."
)]
fn float(n: u64) -> f64 {
    n as f64
}

#[cfg(all(test, feature = "build"))]
mod fake;
#[cfg(test)]
mod tests;
