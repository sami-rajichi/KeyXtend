//! G8's tip hovers key by key: the polls through each rest, and per key the hits, the tips that hid again and when
//! each tip showed.
#![cfg(windows)]

use serde::Serialize;

use crate::stats;

/// One look during a rest: ms since the pointer moved, and whether a new product window showed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Poll {
    /// Ms since the pointer moved onto the key, taken just after the look, so a slow look counts toward the rest.
    pub ms: u64,
    /// A product window showed that was not shown before the round's first hover.
    pub shown: bool,
}

/// What one hover's polls saw.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hover {
    /// A new window showed at the rest's end, as G8 has always counted a tip.
    pub hit: bool,
    /// When the tip first showed, in ms after the move; none when it never showed during the rest.
    pub shown_ms: Option<u64>,
}

impl Hover {
    /// The hover its `polls` saw, in order. A window showing as the pointer arrives is the last key's tip, so this
    /// key's tip shows only after a poll that saw none.
    pub fn of(polls: &[Poll]) -> Self {
        let mut fresh = polls.iter().skip_while(|p| p.shown);
        Self {
            hit: polls.last().is_some_and(|p| p.shown),
            shown_ms: fresh.find(|p| p.shown).map(|p| p.ms),
        }
    }

    /// The tip showed, then hid before the rest ended.
    pub fn hid(self) -> bool {
        self.shown_ms.is_some() && !self.hit
    }
}

/// Polls `shown` every `poll_ms` until `rest_ms` after the move, with `since` giving the ms since the move and `wait`
/// waiting; the last poll ends at or just past the rest's end, and nothing runs after it.
pub fn rest(
    rest_ms: u64,
    poll_ms: u64,
    mut since: impl FnMut() -> u64,
    mut wait: impl FnMut(u64),
    mut shown: impl FnMut() -> bool,
) -> Vec<Poll> {
    let mut polls = Vec::new();
    loop {
        wait(rest_ms.saturating_sub(since()).min(poll_ms));
        let seen = shown();
        let ms = since();
        polls.push(Poll { ms, shown: seen });
        if ms >= rest_ms {
            return polls;
        }
    }
}

/// One tip key's hovers so far: hits, tips that hid again, and each hit's show time in ms.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct Tally {
    hits: usize,
    hid: usize,
    shown_ms: Vec<u64>,
}

/// The hovers of each tip key, by its place in `[g8] churn_tips`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Keys(Vec<Tally>);

impl Keys {
    /// Counts hover `h` on tip key number `i`.
    pub fn add(&mut self, i: usize, h: Hover) {
        if self.0.len() <= i {
            self.0.resize_with(i + 1, Tally::default);
        }
        let t = &mut self.0[i];
        t.hits += usize::from(h.hit);
        t.hid += usize::from(h.hid());
        if let (true, Some(ms)) = (h.hit, h.shown_ms) {
            t.shown_ms.push(ms);
        }
    }

    /// Each key of `ids`, in that order, as G8's result lists it.
    pub fn report(&self, ids: &[String]) -> Vec<KeyTips> {
        let none = Tally::default();
        let tally = |i: usize| self.0.get(i).unwrap_or(&none);
        ids.iter()
            .enumerate()
            .map(|(i, id)| tally(i).report(id))
            .collect()
    }
}

impl Tally {
    /// This key, named `id`, as G8's result lists it.
    fn report(&self, id: &str) -> KeyTips {
        let mut sorted: Vec<f64> = self.shown_ms.iter().map(|&ms| as_f64(ms)).collect();
        sorted.sort_by(f64::total_cmp);
        KeyTips {
            id: id.to_string(),
            hits: self.hits,
            hid: self.hid,
            show_p50_ms: stats::median(&sorted),
            show_max_ms: self.shown_ms.iter().copied().max(),
        }
    }
}

/// `ms` as a float, for the median.
#[allow(clippy::cast_precision_loss, reason = "A rest is far below 2^53 ms.")]
fn as_f64(ms: u64) -> f64 {
    ms as f64
}

/// One tip key in G8's result.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct KeyTips {
    /// The key's board id.
    pub id: String,
    /// Hovers whose tip showed at the rest's end.
    pub hits: usize,
    /// Hovers whose tip showed, then hid before the rest ended.
    pub hid: usize,
    /// The median show time of the hits, in ms after the move; none without a timed hit.
    pub show_p50_ms: Option<f64>,
    /// The slowest show time of the hits, in ms after the move.
    pub show_max_ms: Option<u64>,
}

#[cfg(test)]
mod tests;
