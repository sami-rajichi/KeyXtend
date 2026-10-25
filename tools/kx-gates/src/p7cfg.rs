//! `kx-gates-p7.toml`: G27, a word typed through the product's first chip (`[g27]`), G21's product half
//! (`[g21_product]`) and G8's prediction part (`[g8_predict]`).
#![cfg(windows)]

use kx_mod_predict::script::{self, Script};
use serde::Deserialize;

use crate::config::GatesConfig;
use crate::p5cfg::SEP;

/// `[g27]`: the cases, the lane's gate ids and the waits; times in ms.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct G27 {
    /// One case per language.
    pub cases: Vec<Case>,
    /// The case edge 6 types, by its language.
    pub edge: String,
    /// The gate-id group of the chip lane.
    pub group: String,
    /// A chip's id before its place in the lane.
    pub chip: String,
    /// The lock note's id.
    pub locked: String,
    /// kx-ui's text that names a word chip.
    pub chip_name: String,
    /// That text's variable for the word.
    pub chip_word: String,
    /// Wait after the gate's own copy shows its keyboard before the first click.
    pub ready_ms: u64,
    /// Longest wait for a layout switch and for the keyboard's keys to follow it.
    pub switch_wait_ms: u64,
    /// Pause after each key click.
    pub key_gap_ms: u64,
    /// Longest wait from the last key click to a first chip that offers the prefix.
    pub chip_wait_ms: u64,
    /// Pause before a chip is read and clicked.
    pub settle_ms: u64,
    /// Longest wait for the box to hold the picked word; edge 6 waits it whole.
    pub text_wait_ms: u64,
    /// Longest box text read, in UTF-16 units.
    pub max_chars: usize,
}

/// One language's case: its layout's language and the prefix typed.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Case {
    /// The layout's language, by ISO 639 code.
    pub lang: String,
    /// What the product's keys type before a chip is picked.
    pub prefix: String,
}

/// `[g21_product]`: what G21's product half types.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct G21Product {
    /// The G27 case it types, by its language.
    pub case: String,
}

/// `[g8_predict]`: G8's prediction part.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct G8Predict {
    /// The owner's accepted predict part, in MB; a reading, never a fail.
    pub owner_mb: f64,
    /// Words typed before the typing churn's first reading.
    pub warm_words: usize,
    /// Words the typing churn measures.
    pub words: usize,
    /// Wait after the last word before memory is read, in ms.
    pub settle_ms: u64,
}

impl G27 {
    /// The case for language `lang`.
    pub fn case(&self, lang: &str) -> Option<&Case> {
        self.cases.iter().find(|c| c.lang == lang)
    }

    /// The gate id every chip's starts with.
    pub fn chip_start(&self) -> String {
        format!("{}{SEP}{}", self.group, self.chip)
    }

    /// The gate id of the chip at `place`, 0 the best.
    pub fn chip_id(&self, place: usize) -> String {
        format!("{}{place}", self.chip_start())
    }

    /// The lock note's gate id.
    pub fn locked_id(&self) -> String {
        format!("{}{SEP}{}", self.group, self.locked)
    }
}

/// Refuses a wait of `ms` outside 1 ms to `most`, the mouse guard's limit, naming `key`.
fn wait(key: &str, ms: u64, most: u64) -> Result<(), String> {
    if (1..=most).contains(&ms) {
        Ok(())
    } else {
        Err(format!(
            "{key} must be 1 to {most} ms, the mouse guard's limit"
        ))
    }
}

/// The sum of `parts`, at most `u64::MAX`.
fn total(parts: &[u64]) -> u64 {
    parts.iter().fold(0, |sum, &ms| sum.saturating_add(ms))
}

/// `n` times `ms`, at most `u64::MAX`.
fn times(n: usize, ms: u64) -> u64 {
    u64::try_from(n).map_or(u64::MAX, |n| n.saturating_mul(ms))
}

impl Case {
    /// Refuses a language that is no lower-case ISO 639 code, or a prefix that is not letters of one spaced script,
    /// which word chips serve.
    fn check(&self) -> Result<(), String> {
        let code = !self.lang.is_empty() && self.lang.chars().all(|c| c.is_ascii_lowercase());
        let spaced = script::of(&self.prefix).is_some_and(Script::spaced);
        let letters = self.prefix.chars().all(char::is_alphabetic) && script::single(&self.prefix);
        if code && spaced && letters {
            return Ok(());
        }
        Err(format!(
            "g27.cases: {:?} needs a lower-case ISO 639 code and a prefix of letters of one spaced script",
            self.lang
        ))
    }
}

