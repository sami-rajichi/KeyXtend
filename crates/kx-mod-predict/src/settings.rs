//! The `[predict]` settings: the lane, the engine's limits and weights, emails, and how the box is read and the
//! learned words saved. Each range is a named const, and the engine's own limits bound them.

use std::ops::RangeInclusive;

use kx_module_api::ports::FieldConfig;
use kx_module_api::{Settings, SettingsError, SettingsSpec, validate_as, within};
use serde::{Deserialize, Serialize};

use crate::email::{EmailConfig, is_domain};
use crate::learn::{LearnConfig, MAX_LEARNED};
use crate::pack::layout::MAX_CHAIN;
use crate::score::{BACKOFFS, ScoreConfig, WEIGHTS};
use crate::track::TrackConfig;

/// The chips in the lane.
pub const CHIPS: RangeInclusive<u8> = 3..=7;
/// The previous words a guess reads: up to the pack's longest chain less the guessed word.
pub const CONTEXT_WORDS: RangeInclusive<u8> = 1..=MAX_CHAIN - 1;
/// The uses before a word is offered; a test keeps it within the learner's `MAX_MIN_USES`.
pub const MIN_USES: RangeInclusive<u32> = 1..=9;
/// The learned words kept per language, up to the most a learner holds.
pub const MAX_WORDS: RangeInclusive<usize> = 100..=MAX_LEARNED;
/// The learned chains kept of each length; a test keeps it within the learner's `MAX_CHAINS`.
pub const MAX_CHAINS: RangeInclusive<usize> = 1_000..=300_000;
/// The days an unused entry is kept.
pub const FORGET_DAYS: RangeInclusive<u32> = 30..=3_650;
/// The learned counts' most weight: the scorer's own bound.
pub const LEARN_WEIGHT: RangeInclusive<f64> = WEIGHTS;
/// The typed words at which the learned counts weigh half their most.
pub const LEARN_RAMP: RangeInclusive<u32> = 100..=100_000;
/// The step down to a shorter context: the scorer's own bound.
pub const BACKOFF: RangeInclusive<f64> = BACKOFFS;
/// The longest word with chips, in characters.
pub const MAX_WORD: RangeInclusive<usize> = 16..=64;
/// The known emails kept.
pub const MAX_EMAILS: RangeInclusive<usize> = 5..=200;
/// The typed characters before a known email is offered.
pub const MIN_START: RangeInclusive<usize> = 1..=10;
/// How many domains may be listed.
pub const DOMAINS: RangeInclusive<u8> = 0..=20;
/// The longest read of the box, in ms.
pub const READ_MS: RangeInclusive<u32> = 50..=1_000;
/// The longest text one read takes, in characters.
pub const READ_CHARS: RangeInclusive<u16> = 64..=4_096;
/// The seconds from a first unsaved change to its save.
pub const SAVE_S: RangeInclusive<u32> = 5..=600;

/// The key the menu's Pause writes.
pub const LEARN_KEY: &str = "learn";
/// The key the menu's Off writes.
pub const ON_KEY: &str = "on";

/// Why a domain was refused.
const BAD_DOMAIN: &str = "domains must each be a domain such as example.com";

/// How the kernel reads the `[predict]` section: version 1, with the defaults embedded.
pub static SPEC: SettingsSpec = SettingsSpec {
    version: 1,
    defaults: include_str!("../defaults.toml"),
    migrations: &[],
    validate: validate_as::<PredictSettings>,
};

/// The module's settings; every value comes from `defaults.toml` or the user's file.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PredictSettings {
    /// Word chips are shown and suggested.
    pub on: bool,
    /// Finished words and emails are learned; false pauses learning.
    pub learn: bool,
    /// The most chips in the lane.
    pub chips: u8,
    /// How many previous words guess the next one.
    pub context_words: u8,
    /// The uses a word needs before it is offered, and a forgotten seed word needs to come back.
    pub min_uses: u32,
    /// The most learned words kept per language.
    pub max_words: usize,
    /// The most learned chains kept of each length, per language.
    pub max_chains: usize,
    /// Days unused after which a learned entry is dropped.
    pub forget_days: u32,
    /// The most weight the learned counts reach against the seed.
    pub learn_weight: f64,
    /// The typed words at which the learned counts reach half their most weight.
    pub learn_ramp: u32,
    /// Each step down to a shorter context multiplies a score by this.
    pub backoff: f64,
    /// The longest word that gets chips and is learned, in characters.
    pub max_word: usize,
    /// The most known emails kept.
    pub max_emails: usize,
    /// The fewest typed characters before a known email is offered.
    pub min_start: usize,
    /// The domains offered after one `@`, in order.
    pub domains: Vec<String>,
    /// The longest one read of the box may take, in ms.
    pub read_ms: u32,
    /// The longest text one read takes before the cursor, in characters.
    pub read_chars: u16,
    /// Changed blobs are saved this many seconds after the first change, and at stop.
    pub save_s: u32,
}

