//! The scorer's settings, which the module fills from `[predict]`, and the values the early stop can bound.

use std::ops::RangeInclusive;

use super::float;

/// The learned weights the early stop bounds: a share of the blend.
const WEIGHTS: RangeInclusive<f64> = 0.0..=1.0;
/// The backoffs the early stop bounds: a step down to a shorter context never weighs more (Brants et al., 2007).
const BACKOFFS: RangeInclusive<f64> = 0.0..=1.0;

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

/// The settings' defaults as Task 12 fills them, which the scorer's and the learner's tests share.
#[cfg(test)]
pub(crate) const CFG: ScoreConfig = ScoreConfig {
    backoff: 0.4,
    learn_weight: 0.6,
    learn_ramp: 2000,
    min_uses: 2,
};

/// Why the scorer refuses its settings: the early stop cannot bound the scores they give.
#[derive(Copy, Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum ScoreConfigError {
    /// `learn_weight` is outside 0 to 1.
    #[error("learn_weight is outside 0 to 1")]
    BadWeight,
    /// `backoff` is outside 0 to 1.
    #[error("backoff is outside 0 to 1")]
    BadBackoff,
}

impl ScoreConfig {
    /// `Ok` when the early stop can bound the scores these settings give; `guesses` gives nothing otherwise.
    ///
    /// # Errors
    /// `BadWeight` when `learn_weight` is outside 0 to 1; `BadBackoff` when `backoff` is.
    pub fn check(&self) -> Result<(), ScoreConfigError> {
        if !WEIGHTS.contains(&self.learn_weight) {
            return Err(ScoreConfigError::BadWeight);
        }
        if !BACKOFFS.contains(&self.backoff) {
            return Err(ScoreConfigError::BadBackoff);
        }
        Ok(())
    }

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
