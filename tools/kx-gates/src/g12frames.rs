//! G12's frames: the overlay's swap chain, picked from `PresentMon`'s presents by the ring spans, and its gaps inside
//! each span. `PresentMon` names no window, and the keyboard is idle while a ring plays, so the chain busiest inside
//! the spans is the overlay's; the result names it, so a wrong pick shows.
#![cfg(windows)]

use std::collections::{BTreeMap, BTreeSet};

use kx_module_api::MS_PER_S;
use serde::Serialize;

use crate::presentmon::Present;
use crate::stats::{P99, ms, percentile};

/// Why G12 fails when its capture ended before a ring span did.
const CUT: &str = "PresentMon's capture ended before the last ring span did, so frames there would go unread; raise \
     [g12] presentmon_secs";
/// Why no frame can be judged when no present falls in a ring span.
const APART: &str = "no present of the test build falls in a ring span, so PresentMon's times do not line up with G12's";

/// One press's ring span on the QPC clock, in µs, ends included: from when its ring shows to its release.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    /// When the ring shows.
    pub from_us: i64,
    /// When the press was let go.
    pub to_us: i64,
}

/// The overlay's frames inside the ring spans.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Ring {
    /// The swap chain judged to be the overlay's.
    pub chain: String,
    /// Its presents inside the spans.
    pub frames: usize,
    /// The gaps between its consecutive presents inside one span.
    pub gaps: usize,
    /// The spans holding at least one of those gaps; one with none never drew the ring.
    pub drawn: usize,
    /// Those gaps over their summed time, per second.
    pub fps: Option<f64>,
    /// The 99th percentile of those gaps, in ms.
    pub p99_ms: Option<f64>,
    /// Its presents inside the spans that never reached the screen.
    pub lost: usize,
}

/// The index of the span in `spans` that holds `at_us`.
fn span_of(spans: &[Span], at_us: i64) -> Option<usize> {
    spans
        .iter()
        .position(|s| (s.from_us..=s.to_us).contains(&at_us))
}

/// The swap chain with the most presents inside `spans`; none when no present falls in one.
pub fn chain(presents: &[Present], spans: &[Span]) -> Option<String> {
    let mut count: BTreeMap<&str, usize> = BTreeMap::new();
    for p in presents
        .iter()
        .filter(|p| span_of(spans, p.at_us).is_some())
    {
        *count.entry(p.chain.as_str()).or_default() += 1;
    }
    let busiest = count.into_iter().max_by_key(|&(_, n)| n);
    busiest.map(|(c, _)| c.to_string())
}

/// Frames a second from `gaps` gaps that sum to `sum_ms`; none without a gap.
#[allow(clippy::cast_precision_loss, reason = "Gap counts are far below 2^53.")]
fn per_s(gaps: usize, sum_ms: f64) -> Option<f64> {
    (sum_ms > 0.0).then(|| gaps as f64 * f64::from(MS_PER_S) / sum_ms)
}

/// `chain`'s frames inside `spans`, from `presents` in time order: a gap counts only between two presents of one span.
pub fn ring(presents: &[Present], chain: &str, spans: &[Span]) -> Ring {
    let mine = presents.iter().filter(|p| p.chain == chain);
    let inside: Vec<(usize, &Present)> = mine
        .filter_map(|p| Some((span_of(spans, p.at_us)?, p)))
        .collect();
    let same: Vec<_> = inside.windows(2).filter(|w| w[0].0 == w[1].0).collect();
    let drawn: BTreeSet<usize> = same.iter().map(|w| w[0].0).collect();
    let mut gaps: Vec<f64> = same
        .iter()
        .map(|w| ms(w[1].1.at_us - w[0].1.at_us))
        .collect();
    gaps.sort_by(f64::total_cmp);
    Ring {
        chain: chain.to_string(),
        frames: inside.len(),
        gaps: gaps.len(),
        drawn: drawn.len(),
        fps: per_s(gaps.len(), gaps.iter().sum()),
        p99_ms: percentile(&gaps, P99),
        lost: inside.iter().filter(|(_, p)| !p.shown).count(),
    }
}

/// The overlay's frames among `presents` (in time order) inside `spans`; an error when no present falls in a span.
pub fn judge(presents: &[Present], spans: &[Span]) -> Result<Ring, String> {
    let c = chain(presents, spans).ok_or(APART)?;
    Ok(ring(presents, &c, spans))
}

/// An error when a span in `spans` ends after the capture's end at `end_us`, so G12 never judges fewer frames.
pub fn cut(spans: &[Span], end_us: i64) -> Result<(), String> {
    if spans.iter().any(|s| s.to_us > end_us) {
        return Err(CUT.to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests;
