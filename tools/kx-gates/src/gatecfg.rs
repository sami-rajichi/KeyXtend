//! Settings of the first gates: G1 typing, G2 and G4 clicks, G3 top band, the hold engine run and G5.
#![cfg(windows)]

use serde::Deserialize;

/// G1 typing settings.
#[derive(Debug, Clone, Deserialize)]
pub struct G1 {
    /// Characters typed per run.
    pub count: usize,
    /// Characters per `SendInput` call.
    pub chunk_chars: usize,
    /// Pause after each chunk, in ms.
    pub chunk_pause_ms: u64,
    /// Title of the Chrome test page, used to find its window.
    pub page_title: String,
    /// Characters shown on each side of the first difference.
    pub context_chars: usize,
    /// Longest clipboard text we read, in characters; more is an error.
    pub max_read_chars: usize,
}

/// G2 click settings.
#[derive(Debug, Clone, Deserialize)]
pub struct G2 {
    /// Clicks per run.
    pub clicks: usize,
    /// Pause after each click before checking focus and caret, in ms.
    pub click_gap_ms: u64,
    /// A character belongs to a click if it arrives within this time, in ms.
    pub match_window_ms: i64,
    /// Wait for the last characters to reach the log, in ms.
    pub drain_ms: u64,
    /// Failures listed in the report.
    pub failures_shown: usize,
    /// Failures that stop the run.
    pub max_failures: usize,
}

/// G3 top-band settings.
#[derive(Debug, Clone, Deserialize)]
pub struct G3 {
    /// Wait after opening a surface before probing, in ms.
    pub open_wait_ms: u64,
    /// Scan codes of the probed keys.
    pub probe_codes: Vec<u32>,
    /// System surfaces opened over the face.
    pub surfaces: Vec<Surface>,
}

/// G4 admin-window settings; click pauses and failure limits come from G2.
#[derive(Debug, Clone, Deserialize)]
pub struct G4 {
    /// Clicks per run.
    pub clicks: usize,
}

/// How kx-gates runs the hold engine; the hold itself is `spike.toml [hold]` (spec §5.1).
#[derive(Debug, Clone, Deserialize)]
pub struct AssistCfg {
    /// Longest wait for the hook thread to start or answer, in ms.
    pub reply_ms: u64,
    /// How long the owner's hand try runs, in seconds.
    pub try_secs: u64,
    /// How often the Grab hand try turns Grab back on after a drop, in ms.
    pub rearm_ms: u64,
}

/// G5 hold-engine settings; times in ms, distances in px.
#[derive(Debug, Clone, Deserialize)]
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
    /// Budget for the hook callback's 99th percentile.
    pub hook_p99_ms: f64,
}

/// A system surface to open over the face.
#[derive(Debug, Clone, Deserialize)]
pub struct Surface {
    /// Name in the report.
    pub name: String,
    /// Virtual keys that open it.
    pub open: Vec<u16>,
    /// Virtual keys that close it.
    #[serde(default)]
    pub close_keys: Vec<u16>,
    /// Class of a new window to close with `WM_CLOSE`.
    #[serde(default)]
    pub close_class: Option<String>,
}
