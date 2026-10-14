//! `kx-gates-p4.toml`: the product's test build (`[gate_input]`), which P4's gates start and drive, G5, G12 and G8's
//! hook part on it, and the owner's hand try.
#![cfg(windows)]

use std::ffi::OsStr;
use std::path::Path;

use serde::Deserialize;

/// The product's test build: built with its gate-input feature and started with `flag`, its engine hears only
/// kx-gates' simulated input.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GateInput {
    /// The test build's program; a relative path starts at the settings folder.
    pub program: String,
    /// The flag that makes its engine act on kx-gates' simulated input only; kx-gates' code never holds this text.
    pub flag: String,
    /// The warning it logs at start with the flag; without it, its engine might act on the hand.
    pub warn_line: String,
    /// Longest wait for its keyboard and its warning line after its start, in ms.
    pub start_ms: u64,
    /// Longest wait for a mode or latch key to read on or off after its click, in ms.
    pub lit_ms: u64,
}

impl GateInput {
    /// Refuses an empty text or a zero wait.
    pub fn check(&self) -> Result<(), String> {
        let texts = [
            ("program", &self.program),
            ("flag", &self.flag),
            ("warn_line", &self.warn_line),
        ];
        if let Some((key, _)) = texts.iter().find(|(_, text)| text.is_empty()) {
            return Err(format!("gate_input.{key} must not be empty"));
        }
        if self.start_ms == 0 || self.lit_ms == 0 {
            return Err("gate_input.start_ms and lit_ms must be above 0".to_string());
        }
        Ok(())
    }
}

/// `[g5]`: G5's cases on target-window under the test build's hold; times in ms, distances in physical px.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct G5 {
    /// A drag while pressed.
    pub drag_px: i32,
    /// How far inside and outside the still radius the wiggle and the short move go.
    pub still_margin_px: i32,
    /// How many times the cases run.
    pub rounds: usize,
    /// Wait for the target log after each case.
    pub settle_ms: u64,
    /// How far a logged point may be from where we pressed.
    pub point_slack_px: i32,
    /// The hook must hand the engine at least this many events per case played.
    pub min_events_per_case: u64,
    /// How long the hung case makes target-window sleep.
    pub hang_ms: u64,
    /// How long before the hold time the hung case asks target-window to sleep.
    pub hang_lead_ms: u64,
}

impl G5 {
    /// Refuses no round, no event per case, a still margin within the point slack, as the wiggle and the short move
    /// could then land on the radius, and a lead at or past the hang, as target-window would wake before the right-click.
    pub fn check(&self) -> Result<(), String> {
        if self.rounds == 0 || self.min_events_per_case == 0 {
            return Err("g5.rounds and min_events_per_case must be above 0".to_string());
        }
        if self.still_margin_px <= self.point_slack_px {
            return Err("g5.still_margin_px must be above point_slack_px".to_string());
        }
        if self.hang_lead_ms >= self.hang_ms {
            return Err(format!(
                "g5.hang_lead_ms ({} ms) must stay below hang_ms ({} ms), or target-window wakes before the \
                 right-click",
                self.hang_lead_ms, self.hang_ms
            ));
        }
        Ok(())
    }
}

/// `[hand]`: the owner's hand try on the running keyboard.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Hand {
    /// How long the hand try runs, in seconds, unless `--secs` says otherwise.
    pub try_secs: u64,
}

/// `[g12]`: G12's clicks under the test build's pointer overlay; times in ms, distances in physical px.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct G12 {
    /// Click grid columns and rows over target-window's text box; each point is clicked once.
    pub grid: [u32; 2],
    /// The grid stays this far inside the text box's edges.
    pub inset_px: i32,
    /// Each press lasts the test build's `ring_after_ms` plus this, so its ring shows.
    pub ring_ms: u64,
    /// A press plus this must stay within the test build's `hold_ms`, so it stays a click.
    pub spare_ms: u64,
    /// Wait after each release.
    pub settle_ms: u64,
    /// A logged press may be this far from where we clicked.
    pub slack_px: i32,
    /// Least frames a second the ring must draw.
    pub min_fps: f64,
    /// 99 in 100 frame gaps must be this long or shorter.
    pub p99_ms: f64,
    /// G12's `PresentMon` CSV: a plain file name in the out folder.
    pub presentmon: String,
    /// How long that capture records, in seconds.
    pub presentmon_secs: u64,
}

impl G12 {
    /// Refuses an empty grid, a negative inset or slack, limits not above 0, no ring time, no capture time, and a CSV
    /// that is not a plain file name, as it must stay in the out folder.
    pub fn check(&self) -> Result<(), String> {
        let grid = self.grid.iter().all(|&n| n > 0);
        let limits = self.min_fps > 0.0 && self.p99_ms > 0.0;
        let room = self.inset_px >= 0 && self.slack_px >= 0;
        if !grid || !limits || !room {
            return Err(
                "g12: grid, min_fps and p99_ms must be above 0, inset_px and slack_px 0 or more"
                    .into(),
            );
        }
        if self.ring_ms == 0 || self.presentmon_secs == 0 {
            return Err("g12.ring_ms and g12.presentmon_secs must be above 0".into());
        }
        let plain = Path::new(&self.presentmon).file_name() == Some(OsStr::new(&self.presentmon));
        if !plain {
            return Err("g12.presentmon must be a plain file name, in the out folder".into());
        }
        Ok(())
    }
}

/// `[g8_hook]`: G8's hook part, G5's cases on the test build judged by the hook's p99.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct G8Hook {
    /// The hook callback's 99th percentile may take this long, in ms (spec §10).
    pub p99_ms: f64,
    /// How many times G5's cases run.
    pub rounds: usize,
    /// Fewer events than this over the run leave the p99 not measured.
    pub min_events: u64,
}

impl G8Hook {
    /// Refuses a budget not above 0, no round and no event, as the p99 would then rest on nothing.
    pub fn check(&self) -> Result<(), String> {
        if self.p99_ms.is_nan() || self.p99_ms <= 0.0 {
            return Err("g8_hook.p99_ms must be above 0".into());
        }
        if self.rounds == 0 || self.min_events == 0 {
            return Err("g8_hook.rounds and min_events must be above 0".into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
