//! `kx-gates-p12.toml`: G24 (`[g24]`) on the product, with Snip's gate ids and window titles.
#![cfg(windows)]

mod names;

use kx_module_api::MS_PER_S;
use kx_module_api::ports::{BASE_DPI, Speed, SpeedAsk, physical};
use serde::Deserialize;

use crate::config::GatesConfig;
use crate::fit::{held_fits, times, total};
use crate::p7cfg::runs::end_ms;
use crate::p8cfg::wait;
pub use names::{SnipClock, SnipIds, SnipTitles};

/// The copies G24 starts: its keyboard as a bubble, then hidden.
pub const COPIES: usize = 2;
/// The snips the copies take: the bubble's full screen, region, editor, warm-up video, window, video and low check,
/// then the hidden one's full screen and sped video.
const SNIPS: usize = 9;
/// The most clicks a snip takes: Snip, a way, two corners, a strip key, the editor's or question's keys and a reset.
const SNIP_CLICKS: usize = 8;
/// The most waits a snip takes, each up to `wait_ms`: the freeze, the bar, the strip, the clipboard, the end and more.
const SNIP_WAITS: usize = 8;
/// The memory reads after a pause: idle, then after the snips.
const IDLES: usize = 2;
/// The recordings, each of which may wait `wait_ms` more for the record bar's clock: warm-up, target-window, long.
const RECORDINGS: usize = 3;
/// The order checks of the low part, each up to `order_ms`: the Save dialog's, then ours once it closed.
const ORDERS: usize = 2;
/// The kept videos' Closes, each up to `close_tries` clicks with `close_ms` after: the short and the sped.
const KEPT_CLOSES: usize = 2;
/// The trap part's waits, each up to `wait_ms`: 6 in our keyboard's Esc step, 4 in another program's, 7 in Esc twice
/// and 16 in Save.
const TRAP_WAITS: usize = 33;
/// Its order checks in the same steps, each up to `order_ms`: 2, 1, 3 and 5.
const TRAP_ORDERS: usize = 11;
/// Its clicks and Esc presses, each followed by `settle_ms`: 2, 1, 5 and 7.
const TRAP_SETTLES: usize = 15;

/// `[g24]`: the run's text and region, scale, waits, limits, settings names and ids.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct G24 {
    /// Typed into target-window, so the snipped region holds a known pattern.
    pub text: String,
    /// The region from the text box's top-left: x, y, width and height in logical px.
    pub region_px: [f32; 4],
    /// Longest wait for a Snip window, a part or the clipboard to show, in ms.
    pub wait_ms: u64,
    /// Window class of the taskbar, which the frozen screen must cover (adapter table).
    pub taskbar_class: String,
    /// The screen DPI the exact checks are judged at.
    pub dpi: u32,
    /// The editor's part from the text box's top-left in logical px, its size between logical pixels at `dpi`.
    pub edit_px: [f32; 4],
    /// Wait after a copy's keyboard shows, in ms.
    pub ready_ms: u64,
    /// Pause after each click, in ms.
    pub settle_ms: u64,
    /// Pause after a snip ends before memory is read, in ms.
    pub idle_ms: u64,
    /// The most time from the Snip click until the frozen screen covers, in ms.
    pub cover_ms: u64,
    /// The most passes a reset makes to close what a failed part left open, each up to `wait_ms`.
    pub reset_passes: usize,
    /// The most share of a window's picture that may differ: its caret.
    pub frame_share: f64,
    /// The most share of the keyboard's place a full-screen picture may hold as the keyboard showed it, and of the
    /// bubble's middle as the bubble draws itself.
    pub same_most: f64,
    /// The most memory may grow from idle, in MB.
    pub back_mb: f64,
    /// The recording of target-window, in seconds on the record bar's clock.
    pub record_s: u64,
    /// The warm-up recording, stopped and closed unsaved before idle memory is read, in seconds on the clock.
    pub warm_s: u64,
    /// The largest picture Windows' clipboard history keeps, in decimal MB; a Copy up to it must leave history.
    pub history_item_mb: u32,
    /// How long after a Copy's read back Windows may stamp its picture in history, in ms; at most `wait_ms`.
    pub history_pad_ms: u64,
    /// A video length's slack either way, in ms.
    pub slack_ms: u64,
    /// The long full-screen recording, in seconds.
    pub long_s: u64,
    /// The speed the long recording is copied at, picked on the strip.
    pub speed: u8,
    /// Longest wait for the sped copy, in ms.
    pub speed_wait_ms: u64,
    /// Longest wait for the copy's workers to end before its data folder goes, in ms.
    pub gone_ms: u64,
    /// The most clicks on a kept video's Close; the strip ignores one that lands before its Copy ended.
    pub close_tries: usize,
    /// Longest wait after each such click for the strip to go, in ms.
    pub close_ms: u64,
    /// Longest wait for our windows to lie over every frozen screen once they show, in ms (ADR-0059).
    pub order_ms: u64,
    /// The Save dialog's window class.
    pub dialog_class: String,
    /// The Save dialog's Cancel button, by UI Automation id.
    pub cancel_id: String,
    /// Snip's gate ids.
    pub ids: SnipIds,
    /// kx-ui's text keys of the record bar's clock.
    pub clock: SnipClock,
    /// kx-ui's window title keys.
    pub titles: SnipTitles,
}

