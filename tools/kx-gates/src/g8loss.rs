//! G8's frame rule (owner, 2026-10-11): `dropped_frames` counts only frames lost while a pressed key rises, from each
//! G8 press or G2 click to the end of its rise; losses in a resize or outside the presses are listed, not counted.
// Under `PresentMon --qpc_time_ms`, CPU start plus CPU busy is each frame's present time, on G8's `now_us` clock.
#![cfg(windows)]

use std::collections::HashSet;

use kx_module_api::US_PER_S;
use serde::Serialize;

use crate::presentmon::Present;
use crate::stats::{US_PER_MS_INT, ms};

/// Why no frame can be judged when the screen's refresh rate is unknown.
const NO_HZ: &str =
    "no refresh rate, so a frame replaced within a refresh cannot be told from a lost one";
/// Why G8 fails when its capture ended inside a press window.
const CUT_PRESS: &str = "PresentMon's capture ended inside a press window, so frames lost there would go uncounted; \
     raise [g8] presentmon_secs";
/// The note when the capture ended after every press window but before G2 ended.
const CUT_AFTER: &str = "PresentMon's capture ended after the last press window but before G2 ended, so only losses \
     outside the presses may be missing; raise [g8] presentmon_secs";
/// Why no frame can be judged when no present falls in a press.
const APART: &str =
    "no present of the product falls in a G8 press, so PresentMon's times do not line up with G8's";

/// What G8 was doing when a frame was lost.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum During {
    /// A pressed key rose: from a G8 press or G2 click to the end of its rise.
    Press,
    /// The keyboard resized: from the size button's press until its box settled.
    Resize,
    /// Anything else.
    Outside,
}

/// A stretch of G8's run on the QPC clock, in µs, ends included.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    from_us: i64,
    to_us: i64,
    during: During,
}

impl Span {
    /// A press at `at_us`, let go at `up_us`, and the rise of `rise_ms` that follows the release.
    pub fn press(at_us: i64, up_us: i64, rise_ms: u64) -> Self {
        let rise = i64::try_from(rise_ms).unwrap_or(i64::MAX);
        Self {
            from_us: at_us,
            to_us: up_us.saturating_add(rise.saturating_mul(US_PER_MS_INT)),
            during: During::Press,
        }
    }

    /// A resize from its press at `from_us` until its box settled at `to_us`.
    pub fn resize(from_us: i64, to_us: i64) -> Self {
        Self {
            from_us,
            to_us,
            during: During::Resize,
        }
    }

    /// True when `at_us` falls in it.
    fn holds(&self, at_us: i64) -> bool {
        (self.from_us..=self.to_us).contains(&at_us)
    }
}

/// Press windows for G2's clicks, each held from its press to its release in µs, as `[g2] press_ms` holds a click;
/// each rise follows its release.
pub fn clicks(clicks_us: &[(i64, i64)], rise_ms: u64) -> Vec<Span> {
    clicks_us
        .iter()
        .map(|&(at, up)| Span::press(at, up, rise_ms))
        .collect()
}

/// How a capture ending at `end_us` met G8's run, whose G2 ended at `ran_us`: an error when a press window in `spans`
/// ends after it, so G8 never counts fewer frames; a note when it ended before `ran_us`; else none.
pub fn cut(spans: &[Span], end_us: i64, ran_us: i64) -> Result<Option<&'static str>, String> {
    let presses = spans.iter().filter(|s| s.during == During::Press);
    if presses
        .map(|s| s.to_us)
        .max()
        .is_some_and(|last| last > end_us)
    {
        return Err(CUT_PRESS.to_string());
    }
    Ok((end_us < ran_us).then_some(CUT_AFTER))
}

/// One frame that never reached the screen: when, in ms from G8's start, and what G8 was doing.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Loss {
    /// When it was presented, in ms from G8's start.
    pub at_ms: f64,
    /// What G8 was doing.
    pub during: During,
}

/// The lost frames under G8's frame rule.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Losses {
    /// Lost while a pressed key rose: these are `dropped_frames`.
    pub counted: Vec<Loss>,
    /// Lost in a resize or outside the presses, for information.
    pub listed: Vec<Loss>,
    /// Never counted: a swap chain's first frame, or one a newer frame replaced within a refresh.
    pub skipped: usize,
}

/// What G8 was doing at `at_us`, by `spans`.
fn during(spans: &[Span], at_us: i64) -> During {
    let span = spans.iter().find(|s| s.holds(at_us));
    span.map_or(During::Outside, |s| s.during)
}

/// True when a newer present of the same swap chain follows present `i` within `refresh_us`.
#[allow(
    clippy::cast_precision_loss,
    reason = "Gaps between presents are far below 2^53 µs."
)]
fn replaced(presents: &[Present], i: usize, refresh_us: f64) -> bool {
    let p = &presents[i];
    let next = presents[i + 1..].iter().find(|q| q.chain == p.chain);
    next.is_some_and(|q| (q.at_us - p.at_us) as f64 <= refresh_us)
}

/// One refresh of a screen of `hz`, in µs; an error when the rate is unknown.
pub fn refresh_us(hz: Option<f64>) -> Result<f64, String> {
    hz.filter(|h| *h > 0.0)
        .map(|h| f64::from(US_PER_S) / h)
        .ok_or_else(|| NO_HZ.to_string())
}

/// The indexes of `presents` (in time order) lost under G8's frame rule, and how many unshown ones it skipped: a swap
/// chain's first present, and one a newer present of its chain replaced within `refresh_us`.
pub fn unshown(presents: &[Present], refresh_us: f64) -> (Vec<usize>, usize) {
    let (mut lost, mut skipped, mut seen) = (Vec::new(), 0, HashSet::new());
    for (i, p) in presents.iter().enumerate() {
        // Keyed by address alone, a chain re-made at a reused address is not skipped again, so it only counts more.
        let first = seen.insert(p.chain.as_str());
        if p.shown {
            continue;
        }
        if first || replaced(presents, i, refresh_us) {
            skipped += 1;
        } else {
            lost.push(i);
        }
    }
    (lost, skipped)
}

/// The losses among `presents` (in time order) against G8's `spans`, on a screen of `hz`, with times from `origin_us`;
/// an error when the times cannot be judged, so a run never passes on frames it could not place.
pub fn losses(
    presents: &[Present],
    spans: &[Span],
    hz: Option<f64>,
    origin_us: i64,
) -> Result<Losses, String> {
    let refresh_us = refresh_us(hz)?;
    if !presents
        .iter()
        .any(|p| during(spans, p.at_us) == During::Press)
    {
        return Err(APART.to_string());
    }
    let (gone, skipped) = unshown(presents, refresh_us);
    let mut out = Losses {
        skipped,
        ..Losses::default()
    };
    for p in gone.into_iter().map(|i| &presents[i]) {
        let at = during(spans, p.at_us);
        let loss = Loss {
            at_ms: ms(p.at_us - origin_us),
            during: at,
        };
        if at == During::Press {
            out.counted.push(loss);
        } else {
            out.listed.push(loss);
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests;
