//! Run lengths checked at load (owner, 2026-10-11): each run the mouse guard holds fits its limit, as each click now
//! holds the button for `[g2] press_ms`.
#![cfg(windows)]

use std::time::Duration;

use serde::Deserialize;

use crate::config::GatesConfig;
use crate::gatecfg::G2;

/// `[fit]`: what a run spends beyond the steps counted here; times in ms.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Fit {
    /// A guarded run's steps not counted here, such as starting apps, waits and clean-up.
    pub guard_headroom_ms: u64,
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

/// The least time G8 holds the mouse: until its `PresentMon` capture ends, then each churn round resting on each tip.
fn g8_ms(c: &GatesConfig) -> u64 {
    let g = &c.g8;
    let rests = (g.churn_warm_rounds + g.churn_rounds).saturating_mul(g.churn_tips.len());
    let churn = u64::try_from(rests).map_or(u64::MAX, |n| n.saturating_mul(g.churn_tip_ms));
    ms_of(g.presentmon_secs).saturating_add(churn)
}

/// Refuses a guarded run whose counted steps and headroom pass the guard's limit, as the guard would stop it midway.
pub fn check(c: &GatesConfig) -> Result<(), String> {
    if !c.mouse_guard.on {
        return Ok(());
    }
    let limit = c.mouse_guard.limit_ms;
    let runs = [
        ("g2", clicks_ms(&c.g2, c.g2.clicks)),
        ("g4", clicks_ms(&c.g2, c.g4.clicks)),
        ("g8", g8_ms(c)),
    ];
    for (gate, ms) in runs {
        let need = ms.saturating_add(c.fit.guard_headroom_ms);
        if need > limit {
            return Err(format!(
                "{gate} holds the mouse at least {need} ms with fit.guard_headroom_ms, past mouse_guard.limit_ms \
                 {limit}: count fewer clicks or raise the limit"
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::load;

    #[test]
    fn a_click_run_takes_each_hold_and_gap() {
        let g2 = load().expect("loads").g2;
        let each = g2.press_ms + g2.click_gap_ms;
        assert_eq!(clicks_ms(&g2, 1000), 1000 * each);
        assert_eq!(clicks_ms(&g2, 0), 0);
    }

    #[test]
    fn the_shipped_guarded_runs_fit_the_guards_limit() {
        let c = load().expect("loads");
        assert_eq!(check(&c), Ok(()));
        let g2 = clicks_ms(&c.g2, c.g2.clicks) + c.fit.guard_headroom_ms;
        assert!(g2 <= c.mouse_guard.limit_ms, "G2's {g2} ms");
    }

    /// The shipped settings with `change` made.
    fn with(change: impl FnOnce(&mut GatesConfig)) -> GatesConfig {
        let mut c = load().expect("loads");
        change(&mut c);
        c
    }

    #[test]
    fn a_click_run_or_g8_that_would_pass_the_guards_limit_is_refused() {
        let past = |c: &GatesConfig| {
            let each = c.g2.press_ms + c.g2.click_gap_ms;
            usize::try_from(c.mouse_guard.limit_ms / each).expect("fits")
        };
        let g2 = with(|c| c.g2.clicks = past(c));
        let e = check(&g2).expect_err("G2 runs too long");
        assert!(e.contains("g2"), "{e}");
        let g4 = with(|c| c.g4.clicks = past(c));
        assert!(check(&g4).expect_err("G4 too").contains("g4"));
        let g8 = with(|c| c.g8.presentmon_secs = c.mouse_guard.limit_ms / 1000);
        assert!(check(&g8).expect_err("G8 too").contains("g8"));
    }

    #[test]
    fn with_the_guard_off_nothing_is_cut_short() {
        let c = with(|c| {
            c.mouse_guard.on = false;
            c.g2.clicks = usize::MAX;
        });
        assert_eq!(check(&c), Ok(()));
    }
}
