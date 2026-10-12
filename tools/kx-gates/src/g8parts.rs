//! G8's verdict: what it measured, set against each spec §10 budget it owns, its churn included.
#![cfg(windows)]

use serde_json::Value;

use crate::budget::{self, Part};
use crate::g2;
use crate::g8cfg::G8;
use crate::g8churn::{self, Churn};
use crate::g8size::Resize;
use crate::g8ui::Presses;
use crate::out;
use crate::proc::Memory;
use crate::stats::{P50, percentile};

/// What the product did while G8 measured it.
pub struct Measured {
    /// Start to the log's start line, and that line to the guard's, in ms.
    pub log: Option<(f64, f64)>,
    /// Idle private memory, read after the warm-up.
    pub memory: Memory,
    /// When it was read, in ms from the shown keyboard.
    pub memory_at_ms: f64,
    /// Idle CPU, in percent of one core.
    pub core_pct: Option<f64>,
    /// The program folder, in MB.
    pub size_mb: f64,
    /// Key highlights.
    pub presses: Presses,
    /// Grow and shrink times.
    pub resizes: Vec<Resize>,
    /// G2's result line: click to character.
    pub g2: Value,
    /// When G2 ended, on G8's QPC clock in µs, which the capture's end is set against.
    pub ran_us: i64,
    /// Memory around rounds of pop-ups.
    pub churn: Churn,
}

/// What G8 measured outside the product.
pub struct Outside {
    /// The screen's refresh rate.
    pub hz: Option<f64>,
    /// The bus publish's 99th percentile, in µs.
    pub bus_p99_us: Option<f64>,
    /// Frames lost while a pressed key rose, by G8's frame rule on the `PresentMon` CSV.
    pub dropped: Option<f64>,
}

/// The budgets of spec §10 against a start shown after `shown_ms` and what G8 measured.
pub fn parts(g: &G8, shown_ms: Option<f64>, m: &Measured, weak: bool, o: &Outside) -> Vec<Part> {
    let start = if weak { g.weak_start_ms } else { g.start_ms };
    let char_p99 = m.g2["latency_ms"]["p99"]
        .as_f64()
        .filter(|_| g2::typed_intact(&m.g2));
    let [lit_p50, lit_worst] = highlight(g, &m.presses, o.hz);
    let mut all = vec![
        Part::at_most("start_ms", shown_ms, start),
        Part::at_most("memory_mb", Some(m.memory.working_mb), g.memory_mb),
        Part::at_most("commit_mb", Some(m.memory.commit_mb), g.commit_mb),
        Part::at_most("idle_core_pct", m.core_pct, g.idle_core_pct),
        Part::at_most("size_mb", Some(m.size_mb), g.size_mb),
        lit_p50,
        lit_worst,
        Part::at_most("char_p99_ms", char_p99, g.char_ms),
        Part::at_most("bus_p99_us", o.bus_p99_us, g.bus_p99_us()),
        Part::at_most("dropped_frames", o.dropped, 0.0),
    ];
    all.extend(g8churn::parts(g, &m.churn));
    all
}

/// The key light's part for the median press.
pub const LIT_P50: &str = "highlight_p50_ms";
/// The key light's part for the slowest press.
pub const LIT_WORST: &str = "highlight_worst_ms";

/// The key light's two budgets at `hz`, each plus the median screen copy: the median press within `highlight_frames`
/// frames and the slowest within `highlight_worst_frames` (ADR-0025); a key that never lit, or no rate, fails both.
pub fn highlight(g: &G8, presses: &Presses, hz: Option<f64>) -> [Part; 2] {
    let lit: Option<Vec<f64>> = presses.lit_ms.iter().copied().collect();
    let mut lit = lit.unwrap_or_default();
    lit.sort_by(f64::total_cmp);
    let part = |name, got: Option<f64>, frames| {
        let within = hz.and_then(|hz| budget::lit_within_ms(hz, frames, &presses.copy_ms));
        Part::at_most(
            name,
            got.filter(|_| within.is_some()),
            within.unwrap_or_default(),
        )
    };
    [
        part(LIT_P50, percentile(&lit, P50), g.highlight_frames),
        part(LIT_WORST, lit.last().copied(), g.highlight_worst_frames),
    ]
}

