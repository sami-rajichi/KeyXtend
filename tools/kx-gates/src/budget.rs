//! G8's pass rules: each spec §10 budget against what the run measured, the log's start-up span, the idle share of
//! one core and the highlight's frame budget.
#![cfg(windows)]

use std::time::Duration;

use kx_module_api::{MS_PER_S, US_PER_S};
use serde::Serialize;

use crate::stats::{P50, US_PER_MS, percentile};

/// Seconds in one day.
const SECS_PER_DAY: i64 = 86_400;
/// Seconds in one hour and in one minute.
const SECS_PER_HOUR: i64 = 3_600;
const SECS_PER_MIN: i64 = 60;
/// Digits of a microsecond fraction.
const US_DIGITS: usize = 6;
/// Percent of a whole.
const PCT: f64 = 100.0;

/// One budget: what the run measured, the most it may be, and whether it passed; nothing measured fails.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Part {
    /// The budget's name in the result.
    pub name: &'static str,
    /// What the run measured.
    pub got: Option<f64>,
    /// The most it may be.
    pub limit: f64,
    /// True when it was measured and within the limit.
    pub pass: bool,
}

impl Part {
    /// Budget `name`: `got` may be at most `limit`.
    pub fn at_most(name: &'static str, got: Option<f64>, limit: f64) -> Self {
        let pass = got.is_some_and(|g| g <= limit);
        Self {
            name,
            got,
            limit,
            pass,
        }
    }
}

/// True when there are parts and every one passed.
pub fn all_pass(parts: &[Part]) -> bool {
    !parts.is_empty() && parts.iter().all(|p| p.pass)
}

/// Days from 1970-01-01 to the civil date `y-m-d` (Howard Hinnant's `days_from_civil`).
fn days(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// The number in `text[at..at + len]`.
fn num(text: &str, at: usize, len: usize) -> Option<i64> {
    let part = text.get(at..at + len)?;
    part.bytes().all(|b| b.is_ascii_digit()).then_some(())?;
    part.parse().ok()
}

/// Microseconds since 1970 of the UTC stamp that starts a log line, such as `2026-10-01T18:29:22.924463Z`.
pub fn stamp_us(line: &str) -> Option<i64> {
    let stamp = line.split_whitespace().next()?.strip_suffix('Z')?;
    let (clock, fraction) = stamp.split_once('.').unwrap_or((stamp, ""));
    let [y, mo, d, h, mi, s] =
        [(0, 4), (5, 2), (8, 2), (11, 2), (14, 2), (17, 2)].map(|(at, len)| num(clock, at, len));
    let date = days(y?, mo?, d?);
    let secs = date * SECS_PER_DAY + h? * SECS_PER_HOUR + mi? * SECS_PER_MIN + s?;
    let digits: String = fraction
        .chars()
        .chain(std::iter::repeat('0'))
        .take(US_DIGITS)
        .collect();
    let us = if fraction.is_empty() {
        0
    } else {
        num(&digits, 0, US_DIGITS)?
    };
    Some(secs * i64::from(US_PER_S) + us)
}

/// The stamps of the first line holding `start` and of the first later line holding `end`.
pub fn span_us(log: &str, start: &str, end: &str) -> Option<(i64, i64)> {
    let mut lines = log.lines();
    let first = lines
        .by_ref()
        .find(|l| l.contains(start))
        .and_then(stamp_us)?;
    let last = lines.find(|l| l.contains(end)).and_then(stamp_us)?;
    Some((first, last))
}

/// `d` in ms.
pub fn ms(d: Duration) -> f64 {
    d.as_secs_f64() * f64::from(MS_PER_S)
}

/// Milliseconds between two microsecond stamps.
#[allow(clippy::cast_precision_loss, reason = "Spans are far below 2^53 µs.")]
pub fn ms_between(from_us: i64, to_us: i64) -> f64 {
    (to_us - from_us) as f64 / US_PER_MS
}

/// CPU used, in percent of one core: `cpu_ms` of work over `wall_ms` of time; none when no time passed.
pub fn core_pct(cpu_ms: f64, wall_ms: f64) -> Option<f64> {
    (wall_ms > 0.0).then(|| cpu_ms / wall_ms * PCT)
}

/// The longest a press may take to light its key: `frames` frames at `hz`, plus the median screen copy, which every
/// look at the key costs; none without a refresh rate or a copy.
pub fn lit_within_ms(hz: f64, frames: f64, copy_ms: &[f64]) -> Option<f64> {
    let mut copies = copy_ms.to_vec();
    copies.sort_by(f64::total_cmp);
    let copy = percentile(&copies, P50)?;
    (hz > 0.0).then(|| frames * f64::from(MS_PER_S) / hz + copy)
}

#[cfg(test)]
mod tests;