/// True when `px` logical pixels at `dpi` land between whole logical pixels once made physical.
fn between(px: f32, dpi: u32) -> bool {
    let side = u32::try_from(physical(f64::from(px), dpi));
    dpi > 0 && side.is_ok_and(|s| s > 0 && !(s * BASE_DPI).is_multiple_of(dpi))
}

impl G24 {
    /// Refuses a value that could judge nothing: an empty text or region, no scale, an editor's part on whole logical
    /// pixels, a share outside its range, no memory limit, or a speed the strip lacks or plays as recorded.
    fn check_values(&self) -> Result<(), String> {
        let [.., w, h] = self.region_px;
        let [.., ew, eh] = self.edit_px;
        let rules = [
            (self.text.is_empty(), "g24.text must not be empty"),
            (
                w <= 0.0 || h <= 0.0,
                "g24.region_px needs a width and a height",
            ),
            (self.dpi == 0, "g24.dpi must be above 0"),
            (
                !(between(ew, self.dpi) && between(eh, self.dpi)),
                "g24.edit_px needs a width and a height between whole logical pixels at g24.dpi",
            ),
            (
                !(0.0..1.0).contains(&self.frame_share),
                "g24.frame_share must be from 0 to below 1",
            ),
            (
                !(self.same_most > 0.0 && self.same_most < 1.0),
                "g24.same_most must be between 0 and 1",
            ),
            (self.back_mb <= 0.0, "g24.back_mb must be above 0"),
            (
                Speed::new(self.speed).is_none_or(|s| s == Speed::PLAIN),
                "g24.speed must be a speed the strip offers, not as recorded",
            ),
        ];
        if let Some((_, why)) = rules.iter().find(|(bad, _)| *bad) {
            return Err((*why).to_string());
        }
        self.check_runs()
    }

    /// Refuses no recording, warm-up, reset pass, Close try or history size, and a history pad of 0 ms or past a wait.
    fn check_runs(&self) -> Result<(), String> {
        let rules = [
            (self.record_s == 0, "g24.record_s must be above 0"),
            (self.warm_s == 0, "g24.warm_s must be above 0"),
            (self.reset_passes == 0, "g24.reset_passes must be above 0"),
            (self.close_tries == 0, "g24.close_tries must be above 0"),
            (
                self.history_item_mb == 0,
                "g24.history_item_mb must be above 0",
            ),
            (
                !(1..=self.wait_ms).contains(&self.history_pad_ms),
                "g24.history_pad_ms must be from 1 to g24.wait_ms",
            ),
        ];
        if let Some((_, why)) = rules.iter().find(|(bad, _)| *bad) {
            return Err((*why).to_string());
        }
        self.check_lengths()
    }

    /// Refuses a slack that swallows the recording, or the long one at its speed.
    fn check_lengths(&self) -> Result<(), String> {
        let ms = u64::from(MS_PER_S);
        if self.slack_ms >= self.record_s.saturating_mul(ms) {
            return Err("g24.slack_ms must be shorter than g24.record_s".into());
        }
        let sped = self.long_s.saturating_mul(ms) / u64::from(self.speed.max(1));
        if sped <= self.slack_ms {
            return Err("g24.long_s at g24.speed must be longer than g24.slack_ms".into());
        }
        Ok(())
    }

