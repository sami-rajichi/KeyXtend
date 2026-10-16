//! `kx-gates-p5.toml`: G6 on the test build (`[g6]`), the scroll pad's parts (`[pad]`), G9's outline steps
//! (`[outline]`), G8's scroll steps (`[g8_scroll]`) and what G8 logs of the machine (`[host]`).
#![cfg(windows)]

use kx_module_api::ports::Dir;
use serde::Deserialize;

use crate::aim::Reach;
use crate::config::GatesConfig;

/// `[g6]`: G6's cases on the test build; times in ms, distances in physical px.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct G6 {
    /// What the pictures leave out at the document's sides and above the keyboard.
    pub inset_px: i32,
    /// A picture counts as changed when more than this share of its pixels changed.
    pub min_changed: f64,
    /// How long each speed scrolls while its percent is read.
    pub phase_ms: u64,
    /// How often the percent is read during a speed.
    pub sample_ms: u64,
    /// How far the fast ÷ slow ratio may be off the test build's, as a share.
    pub ratio_slack: f64,
    /// How long each other direction scrolls before it is judged.
    pub move_ms: u64,
    /// How far apart the two looks that show a stop are.
    pub still_ms: u64,
    /// How long the end case scrolls up from the end first.
    pub back_ms: u64,
    /// Longest wait for a scroll to reach the end.
    pub end_wait_ms: u64,
    /// Longest wait for UI Automation's percent to show G6's own step.
    pub lag_wait_ms: u64,
    /// How long target-window sleeps in the hang record.
    pub hang_ms: u64,
    /// How long after the tap it is asked to sleep.
    pub hang_lead_ms: u64,
    /// The span its wheel messages are counted in, before the hang and after it wakes.
    pub burst_ms: u64,
}

/// The fewest percent reads a slope rests on.
pub const SLOPE_READS: u64 = 2;

/// True when `share` lies strictly between 0 and 1.
fn a_share(share: f64) -> bool {
    share > 0.0 && share < 1.0
}

impl G6 {
    /// Refuses a share outside 0 to 1, an inset or waits not above 0, a speed read too short for a slope, and a count
    /// before the hang longer than the wait before it.
    pub fn check(&self) -> Result<(), String> {
        if !a_share(self.min_changed) || !a_share(self.ratio_slack) {
            return Err("g6.min_changed and g6.ratio_slack must lie between 0 and 1".to_string());
        }
        if self.inset_px <= 0 {
            return Err("g6.inset_px must be above 0".to_string());
        }
        let waits = [
            self.sample_ms,
            self.move_ms,
            self.still_ms,
            self.back_ms,
            self.end_wait_ms,
            self.lag_wait_ms,
            self.hang_ms,
            self.burst_ms,
        ];
        if waits.contains(&0) {
            return Err("g6's waits must be above 0".to_string());
        }
        if self.phase_ms < self.sample_ms.saturating_mul(SLOPE_READS) {
            return Err(format!(
                "g6.phase_ms must hold at least {SLOPE_READS} reads of sample_ms, or no slope is measured"
            ));
        }
        if self.burst_ms > self.hang_lead_ms {
            return Err(
                "g6.burst_ms must fit in hang_lead_ms, the count before the hang".to_string(),
            );
        }
        Ok(())
    }
}

/// `[pad]`: the scroll pad's parts by UI Automation id, and the wait after each tap or press.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pad {
    /// What every part's id starts with, before the separator.
    pub group: String,
    /// Stop's icons.toml `[dpad]` id.
    pub stop: String,
    /// Wait after each tap or press before the look, in ms.
    pub settle_ms: u64,
    /// How far under the keyboard's top edge the scrolled window must reach, in physical px.
    pub under_px: i32,
    /// How far above the keyboard's top the glide onto the pad starts, on that window, in physical px.
    pub lead_px: i32,
}

/// What kx-ui puts between a part's group and its id (`KbWindow.qml` `uid`).
pub const SEP: &str = ".";

