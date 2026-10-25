//! The scorer (spec §6 "Engine", ADR-0032 decision 1): stupid backoff over the context, the seed's chains and
//! frequencies blended with the learned counts, and capitals that follow what was typed.

use kx_module_api::Redacted;
use kx_module_api::ports::TypingMode;

use crate::casing::Casing;
use crate::fold::{fold, is_apostrophe};
use crate::pack::Pack;
use crate::script::{self, Script};
use crate::track::{Query, Reading};

mod best;
mod case;
mod model;
mod sink;

use best::Best;
use case::Case;
use model::Model;
use sink::Sink;

/// The scorer's settings; the module fills them from `[predict]`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScoreConfig {
    /// Each step down to a shorter context multiplies a score by this (`backoff`).
    pub backoff: f64,
    /// The most weight the learned part reaches (`learn_weight`).
    pub learn_weight: f64,
    /// The typed words at which the learned part has half its most weight (`learn_ramp`).
    pub learn_ramp: u32,
    /// The uses a learned word that is not a seed word needs before it is offered (`min_uses`).
    pub min_uses: u32,
}

impl ScoreConfig {
    /// The learned part's weight after `uses` typed words: `learn_weight` × uses / (uses + `learn_ramp`).
    #[must_use]
    pub fn weight(&self, uses: u64) -> f64 {
        let uses = float(uses);
        let total = uses + f64::from(self.learn_ramp);
        if total > 0.0 {
            self.learn_weight * uses / total
        } else {
            0.0
        }
    }
}

/// One guess, as its chip shows and types it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Guess {
    /// The text that replaces the open word.
    pub text: Redacted<String>,
    /// The text is written right to left.
    pub rtl: bool,
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
}

/// True when `seed` holds `word` in any case or accents; false without a pack. The tracker splits apostrophes by it.
#[must_use]
pub fn knows(seed: Option<&Pack<'_>>, word: &str) -> bool {
    seed.is_some_and(|pack| !pack.run(&fold(word, pack.casing())).is_empty())
}

/// At most `limit` distinct guesses for `query`, best first, from the `seed` pack and the `learned` counts; capitals
/// follow each reading's typed prefix and `mode`.
#[must_use]
pub fn guesses(
    seed: Option<&Pack<'_>>,
    learned: &dyn Learned,
    query: &Query,
    mode: TypingMode,
    limit: usize,
    cfg: &ScoreConfig,
) -> Vec<Guess> {
    let ask = Ask {
        seed,
        learned,
        query,
        mode,
        cfg,
        casing: seed.map_or(Casing::root(), Pack::casing),
    };
    let mut best = Best::new(limit);
    for reading in &query.readings {
        ask.offer(reading, &mut best);
    }
    let guess = |text: String| Guess {
        rtl: script::of(&text).is_some_and(Script::rtl),
        text: Redacted::new(text),
    };
    best.into_texts().into_iter().map(guess).collect()
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
    /// How the pack's language cases its words.
    casing: &'static Casing,
}

impl Ask<'_> {
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
        let model = Model::new(self.seed, self.learned, &context, self.cfg);
        let folded = fold(prefix, self.casing);
        if kept == 0 || (!model.offer_followers(&folded, &mut sink) && !folded.is_empty()) {
            model.offer(&folded, &mut sink);
        }
    }

    /// The chance of the last `kept` words of `context`, the lead's elisions, each after the words before it.
    fn lead_weight(&self, context: &[&str], kept: usize) -> f64 {
        let first = context.len().saturating_sub(kept);
        let each = |at: usize| {
            let (before, word) = (context.get(..at)?, context.get(at)?);
            Some(Model::new(self.seed, self.learned, before, self.cfg).score(word))
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