    /// Refuses an empty or shared id, an empty window class or Cancel id, and a clock text or title kx-ui lacks.
    fn check_names(&self) -> Result<(), String> {
        self.ids.check()?;
        self.clock.check()?;
        let names = [
            ("dialog_class", &self.dialog_class),
            ("cancel_id", &self.cancel_id),
            ("taskbar_class", &self.taskbar_class),
        ];
        if let Some((key, _)) = names.iter().find(|(_, n)| n.is_empty()) {
            return Err(format!("g24.{key} must not be empty"));
        }
        self.titles.check()
    }

    /// The strip's Speed clicks from as recorded to `speed`: its place in `SpeedAsk::SPEEDS`, which each click steps.
    pub fn speed_steps(&self) -> usize {
        let at = SpeedAsk::SPEEDS.iter().position(|&t| t == self.speed);
        at.unwrap_or_default()
    }

    /// Refuses a wait outside 1 ms to `most_ms`, the mouse guard's limit.
    fn check_waits(&self, most_ms: u64) -> Result<(), String> {
        let waits = [
            ("g24.wait_ms", self.wait_ms),
            ("g24.ready_ms", self.ready_ms),
            ("g24.settle_ms", self.settle_ms),
            ("g24.idle_ms", self.idle_ms),
            ("g24.cover_ms", self.cover_ms),
            ("g24.speed_wait_ms", self.speed_wait_ms),
            ("g24.gone_ms", self.gone_ms),
            ("g24.order_ms", self.order_ms),
            ("g24.close_ms", self.close_ms),
        ];
        waits
            .iter()
            .try_for_each(|(key, ms)| wait(key, *ms, most_ms))
    }
}

/// The longest G24 on the product may hold the mouse: target-window, both copies, every snip's clicks, waits and
/// reset, the memory pauses, the three recordings with their clock waits, the order checks, the kept videos' Close
/// tries, the sped copy's wait and the workers' end.
pub fn g24_ms(c: &GatesConfig) -> u64 {
    let (g, target, copy) = (&c.g24, target_ms(c), copy_ms(c));
    let snip = total(&[
        times(SNIP_CLICKS, g.settle_ms),
        times(SNIP_WAITS, g.wait_ms),
        times(g.reset_passes, g.wait_ms),
        g.cover_ms,
    ]);
    let ms = u64::from(MS_PER_S);
    total(&[
        target,
        times(COPIES, copy),
        times(SNIPS, snip),
        times(IDLES, g.idle_ms),
        g.warm_s.saturating_mul(ms),
        g.record_s.saturating_mul(ms),
        g.long_s.saturating_mul(ms),
        times(RECORDINGS, g.wait_ms),
        times(ORDERS, g.order_ms),
        times(
            KEPT_CLOSES.saturating_mul(g.close_tries),
            g.close_ms.saturating_add(g.settle_ms),
        ),
        g.speed_wait_ms,
        g.gone_ms,
    ])
}

/// Target-window's pause before typing.
fn target_ms(c: &GatesConfig) -> u64 {
    let target = c.apps.get(crate::apps::AppKind::Target.name());
    target.map_or(0, |a| a.ready_ms)
}

/// One copy: its start, the wait once its keyboard shows, and its end.
fn copy_ms(c: &GatesConfig) -> u64 {
    total(&[c.g8.start_wait_ms, c.g24.ready_ms, end_ms(c)])
}

/// The longest the trap part (`g24 trap`) may hold the mouse: target-window, one copy, its steps' waits, order checks
/// and pauses, and the workers' end.
pub fn trap_ms(c: &GatesConfig) -> u64 {
    let g = &c.g24;
    total(&[
        target_ms(c),
        copy_ms(c),
        times(TRAP_WAITS, g.wait_ms),
        times(TRAP_ORDERS, g.order_ms),
        times(TRAP_SETTLES, g.settle_ms),
        g.gone_ms,
    ])
}

/// Refuses P12 settings that could judge nothing, name no button or window, or wait past the mouse guard.
pub fn check(c: &GatesConfig) -> Result<(), String> {
    let g = &c.g24;
    g.check_values()?;
    g.check_names()?;
    g.check_waits(c.mouse_guard.limit_ms)?;
    held_fits(c, "g24: the trap part", trap_ms(c))?;
    held_fits(c, "g24: G24 on the product", g24_ms(c))
}

#[cfg(test)]
mod tests;
