//! G8's verdict: what it measured, set against each spec §10 budget it owns.
#![cfg(windows)]

use serde_json::{Value, json};

use crate::budget::{self, Part};
use crate::g2;
use crate::g8cfg::G8;
use crate::g8ui::{Presses, Resize};

/// G8's verdict in weak mode when every budget held: weak mode does not draw in software yet, so it is no pass.
pub const PARTIAL: &str = "partial";

/// What the product did while G8 measured it.
pub struct Measured {
    /// Start to the log's start line, and that line to the guard's, in ms.
    pub log: Option<(f64, f64)>,
    /// Idle private memory, in MB.
    pub memory_mb: f64,
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
}

/// What G8 measured outside the product.
pub struct Outside {
    /// The screen's refresh rate.
    pub hz: Option<f64>,
    /// The bus publish's 99th percentile, in µs.
    pub bus_p99_us: Option<f64>,
    /// Frames that never showed, from the `PresentMon` CSV.
    pub dropped: Option<f64>,
}

/// The budgets of spec §10 against a start shown after `shown_ms` and what G8 measured.
pub fn parts(g: &G8, shown_ms: Option<f64>, m: &Measured, weak: bool, o: &Outside) -> Vec<Part> {
    let start = if weak { g.weak_start_ms } else { g.start_ms };
    let lit: Option<Vec<f64>> = m.presses.lit_ms.iter().copied().collect();
    let worst = lit.and_then(|l| l.into_iter().reduce(f64::max));
    let lit_in =
        o.hz.and_then(|hz| budget::lit_within_ms(hz, g.highlight_frames, &m.presses.copy_ms));
    let char_p99 = m.g2["latency_ms"]["p99"]
        .as_f64()
        .filter(|_| g2::typed_intact(&m.g2));
    vec![
        Part::at_most("start_ms", shown_ms, start),
        Part::at_most("memory_mb", Some(m.memory_mb), g.memory_mb),
        Part::at_most("idle_core_pct", m.core_pct, g.idle_core_pct),
        Part::at_most("size_mb", Some(m.size_mb), g.size_mb),
        Part::at_most(
            "highlight_ms",
            worst.filter(|_| lit_in.is_some()),
            lit_in.unwrap_or_default(),
        ),
        Part::at_most("char_p99_ms", char_p99, g.char_ms),
        Part::at_most("bus_p99_us", o.bus_p99_us, g.bus_p99_us()),
        Part::at_most("dropped_frames", o.dropped, 0.0),
    ]
}

/// G8's `pass`: true when every budget was `met`, only `PARTIAL` then in weak mode, else false.
pub fn verdict(weak: bool, met: bool) -> Value {
    match (met, weak) {
        (false, _) => json!(false),
        (true, false) => json!(true),
        (true, true) => json!(PARTIAL),
    }
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
        let m = Measured {
            log: Some((180.0, 1074.3)),
            memory_mb: 61.0,
            core_pct: Some(0.1),
            size_mb: 58.1,
            presses: Presses {
                lit_ms: vec![Some(9.0), Some(12.5)],
                copy_ms: vec![2.0, 2.5, 3.0],
            },
            resizes: vec![],
            g2: json!({
                "planned": 100, "clicks": 100, "matched": 100, "focus_ok": true,
                "latency_ms": { "p50": 4.2, "p99": 13.1 },
            }),
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
        assert_eq!(parts.len(), 8, "one part per spec §10 budget G8 owns");
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
    fn a_late_light_a_lost_press_or_no_presentmon_csv_fails_its_part() {
        let (g, mut m, mut o) = sample();
        m.presses.lit_ms.push(Some(20.0));
        assert_eq!(
            failed(&parts(&g, Some(420.0), &m, false, &o)),
            ["highlight_ms"],
            "two frames at 165 Hz plus a copy"
        );
        m.presses.lit_ms = vec![Some(5.0), None];
        o.dropped = None;
        assert_eq!(
            failed(&parts(&g, Some(420.0), &m, false, &o)),
            ["highlight_ms", "dropped_frames"]
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
    fn weak_mode_meeting_every_budget_is_partial_and_a_miss_fails() {
        assert_eq!(verdict(false, true), json!(true));
        assert_eq!(verdict(true, true), json!(PARTIAL));
        assert_eq!(verdict(true, false), json!(false));
        assert_eq!(verdict(false, false), json!(false));
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
