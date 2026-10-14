//! `kx-gates-p4.toml`: the product's test build (`[gate_input]`), which P4's gates start and drive, G5 on it, and
//! the owner's hand try.
#![cfg(windows)]

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
    /// The line its mouse module logs with the hook's numbers at each hook stop.
    pub hook_line: String,
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
            ("hook_line", &self.hook_line),
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

#[cfg(test)]
mod tests;
