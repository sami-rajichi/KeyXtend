//! Run lengths checked at load (owner, 2026-10-11): each run the mouse guard holds fits its limit, G8's capture covers
//! its steps through its G2 clicks, and G12's covers every press at its longest.
#![cfg(windows)]

use std::time::Duration;

use kx_mod_mouse::MouseSettings;
use serde::Deserialize;

use crate::config::GatesConfig;
use crate::g5play::{Case, cases};
use crate::gatecfg::G2;
use crate::p7cfg::runs::end_ms;

/// `[fit]`: what a run spends beyond the steps counted here; times in ms.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Fit {
    /// A guarded run's steps not counted here, such as starting apps, waits and clean-up.
    pub guard_headroom_ms: u64,
    /// G8's steps before its G2 clicks end that are not counted here: the cold start, resizes and each target-window.
    pub capture_headroom_ms: u64,
}

/// The least time `clicks` of G2's clicks take: each holds `press_ms`, then rests `click_gap_ms`.
pub fn clicks_ms(g2: &G2, clicks: usize) -> u64 {
    let each = g2.press_ms.saturating_add(g2.click_gap_ms);
    u64::try_from(clicks).map_or(u64::MAX, |n| n.saturating_mul(each))
}

/// `secs` in ms.
fn ms_of(secs: u64) -> u64 {
    u64::try_from(Duration::from_secs(secs).as_millis()).unwrap_or(u64::MAX)
}

/// The least time G8 holds the mouse: its steps through G2's clicks, then the use step's rest on each of the board's
/// `keys` and its wait, and the scroll and its wait, or its `PresentMon` capture when that ends later; then each churn
/// round resting on each tip and scrolling once, and its copy's end as the P7 gates count it.
fn g8_ms(c: &GatesConfig, keys: usize) -> u64 {
    let (g, scroll_ms) = (&c.g8, c.g8_scroll.scroll_ms);
    let rounds = g.churn_warm_rounds.saturating_add(g.churn_rounds);
    let rests = rounds.saturating_mul(g.churn_tips.len());
    let times = |n: usize, ms: u64| u64::try_from(n).map_or(u64::MAX, |n| n.saturating_mul(ms));
    let churn = total(&[times(rests, g.churn_tip_ms), times(rounds, scroll_ms)]);
    let used = total(&[g8_captured_ms(c), times(keys, g.use_hover_ms), g.warm_ms]);
    let scrolled = total(&[used, scroll_ms, g.warm_ms]);
    total(&[ms_of(g.presentmon_secs).max(scrolled), churn, end_ms(c)])
}

/// G8's counted steps from `PresentMon`'s start until its G2 clicks end: the warm-up, the idle time, each press at its
/// longest, the rise and the presses' drain, and the clicks with theirs.
fn g8_captured_ms(c: &GatesConfig) -> u64 {
    let g = &c.g8;
    let press = g.hover_settle_ms.saturating_add(g.press_wait_ms);
    let presses = u64::try_from(g.presses).map_or(u64::MAX, |n| n.saturating_mul(press));
    let waits = [
        g.presentmon_wait_ms,
        g.warm_ms,
        g.idle_settle_ms,
        g.idle_ms,
        presses,
        g.rise_wait_ms,
        c.g2.drain_ms,
        clicks_ms(&c.g2, g.clicks),
        c.g2.drain_ms,
    ];
    waits.iter().fold(0, |sum, &ms| sum.saturating_add(ms))
}

/// Refuses a G8 capture that would end before its G2 clicks, so their frames would go unseen.
fn check_capture(c: &GatesConfig) -> Result<(), String> {
    let need = g8_captured_ms(c).saturating_add(c.fit.capture_headroom_ms);
    let secs = c.g8.presentmon_secs;
    if ms_of(secs) < need {
        return Err(format!(
            "g8.presentmon_secs ({secs} s) ends before G8's G2 clicks end, {need} ms in: raise it"
        ));
    }
    Ok(())
}

/// The sum of `parts`, at most `u64::MAX`.
fn total(parts: &[u64]) -> u64 {
    parts.iter().fold(0, |sum, &ms| sum.saturating_add(ms))
}