impl G27 {
    /// Refuses no case, a bad case, a language twice, and an edge case that is missing or reads right to left, as
    /// edge 6 clicks the box's left end.
    fn check_cases(&self) -> Result<(), String> {
        if self.cases.is_empty() {
            return Err("g27.cases must hold at least one case".to_string());
        }
        self.cases.iter().try_for_each(Case::check)?;
        let mut langs: Vec<&str> = self.cases.iter().map(|c| c.lang.as_str()).collect();
        langs.sort_unstable();
        langs.dedup();
        if langs.len() != self.cases.len() {
            return Err("g27.cases must name each language once".to_string());
        }
        let edge = self.case(&self.edge);
        let edge = edge.ok_or_else(|| format!("g27.edge: {} names no case", self.edge))?;
        if script::of(&edge.prefix).is_none_or(Script::rtl) {
            return Err("g27.edge: its case's prefix must read left to right".to_string());
        }
        Ok(())
    }

    /// Refuses an empty or dotted group, an empty id, and a chip name kx-ui lacks in a language.
    fn check_ids(&self) -> Result<(), String> {
        if self.group.is_empty() || self.group.contains(SEP) {
            return Err(format!("g27.group must be a name without {SEP}"));
        }
        if self.chip.is_empty() || self.locked.is_empty() {
            return Err("g27.chip and g27.locked must not be empty".to_string());
        }
        crate::ftl::around(&self.chip_name, &self.chip_word)
            .map(drop)
            .map_err(|e| format!("g27.chip_name and g27.chip_word: {e}"))
    }

    /// Refuses bad cases or ids, a wait outside 1 ms to `most_ms`, and a read cap outside 1 to `most_chars`.
    fn check(&self, most_ms: u64, most_chars: usize) -> Result<(), String> {
        self.check_cases()?;
        self.check_ids()?;
        let waits = [
            ("g27.ready_ms", self.ready_ms),
            ("g27.switch_wait_ms", self.switch_wait_ms),
            ("g27.key_gap_ms", self.key_gap_ms),
            ("g27.chip_wait_ms", self.chip_wait_ms),
            ("g27.settle_ms", self.settle_ms),
            ("g27.text_wait_ms", self.text_wait_ms),
        ];
        waits
            .iter()
            .try_for_each(|(key, ms)| wait(key, *ms, most_ms))?;
        if !(1..=most_chars).contains(&self.max_chars) {
            return Err(format!(
                "g27.max_chars must be 1 to {most_chars}, g1.max_read_chars"
            ));
        }
        Ok(())
    }
}

/// The cold starts `g8 predict` makes: predict on, then off.
const RUNS: usize = 2;

/// The least time `g8 predict` holds the mouse: each cold start and its warm-up, each typed word at its longest, the
/// settle and the run's headroom.
fn predict_ms(c: &GatesConfig) -> u64 {
    let (g, p) = (&c.g27, &c.g8_predict);
    let keys = g.cases.iter().map(|k| k.prefix.chars().count()).max();
    let typing = times(keys.unwrap_or_default(), g.key_gap_ms);
    let word = total(&[
        g.switch_wait_ms,
        typing,
        g.chip_wait_ms,
        g.settle_ms,
        g.text_wait_ms,
    ]);
    let start = c.g8.start_wait_ms.saturating_add(c.g8.warm_ms);
    let words = times(p.warm_words.saturating_add(p.words), word);
    total(&[
        times(RUNS, start),
        words,
        p.settle_ms,
        c.fit.guard_headroom_ms,
    ])
}

impl G8Predict {
    /// Refuses a figure outside 0 to `[g8] memory_mb`, no words, a bad settle, and a run past the mouse guard's limit.
    fn check(&self, c: &GatesConfig) -> Result<(), String> {
        let most = c.g8.memory_mb;
        if !(self.owner_mb > 0.0 && self.owner_mb <= most) {
            return Err(format!(
                "g8_predict.owner_mb must be above 0 and at most g8.memory_mb ({most} MB)"
            ));
        }
        if self.warm_words == 0 || self.words == 0 {
            return Err("g8_predict.warm_words and g8_predict.words must be above 0".to_string());
        }
        let limit = c.mouse_guard.limit_ms;
        wait("g8_predict.settle_ms", self.settle_ms, limit)?;
        let need = predict_ms(c);
        if need > limit {
            return Err(format!(
                "g8_predict.words: g8 predict takes at least {need} ms, past the mouse guard's {limit} ms"
            ));
        }
        Ok(())
    }
}

/// Refuses P7 settings that could type nothing, pick the wrong word, wait past the mouse guard or hide a figure.
pub fn check(c: &GatesConfig) -> Result<(), String> {
    c.g27.check(c.mouse_guard.limit_ms, c.g1.max_read_chars)?;
    if c.g27.case(&c.g21_product.case).is_none() {
        return Err(format!(
            "g21_product.case: {} names no g27 case",
            c.g21_product.case
        ));
    }
    c.g8_predict.check(c)
}

#[cfg(test)]
mod tests;
