//! Settings of gate G8, the budgets of spec §10, from `[g8]` of `kx-gates-p3.toml`; `check` refuses a run that could pass with nothing
//! measured.
#![cfg(windows)]

use serde::Deserialize;

use crate::board;

/// G8 settings; times in ms unless named otherwise.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct G8 {
    /// The product copy G8 starts, through the shell.
    pub program: String,
    /// Cold start to a shown keyboard, on this PC.
    pub start_ms: f64,
    /// Cold start to a shown keyboard, in weak mode.
    pub weak_start_ms: f64,
    /// Longest wait for the keyboard to show and for the log's guard line.
    pub start_wait_ms: u64,
    /// How often the start and each resize are looked at.
    pub look_ms: u64,
    /// Text of the log line that says the app started.
    pub log_start: String,
    /// Text of the log line that says the window guard is done.
    pub log_guarded: String,
    /// Wait before the idle measure.
    pub idle_settle_ms: u64,
    /// How long the idle measure lasts.
    pub idle_ms: u64,
    /// Wait from the shown keyboard to the memory sample, a fixed warm-up so runs compare (ADR-0021).
    pub warm_ms: u64,
    /// Idle private working set, in MB (10⁶ bytes, as spec §10 counts).
    pub memory_mb: f64,
    /// Idle private commit, in MB; a trim cannot lower it, so it keeps the working set honest.
    pub commit_mb: f64,
    /// Idle CPU, in percent of one core, in both modes.
    pub idle_core_pct: f64,
    /// The program folder, in MB of 10⁶ bytes: the stage's `max_mb`.
    pub size_mb: f64,
    /// The key pressed for the highlight check, by board id.
    pub press_key: String,
    /// How many presses are timed.
    pub presses: usize,
    /// Screen frames the median press may take to light its key (ADR-0025).
    pub highlight_frames: f64,
    /// Screen frames the slowest press may take to light its key; at least `highlight_frames`.
    pub highlight_worst_frames: f64,
    /// Wait with the pointer on the key before a press, so the hover has settled.
    pub hover_settle_ms: u64,
    /// Longest wait for a press to light its key.
    pub press_wait_ms: u64,
    /// A pressed key's rise after its release; frames lost from the press to its end count as `dropped_frames`.
    pub rise_ms: u64,
    /// Wait after the last press before the resizes, so its rise has ended; at least `rise_ms`.
    pub rise_wait_ms: u64,
    /// The top bar's grow button, by UI Automation id.
    pub grow: String,
    /// The top bar's shrink button, by UI Automation id.
    pub shrink: String,
    /// The window has its new size once its box stays the same this long.
    pub resize_settle_ms: u64,
    /// Longest wait for a resize.
    pub resize_wait_ms: u64,
    /// Keyboard widths this many px apart count as one size, for DPI rounding; G8 and G9 size back past it.
    pub resize_slack_px: i32,
    /// G2 clicks on the started copy, for the click-to-character time.
    pub clicks: usize,
    /// Click to character, 99th percentile.
    pub char_ms: f64,
    /// Handlers on the timed bus event.
    pub subscribers: usize,
    /// Publishes timed.
    pub publishes: usize,
    /// A publish's 99th percentile may take this share of `char_ms`.
    pub bus_share: f64,
    /// `PresentMon`'s CSV, in the out folder.
    pub presentmon: String,
    /// How long `PresentMon` records, in seconds; long enough to cover G8's run up to the churn.
    pub presentmon_secs: u64,
    /// The `PresentMon` program G8 starts as administrator; empty when G8 only says how to record.
    pub presentmon_exe: String,
    /// Wait after starting `PresentMon`, so its start-up does not slow the cold start.
    pub presentmon_wait_ms: u64,
    /// Longest wait past the recording time for `PresentMon` to stop and write its last rows, whether G8 or the owner
    /// started it.
    pub presentmon_stop_ms: u64,
    /// Quit tries on a copy G8 started but could not open or cap, whose command window may not exist yet, and the
    /// wait between them.
    pub quit_tries: u32,
    /// The wait between those quit tries, in ms.
    pub quit_try_ms: u64,
    /// Unmeasured churn rounds first, so each pop-up's first show is not counted as growth.
    pub churn_warm_rounds: usize,
    /// Measured churn rounds of tips, the language list, the bubble and a theme cycle.
    pub churn_rounds: usize,
    /// Most growth of the working set and of the commit over those rounds, in MB.
    pub churn_growth_mb: f64,
    /// Rounds at each end whose median readings the growth compares; at most half of `churn_rounds`.
    pub churn_median_rounds: usize,
    /// Keys hovered each round, by board id; each has an icon, so its tip shows.
    pub churn_tips: Vec<String>,
    /// Rest on each key past the running copy's `[ui] tip_ms`, so its tip has shown.
    pub churn_tip_ms: u64,
    /// The top bar's theme button, by UI Automation id.
    pub churn_theme: String,
    /// Wait after each list, bubble or theme press, so the pop-ups have shown or gone.
    pub churn_settle_ms: u64,
}

impl G8 {
    /// Refuses a zero count or warm-up, a bad budget, a negative slack, a pressed key the board does not have, or a
    /// churn that churns nothing.
    pub fn check(&self) -> Result<(), String> {
        self.check_budgets()?;
        if [self.presses, self.clicks, self.subscribers, self.publishes].contains(&0) {
            return Err(
                "g8 presses, clicks, subscribers and publishes must be above 0".to_string(),
            );
        }
        if self.presentmon_secs == 0 || self.warm_ms == 0 {
            // PresentMon runs hidden as administrator, so it must stop by itself; memory is read after a warm-up.
            return Err("g8.presentmon_secs and g8.warm_ms must be above 0".to_string());
        }
        if self.rise_ms == 0 || self.rise_wait_ms < self.rise_ms {
            return Err(
                "g8.rise_ms must be above 0 and g8.rise_wait_ms at least rise_ms".to_string(),
            );
        }
        if self.quit_tries == 0 || self.resize_slack_px < 0 {
            return Err(
                "g8.quit_tries must be above 0 and g8.resize_slack_px not below 0".to_string(),
            );
        }
        if !board::ids()?.contains(&self.press_key) {
            return Err(format!(
                "g8.press_key: {} is no key of the product's board",
                self.press_key
            ));
        }
        crate::g8churn::check(self)
    }

    /// Refuses a budget that is not above 0, or a slowest light allowed fewer frames than the median.
    fn check_budgets(&self) -> Result<(), String> {
        let budgets = [
            self.start_ms,
            self.weak_start_ms,
            self.memory_mb,
            self.commit_mb,
            self.idle_core_pct,
            self.size_mb,
            self.highlight_frames,
            self.highlight_worst_frames,
            self.char_ms,
            self.bus_share,
        ];
        if budgets.iter().any(|b| b.is_nan() || *b <= 0.0) {
            return Err("every g8 budget must be above 0".to_string());
        }
        if self.highlight_worst_frames < self.highlight_frames {
            return Err("g8.highlight_worst_frames must be at least highlight_frames".to_string());
        }
        Ok(())
    }

    /// The bus publish budget, in µs: `bus_share` of the click-to-character budget.
    pub fn bus_p99_us(&self) -> f64 {
        self.char_ms * self.bus_share * crate::stats::US_PER_MS
    }
}

#[cfg(test)]
mod tests;
