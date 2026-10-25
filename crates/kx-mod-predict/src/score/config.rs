//! The scorer's settings, which the module fills from `[predict]`, and the values the early stop can bound.

use std::ops::RangeInclusive;

use super::float;

/// The learned weights the early stop bounds: a share of the blend.
const WEIGHTS: RangeInclusive<f64> = 0.0..=1.0;

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

/// Why the scorer refuses its settings: the early stop cannot bound the scores they give.
#[derive(Copy, Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum ScoreConfigError {
    /// `learn_weight` is outside 0 to 1.
    #[error("learn_weight is outside 0 to 1")]
    BadWeight,
    /// `backoff` is below 0 or not a finite number.
    #[error("backoff is below 0 or not finite")]
    BadBackoff,
}

impl ScoreConfig {
    /// `Ok` when the early stop can bound the scores these settings give; `guesses` gives nothing otherwise.
    ///
    /// # Errors
    /// `BadWeight` when `learn_weight` is outside 0 to 1; `BadBackoff` when `backoff` is below 0 or not finite.
    pub fn check(&self) -> Result<(), ScoreConfigError> {
        if !WEIGHTS.contains(&self.learn_weight) {
            return Err(ScoreConfigError::BadWeight);
        }
        if !self.backoff.is_finite() || self.backoff < 0.0 {
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