impl PredictSettings {
    /// The settings in the embedded `defaults.toml`.
    ///
    /// # Errors
    /// `SettingsError::Parse` when the defaults do not parse, or the error from `check`.
    pub fn from_defaults() -> Result<Self, SettingsError> {
        SPEC.defaults_as()
    }

    /// What the tracker needs.
    #[must_use]
    pub fn track_config(&self) -> TrackConfig {
        TrackConfig {
            context_words: usize::from(self.context_words),
            max_word: self.max_word,
        }
    }

    /// What the scorer needs; `min_uses` is the learner's too.
    #[must_use]
    pub fn score_config(&self) -> ScoreConfig {
        ScoreConfig {
            backoff: self.backoff,
            learn_weight: self.learn_weight,
            learn_ramp: self.learn_ramp,
            min_uses: self.min_uses,
        }
    }

    /// What the learner needs; `min_uses` is the scorer's too.
    #[must_use]
    pub fn learn_config(&self) -> LearnConfig {
        LearnConfig {
            min_uses: self.min_uses,
            max_words: self.max_words,
            max_chains: self.max_chains,
            forget_days: self.forget_days,
            context_words: usize::from(self.context_words),
        }
    }

    /// What the email chips need.
    #[must_use]
    pub fn email_config(&self) -> EmailConfig {
        EmailConfig {
            max_emails: self.max_emails,
            domains: self.domains.clone(),
            min_start: self.min_start,
        }
    }

    /// How the platform reads the focused box.
    #[must_use]
    pub fn field_config(&self) -> FieldConfig {
        FieldConfig {
            read_ms: self.read_ms,
            read_chars: self.read_chars,
        }
    }

    /// The domain list's rules.
    fn check_domains(&self) -> Result<(), SettingsError> {
        let count = u8::try_from(self.domains.len()).unwrap_or(u8::MAX);
        within("domains", count, &DOMAINS)?;
        if !self.domains.iter().all(|d| is_domain(d)) {
            return Err(SettingsError::Invalid(BAD_DOMAIN.to_owned()));
        }
        Ok(())
    }
}

impl Settings for PredictSettings {
    /// Each range, then the domains, then what the scorer and the learner refuse, so `guesses` never gets a bad config.
    fn check(&self) -> Result<(), SettingsError> {
        within("chips", self.chips, &CHIPS)?;
        within("context_words", self.context_words, &CONTEXT_WORDS)?;
        within("min_uses", self.min_uses, &MIN_USES)?;
        within("max_words", self.max_words, &MAX_WORDS)?;
        within("max_chains", self.max_chains, &MAX_CHAINS)?;
        within("forget_days", self.forget_days, &FORGET_DAYS)?;
        within("learn_weight", self.learn_weight, &LEARN_WEIGHT)?;
        within("learn_ramp", self.learn_ramp, &LEARN_RAMP)?;
        within("backoff", self.backoff, &BACKOFF)?;
        within("max_word", self.max_word, &MAX_WORD)?;
        within("max_emails", self.max_emails, &MAX_EMAILS)?;
        within("min_start", self.min_start, &MIN_START)?;
        within("read_ms", self.read_ms, &READ_MS)?;
        within("read_chars", self.read_chars, &READ_CHARS)?;
        within("save_s", self.save_s, &SAVE_S)?;
        self.check_domains()?;
        let invalid = |e: &dyn std::fmt::Display| SettingsError::Invalid(e.to_string());
        self.score_config().check().map_err(|e| invalid(&e))?;
        self.learn_config().check().map_err(|e| invalid(&e))
    }
}

#[cfg(test)]
mod tests;
