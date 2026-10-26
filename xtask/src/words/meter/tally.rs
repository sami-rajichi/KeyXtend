//! One run's counts: sentences, words, presses without and with chips, and the time of each scorer call.

use super::slow_note;
use crate::words::float;
use crate::words::manifest::PERCENT;
use crate::words::report::shown_pct;

/// One run's counts over a language's sentences.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct Tally {
    /// The sentences typed.
    pub(crate) sentences: u64,
    /// Their words, by the packs' word rule.
    pub(crate) words: u64,
    /// The presses without chips: every character and space, and a Shift before each capital.
    pub(crate) without: u64,
    /// The presses with chips.
    pub(crate) with: u64,
    /// The time of each scorer call, in microseconds.
    pub(crate) times: Vec<u64>,
}

impl Tally {
    /// The share of presses the chips save, in percent: 1 − with / without; none when nothing was typed.
    pub(crate) fn saved_pct(&self) -> f64 {
        if self.without == 0 {
            return 0.0;
        }
        f64::from(PERCENT) * (1.0 - float(self.with) / float(self.without))
    }

    /// The line `iso`'s `run` prints: its counts, the saved share and each of `percentiles`, never a word.
    pub(crate) fn line(&self, iso: &str, run: &str, percentiles: &[u32]) -> String {
        format!(
            "{iso} {run}: {} sentences, {} words, {} presses without chips, {} with, saved {}%, guess {}{}",
            self.sentences,
            self.words,
            self.without,
            self.with,
            shown_pct(self.saved_pct()),
            times_text(&self.times, percentiles),
            slow_note(),
        )
    }
}

/// Each of `percentiles` of `times` as printed: `p50 120 us, p99 900 us`.
pub(crate) fn times_text(times: &[u64], percentiles: &[u32]) -> String {
    let parts: Vec<String> = (percentiles.iter())
        .map(|&p| format!("p{p} {} us", percentile(times, p)))
        .collect();
    parts.join(", ")
}

/// The `p` percentile of `times` by nearest rank, the ⌈n·p/100⌉-th smallest; 0 when there are none.
pub(crate) fn percentile(times: &[u64], p: u32) -> u64 {
    let mut sorted = times.to_vec();
    sorted.sort_unstable();
    let n = u64::try_from(sorted.len()).unwrap_or(u64::MAX);
    let rank = n.saturating_mul(u64::from(p)).div_ceil(u64::from(PERCENT));
    let at = usize::try_from(rank.saturating_sub(1)).unwrap_or(usize::MAX);
    let last = sorted.len().saturating_sub(1);
    sorted.get(at.min(last)).copied().unwrap_or_default()
}
