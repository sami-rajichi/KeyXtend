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
    /// Screen frames a press may take to light its key.
    pub highlight_frames: f64,
    /// Wait with the pointer on the key before a press, so the hover has settled.
    pub hover_settle_ms: u64,
    /// Longest wait for a press to light its key.
    pub press_wait_ms: u64,
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
    /// How long `PresentMon` records, in seconds; long enough to cover a G8 run.
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
    /// Refuses a zero count or warm-up, a budget that is not above 0, a negative slack, a pressed key the board does not
    /// have, or a churn that churns nothing.
    pub fn check(&self) -> Result<(), String> {
        let budgets = [
            self.start_ms,
            self.weak_start_ms,
            self.memory_mb,
            self.commit_mb,
            self.idle_core_pct,
            self.size_mb,
            self.highlight_frames,
            self.char_ms,
            self.bus_share,
        ];
        if budgets.iter().any(|b| b.is_nan() || *b <= 0.0) {
            return Err("every g8 budget must be above 0".to_string());
        }
        if [self.presses, self.clicks, self.subscribers, self.publishes].contains(&0) {
            return Err(
                "g8 presses, clicks, subscribers and publishes must be above 0".to_string(),
            );
        }
        if self.presentmon_secs == 0 || self.warm_ms == 0 {
            // PresentMon runs hidden as administrator, so it must stop by itself; memory is read after a warm-up.
            return Err("g8.presentmon_secs and g8.warm_ms must be above 0".to_string());
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

    /// The bus publish budget, in µs: `bus_share` of the click-to-character budget.
    pub fn bus_p99_us(&self) -> f64 {
        self.char_ms * self.bus_share * crate::stats::US_PER_MS
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The product's code that writes the start line and the guard line G8 waits for.
    // Workspace paths from `tools/kx-gates/src`; a moved crate breaks the build here, never silently.
    const RUN_RS: &str = include_str!("../../../apps/keyxtend/src/run.rs");
    const FACE_RS: &str = include_str!("../../../crates/kx-ui/src/hub/face.rs");
    /// The workspace manifest, whose stage settings hold the download budget.
    const ROOT_TOML: &str = include_str!("../../../Cargo.toml");
    /// The spec, whose §10 sets the budgets G8 holds the product to.
    const SPEC: &str =
        include_str!("../../../docs/superpowers/specs/2026-09-26-accessible-keyboard-design.md");

    fn shipped() -> G8 {
        crate::config::load().expect("kx-gates.toml loads").g8
    }

    /// The number after each `≤ ` in spec §10's row for `metric`.
    fn spec_limits(metric: &str) -> Vec<f64> {
        let head = format!("| {metric} |");
        let row = SPEC.lines().find(|l| l.starts_with(&head));
        let row = row.unwrap_or_else(|| panic!("spec §10 has no {metric} row"));
        let numbers = row.split("≤ ").skip(1);
        numbers
            .filter_map(|n| n.split_whitespace().next()?.parse().ok())
            .collect()
    }

    #[test]
    fn the_folder_budget_is_the_stages_own_limit() {
        let root: toml::Table = ROOT_TOML.parse().expect("is TOML");
        let max_mb = root["workspace"]["metadata"]["stage"]["max_mb"].as_integer();
        let max_mb = i32::try_from(max_mb.expect("a whole MB count")).expect("fits");
        let size_mb = shipped().size_mb;
        assert!(
            (size_mb - f64::from(max_mb)).abs() < f64::EPSILON,
            "{size_mb} vs {max_mb}"
        );
    }

    #[test]
    fn the_shipped_budgets_are_spec_10s() {
        let g = shipped();
        assert_eq!(g.check(), Ok(()));
        let memory = spec_limits("Main process memory, idle");
        assert_eq!(
            memory,
            [g.memory_mb, g.commit_mb],
            "working set, then commit"
        );
        assert_eq!(spec_limits("Cold start to visible keyboard"), [g.start_ms]);
        assert_eq!(
            spec_limits("Key click to character in target app"),
            [g.char_ms]
        );
        assert_eq!(
            spec_limits("Download size, without voice models"),
            [g.size_mb]
        );
        assert!(g.start_ms < g.weak_start_ms, "weak mode may start slower");
        assert!(
            g.presentmon_exe.is_empty(),
            "each PC names its own PresentMon"
        );
    }

    #[test]
    fn the_log_lines_g8_waits_for_are_the_ones_the_product_writes() {
        let g = shipped();
        let quoted = |s: &str| format!("\"{s}\"");
        assert!(RUN_RS.contains(&quoted(&g.log_start)), "{}", g.log_start);
        assert!(
            FACE_RS.contains(&quoted(&g.log_guarded)),
            "{}",
            g.log_guarded
        );
    }

    #[test]
    fn the_bus_budget_is_a_share_of_click_to_character_in_microseconds() {
        let g = G8 {
            char_ms: 30.0,
            bus_share: 0.01,
            ..shipped()
        };
        assert!((g.bus_p99_us() - 300.0).abs() < 1e-9, "{}", g.bus_p99_us());
    }

    #[test]
    fn a_g8_table_without_a_commit_budget_is_refused() {
        let file: toml::Table = include_str!("../kx-gates-p3.toml")
            .parse()
            .expect("is TOML");
        let mut g8 = file["g8"].as_table().expect("a [g8] table").clone();
        let read = |t: toml::Table| toml::Value::Table(t).try_into::<G8>();
        assert!(read(g8.clone()).is_ok(), "the shipped table reads");
        g8.remove("commit_mb");
        let e = read(g8).expect_err("commit_mb is needed");
        assert!(e.to_string().contains("commit_mb"), "{e}");
    }

    #[test]
    fn a_run_that_could_pass_with_nothing_measured_is_refused() {
        let bad: [fn(&mut G8); 10] = [
            |g| g.memory_mb = 0.0,
            |g| g.commit_mb = f64::NAN,
            |g| g.warm_ms = 0,
            |g| g.bus_share = f64::NAN,
            |g| g.presses = 0,
            |g| g.publishes = 0,
            |g| g.quit_tries = 0,
            |g| g.resize_slack_px = -1,
            |g| g.press_key = "c-0xFFFF".into(),
            |g| g.presentmon_secs = 0,
        ];
        for f in bad {
            let mut g = shipped();
            f(&mut g);
            assert!(g.check().is_err());
        }
    }
}
