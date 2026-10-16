//! G12's frames: the overlay's swap chain, picked from `PresentMon`'s presents by the ring spans, its gaps inside each
//! span and the quiet time around them. `PresentMon` names no window, and the keyboard is idle while a ring plays, so
//! the chain busiest inside the spans is the overlay's; the result names it, so a wrong pick shows.
#![cfg(windows)]

use std::collections::{BTreeMap, BTreeSet};

use kx_module_api::MS_PER_S;
use serde::Serialize;

use crate::g8loss::{refresh_us, unshown};
use crate::presentmon::Present;
use crate::stats::{P99, ms, percentile, us_of};

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
    /// Those gaps over their summed time plus `quiet_ms`, per second, so a ring that stalls reads slow.
    pub fps: Option<f64>,
    /// The spans' time with no present: before each first one, past the ring's time to show, and after each last one.
    pub quiet_ms: f64,
    /// The 99th percentile of those gaps, in ms.
    pub p99_ms: Option<f64>,
    /// Its presents inside the spans lost under G8's frame rule, which skips the chain's first present and one replaced
    /// within a refresh.
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

/// The first and last present of each span in `inside`, in time order, by the span's index.
fn ends(inside: &[(usize, &Present)]) -> BTreeMap<usize, (i64, i64)> {
    let mut ends = BTreeMap::new();
    for &(i, p) in inside {
        ends.entry(i)
            .and_modify(|e: &mut (i64, i64)| e.1 = p.at_us)
            .or_insert((p.at_us, p.at_us));
    }
    ends
}

/// The time in `spans` with no present of `inside`, in ms: before each span's first present, less `show_us` for the
/// ring to show, and after its last; a span with none is quiet from `show_us` on.
fn quiet_ms(spans: &[Span], inside: &[(usize, &Present)], show_us: i64) -> f64 {
    let ends = ends(inside);
    let quiet = spans.iter().enumerate().map(|(i, s)| {
        let shown = s.from_us.saturating_add(show_us);
        match ends.get(&i) {
            Some(&(first, last)) => (first - shown).max(0) + (s.to_us - last),
            None => (s.to_us - shown).max(0),
        }
    });
    ms(quiet.sum())
}

/// `chain`'s frames inside `spans`, from `presents` in time order on a screen refreshing every `refresh_us`: a gap
/// counts only between two presents of one span, and the rate counts each span's quiet head, past `show_us`, and tail.
pub fn ring(
    presents: &[Present],
    chain: &str,
    spans: &[Span],
    refresh_us: f64,
    show_us: i64,
) -> Ring {
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
    let lost = unshown(presents, refresh_us)
        .0
        .into_iter()
        .map(|i| &presents[i]);
    let lost = lost.filter(|p| p.chain == chain && span_of(spans, p.at_us).is_some());
    let quiet = quiet_ms(spans, &inside, show_us);
    Ring {
        chain: chain.to_string(),
        frames: inside.len(),
        gaps: gaps.len(),
        drawn: drawn.len(),
        fps: per_s(gaps.len(), gaps.iter().sum::<f64>() + quiet),
        quiet_ms: quiet,
        p99_ms: percentile(&gaps, P99),
        lost: lost.count(),
    }
}

/// The overlay's frames among `presents` (in time order) inside `spans`, on a screen of `hz`, its ring given `show_ms`
/// to show; an error when the rate is unknown or no present falls in a span.
pub fn judge(
    presents: &[Present],
    spans: &[Span],
    hz: Option<f64>,
    show_ms: u64,
) -> Result<Ring, String> {
    let refresh_us = refresh_us(hz)?;
    let c = chain(presents, spans).ok_or(APART)?;
    Ok(ring(presents, &c, spans, refresh_us, us_of(show_ms)))
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