impl Pad {
    /// The UI Automation id of the arrow towards `dir`.
    pub fn arrow(&self, dir: Dir) -> String {
        let name = serde_json::to_value(dir).ok();
        let name = name.as_ref().and_then(|v| v.as_str()).unwrap_or_default();
        self.part(name)
    }

    /// How far the scrolled window must reach around the keyboard's top.
    pub fn reach(&self) -> Reach {
        Reach {
            under_px: self.under_px,
            lead_px: self.lead_px,
        }
    }

    /// The UI Automation id of Stop.
    pub fn stop_id(&self) -> String {
        self.part(&self.stop)
    }

    /// The UI Automation id of the part called `name`.
    fn part(&self, name: &str) -> String {
        format!("{}{SEP}{name}", self.group)
    }

    /// Refuses an empty or dotted group, a Stop id that is no icons.toml `[dpad]` id, no wait and no reach.
    pub fn check(&self) -> Result<(), String> {
        if self.under_px <= 0 || self.lead_px <= 0 {
            return Err("pad.under_px and pad.lead_px must be above 0".to_string());
        }
        if self.group.is_empty() || self.group.contains(SEP) {
            return Err(format!("pad.group must be a name without {SEP}"));
        }
        let themes = kx_theme::Themes::shipped().map_err(|e| format!("the shipped tokens: {e}"))?;
        let icons = serde_json::to_value(&themes.icons).map_err(|e| e.to_string())?;
        if icons["dpad"].get(&self.stop).is_none() {
            return Err(format!(
                "pad.stop: {} is no icons.toml [dpad] id",
                self.stop
            ));
        }
        if self.settle_ms == 0 {
            return Err("pad.settle_ms must be above 0".to_string());
        }
        Ok(())
    }
}

/// `[outline]`: G9's outline steps.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Outline {
    /// kx-ui's text that titles the outline's edge windows.
    pub title: String,
    /// The key the pointer rests on while it scrolls, off the pad, by board id.
    pub rest: String,
    /// How many edge windows an outline has.
    pub edges: usize,
}

impl Outline {
    /// Refuses a title kx-ui lacks in a language, a rest key that is no board key, and no edge.
    pub fn check(&self) -> Result<(), String> {
        crate::ftl::every(&self.title).map_err(|e| format!("outline.title: {e}"))?;
        if !crate::board::ids()?.contains(&self.rest) {
            return Err(format!(
                "outline.rest: {} is no key of the product's board",
                self.rest
            ));
        }
        if self.edges == 0 {
            return Err("outline.edges must be above 0".to_string());
        }
        Ok(())
    }
}

/// `[g8_scroll]`: G8's scroll steps.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct G8Scroll {
    /// How long each scroll runs before Stop, in ms.
    pub scroll_ms: u64,
}

/// `[host]`: what G8 logs of the machine beside its numbers.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Host {
    /// The virtual machine processes whose memory it logs, by program name.
    pub vm: Vec<String>,
}

impl Host {
    /// Refuses no process and a name that is not a plain program name.
    pub fn check(&self) -> Result<(), String> {
        let plain = |n: &String| !n.is_empty() && !n.contains(['\\', '/', ':']);
        if self.vm.is_empty() || !self.vm.iter().all(plain) {
            return Err("host.vm must name programs, without a folder".to_string());
        }
        Ok(())
    }
}

/// Refuses P5's settings that could pass with nothing checked, and a G6 hang longer than target-window allows.
pub fn check(c: &GatesConfig) -> Result<(), String> {
    c.g6.check()?;
    c.pad.check()?;
    c.outline.check()?;
    c.host.check()?;
    if c.g8_scroll.scroll_ms == 0 {
        return Err("g8_scroll.scroll_ms must be above 0".to_string());
    }
    let target = kx_target_window::config::load().map_err(|e| e.to_string())?;
    if c.g6.hang_ms > target.hang_max_ms {
        return Err(format!(
            "g6.hang_ms must stay within kx-target-window.toml's hang_max_ms ({} ms)",
            target.hang_max_ms
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests;