/// True when the product `ended`, its `presses` typed exactly their characters and every budget in `parts` passed.
pub fn met(ended: bool, presses: &Presses, parts: &[Part]) -> bool {
    ended && presses.typed_ok && budget::all_pass(parts)
}

/// G8's `pass`: true when every budget was `met`; only `out::PARTIAL` then in weak mode or when not every churn
/// pop-up showed; else false.
pub fn verdict(weak: bool, churn: &Churn, met: bool) -> Value {
    out::pass(met, !weak && churn.whole())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    /// Idle CPU time of a QML animation at 60 Hz and 0.5 ms a frame over a 10 s measure, and that measure, in ms.
    const ANIMATION_MS: (f64, f64) = (300.0, 10_000.0);

    /// A run like the P1 numbers (ADR-0013: G2 p99 13.1 ms, a 55.4 MiB or 58.1 MB stage, 165 Hz) on the shipped
    /// budgets.
    fn sample() -> (G8, Measured, Outside) {
        let g = crate::config::load().expect("kx-gates.toml loads").g8;
        // Flat memory under churn, near ADR-0021's expected idle.
        let still = Memory {
            working_mb: 98.0,
            commit_mb: 138.0,
        };
        let churn = g8churn::whole_over(&g, still, still);
        let m = Measured {
            log: Some((180.0, 1074.3)),
            memory: Memory {
                working_mb: 61.0,
                commit_mb: 90.0,
            },
            memory_at_ms: 10_000.0,
            core_pct: Some(0.1),
            size_mb: 58.1,
            presses: Presses {
                lit_ms: vec![Some(9.0), Some(12.5)],
                copy_ms: vec![2.0, 2.5, 3.0],
                ..Presses::default()
            },
            resizes: vec![],
            g2: json!({
                "planned": 100, "clicks": 100, "matched": 100, "focus_ok": true, "caret_kept": 100,
                "latency_ms": { "p50": 4.2, "p99": 13.1 },
            }),
            ran_us: 0,
            churn,
        };
        let o = Outside {
            hz: Some(165.0),
            bus_p99_us: Some(4.0),
            dropped: Some(0.0),
        };
        (g, m, o)
    }

    fn failed(parts: &[Part]) -> Vec<&str> {
        parts.iter().filter(|p| !p.pass).map(|p| p.name).collect()
    }

    #[test]
    fn a_run_within_every_budget_passes() {
        let (g, m, o) = sample();
        let parts = parts(&g, Some(420.0), &m, false, &o);
        assert!(budget::all_pass(&parts), "{:?}", failed(&parts));
        assert_eq!(
            parts.len(),
            12,
            "one part per spec §10 budget G8 owns, and two for the key light"
        );
    }

    #[test]
    fn presses_that_typed_wrong_or_a_product_left_running_fail_the_run() {
        let (g, mut m, o) = sample();
        m.presses.typed_ok = true;
        let parts = parts(&g, Some(420.0), &m, false, &o);
        assert!(met(true, &m.presses, &parts));
        assert!(!met(false, &m.presses, &parts), "the product never quit");
        m.presses.typed_ok = false;
        assert!(
            !met(true, &m.presses, &parts),
            "the presses' characters did not all arrive alone"
        );
    }

    #[test]
    fn memory_that_grew_under_churn_fails_g8() {
        let (g, mut m, o) = sample();
        m.churn.last_median.commit_mb += g.churn_growth_mb + 1.0;
        assert_eq!(
            failed(&parts(&g, Some(420.0), &m, false, &o)),
            ["churn_commit_mb"]
        );
    }

    #[test]
    fn a_working_set_or_a_commit_over_its_budget_fails_its_part() {
        let (g, mut m, o) = sample();
        let over = |working_mb, commit_mb| Memory {
            working_mb,
            commit_mb,
        };
        m.memory = over(g.memory_mb + 1.0, g.commit_mb);
        assert_eq!(
            failed(&parts(&g, Some(420.0), &m, false, &o)),
            ["memory_mb"]
        );
        m.memory = over(g.memory_mb, g.commit_mb + 1.0);
        assert_eq!(
            failed(&parts(&g, Some(420.0), &m, false, &o)),
            ["commit_mb"],
            "a trim lowers the working set only, so commit is judged too"
        );
    }

    #[test]
    fn weak_mode_allows_a_slower_cold_start_only() {
        let (g, m, o) = sample();
        assert_eq!(failed(&parts(&g, Some(700.0), &m, false, &o)), ["start_ms"]);
        assert!(budget::all_pass(&parts(&g, Some(700.0), &m, true, &o)));
        assert_eq!(
            failed(&parts(&g, None, &m, true, &o)),
            ["start_ms"],
            "never shown"
        );
    }

    #[test]
    fn the_slowest_light_may_come_one_refresh_after_the_median() {
        let (g, mut m, o) = sample();
        // At 165 Hz with a 2.5 ms median copy: 14.6 ms for the median and 20.7 ms for the slowest.
        m.presses.lit_ms = vec![Some(9.0), Some(12.5), Some(20.0)];
        assert!(budget::all_pass(&parts(&g, Some(420.0), &m, false, &o)));
        m.presses.lit_ms.push(Some(21.0));
        assert_eq!(
            failed(&parts(&g, Some(420.0), &m, false, &o)),
            [LIT_WORST],
            "three frames at 165 Hz plus a copy"
        );
        m.presses.lit_ms = vec![Some(15.0), Some(15.0), Some(16.0)];
        assert_eq!(
            failed(&parts(&g, Some(420.0), &m, false, &o)),
            [LIT_P50],
            "two frames at 165 Hz plus a copy"
        );
    }

    #[test]
    fn a_lost_press_no_refresh_rate_or_no_presentmon_csv_fails_its_parts() {
        let (g, mut m, mut o) = sample();
        m.presses.lit_ms = vec![Some(5.0), None];
        o.dropped = None;
        assert_eq!(
            failed(&parts(&g, Some(420.0), &m, false, &o)),
            [LIT_P50, LIT_WORST, "dropped_frames"]
        );
        let (_, m, _) = sample();
        let no_hz = highlight(&g, &m.presses, None);
        assert!(
            no_hz.iter().all(|p| !p.pass && p.got.is_none()),
            "{no_hz:?}"
        );
    }

    #[test]
    fn an_idle_animation_or_a_lost_character_fails_its_part() {
        let (g, mut m, o) = sample();
        m.core_pct = budget::core_pct(ANIMATION_MS.0, ANIMATION_MS.1);
        m.g2["matched"] = json!(99);
        assert_eq!(
            failed(&parts(&g, Some(420.0), &m, false, &o)),
            ["idle_core_pct", "char_p99_ms"]
        );
    }

    #[test]
    fn weak_mode_or_a_churn_short_of_pop_ups_meeting_every_budget_is_partial_and_a_miss_fails() {
        let (_, m, _) = sample();
        let whole = &m.churn;
        let no_list = Churn {
            lists: None,
            ..whole.clone()
        };
        let no_bubble = Churn {
            bubbles: whole.rounds - 1,
            ..whole.clone()
        };
        assert_eq!(verdict(false, whole, true), json!(true));
        assert_eq!(
            verdict(false, &no_list, true),
            json!(true),
            "the default design"
        );
        assert_eq!(verdict(true, whole, true), json!(out::PARTIAL));
        assert_eq!(verdict(false, &no_bubble, true), json!(out::PARTIAL));
        assert_eq!(verdict(true, whole, false), json!(false));
        assert_eq!(verdict(false, whole, false), json!(false));
    }

    #[test]
    fn a_g2_error_or_a_slow_bus_fails_its_part() {
        let (g, mut m, mut o) = sample();
        m.g2["error"] = json!("the target could not be brought back");
        o.bus_p99_us = Some(g.bus_p99_us() + 1.0);
        assert_eq!(
            failed(&parts(&g, Some(420.0), &m, false, &o)),
            ["char_p99_ms", "bus_p99_us"]
        );
    }
}