/// G12's counted steps from `PresentMon`'s start: its wait, then each press at its longest, `hold_ms` less
/// `[g12] spare_ms`, with the move before it, the settle after it and a poll's lateness.
fn g12_ms(c: &GatesConfig, hold_ms: u64) -> u64 {
    let g = &c.g12;
    let press = hold_ms.saturating_sub(g.spare_ms);
    let each = total(&[c.sim.step_ms, press, g.settle_ms, c.timing.poll_ms]);
    let clicks = u64::from(g.grid[0]).saturating_mul(u64::from(g.grid[1]));
    clicks
        .saturating_mul(each)
        .saturating_add(c.g8.presentmon_wait_ms)
}

/// The time `rounds` rounds of G5's cases take under a hold of `hold_ms`, counted generously: each case's two steps,
/// its click or its hold with the margin, its settle, and the hung case's hang.
fn rounds_ms(c: &GatesConfig, rounds: usize, hold_ms: u64) -> u64 {
    let (g, s) = (&c.g5, &c.sim);
    let case = |k: &Case| {
        let press = if k.long {
            hold_ms.saturating_add(s.hold_margin_ms)
        } else {
            s.click_ms
        };
        let hang = if k.hang { g.hang_ms } else { 0 };
        total(&[s.step_ms, s.step_ms, press, g.settle_ms, hang])
    };
    let round = total(&cases(g, 0).iter().map(case).collect::<Vec<_>>());
    u64::try_from(rounds).map_or(u64::MAX, |n| n.saturating_mul(round))
}

/// Refuses a G12 capture that would end before its presses, so their frames would go unseen, or a G12 run past the
/// guard's limit.
fn check_g12(c: &GatesConfig, hold_ms: u64) -> Result<(), String> {
    let (need, secs) = (g12_ms(c, hold_ms), c.g12.presentmon_secs);
    if ms_of(secs) < need {
        return Err(format!(
            "g12.presentmon_secs ({secs} s) ends before G12's presses end, {need} ms in: raise it"
        ));
    }
    held_fits(c, "g12", need)
}

/// Refuses a run of `gate` that holds the mouse `ms` and, with `[fit] guard_headroom_ms`, passes the guard's limit, as
/// the guard would stop it midway; with the guard off, nothing is held.
fn held_fits(c: &GatesConfig, gate: &str, ms: u64) -> Result<(), String> {
    let (need, limit) = (
        ms.saturating_add(c.fit.guard_headroom_ms),
        c.mouse_guard.limit_ms,
    );
    if c.mouse_guard.on && need > limit {
        return Err(format!(
            "{gate} holds the mouse at least {need} ms with fit.guard_headroom_ms, past mouse_guard.limit_ms \
             {limit}: count fewer clicks or raise the limit"
        ));
    }
    Ok(())
}

/// Refuses `clicks` clicks of `gate`, G2 or G4, that would pass the guard's limit, such as a `--clicks` count.
pub fn clicks_fit(c: &GatesConfig, gate: &str, clicks: usize) -> Result<(), String> {
    held_fits(c, gate, clicks_ms(&c.g2, clicks))
}

/// Refuses a guarded run of the settings that would pass the guard's limit, and a G8 or G12 capture that would end
/// before its steps; G5's and G12's holds are the mouse module's defaults.
pub fn check(c: &GatesConfig) -> Result<(), String> {
    let hold_ms = MouseSettings::from_defaults()
        .map(|m| u64::from(m.hold_ms))
        .map_err(|e| format!("the mouse defaults: {e}"))?;
    check_capture(c)?;
    check_g12(c, hold_ms)?;
    clicks_fit(c, "g2", c.g2.clicks)?;
    clicks_fit(c, "g4", c.g4.clicks)?;
    held_fits(c, "g5", rounds_ms(c, c.g5.rounds, hold_ms))?;
    held_fits(c, "g8 hook", rounds_ms(c, c.g8_hook.rounds, hold_ms))?;
    held_fits(c, "g8", g8_ms(c, crate::board::ids()?.len()))
}

#[cfg(test)]
mod tests;
