//! G8's verdict: what it measured, set against each spec §10 budget it owns, its churn and its readings after use and
//! after a scroll included.
#![cfg(windows)]

use serde_json::Value;

use crate::budget::{self, Part};
use crate::g2;
use crate::g8cfg::G8;
use crate::g8churn::{self, Churn};
use crate::g8scroll::{self, Scrolled};
use crate::g8size::Resize;
use crate::g8ui::Presses;
use crate::g8use::{self, Used};
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
    /// Private memory after use (ADR-0028), or why it was not read.
    pub used: Result<Used, String>,
    /// Private memory after a scroll, or why it was not read.
    pub scrolled: Result<Scrolled, String>,
    /// What the scroll's target-window left open when it closed.
    pub scroll_left: Vec<String>,
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

/// The budgets of spec §10 against a start shown after `shown_ms` and what G8 measured, the memory cold, after use and
/// after a scroll.
pub fn parts(g: &G8, shown_ms: Option<f64>, m: &Measured, weak: bool, o: &Outside) -> Vec<Part> {
    let start = if weak { g.weak_start_ms } else { g.start_ms };
    let char_p99 = m.g2["latency_ms"]["p99"]
        .as_f64()
        .filter(|_| g2::typed_intact(&m.g2));
    let [lit_p50, lit_worst] = highlight(g, &m.presses, o.hz);
    let [used_memory, used_commit] = g8use::parts(g, m.used.as_ref().ok());
    let [scrolled_memory, scrolled_commit] = g8scroll::parts(g, m.scrolled.as_ref().ok());
    let mut all = vec![
        Part::at_most("start_ms", shown_ms, start),
        Part::at_most("memory_mb", Some(m.memory.working_mb), g.memory_mb),
        Part::at_most("commit_mb", Some(m.memory.commit_mb), g.commit_mb),
        used_memory,
        used_commit,
        scrolled_memory,
        scrolled_commit,
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

/// G8's `pass`: true when every budget was `met`; only `out::PARTIAL` then in weak mode, when not every churn
/// pop-up showed or when the churn left its scroll out; else false.
pub fn verdict(weak: bool, churn: &Churn, met: bool) -> Value {
    out::pass(met, !weak && churn.whole())
}

#[cfg(test)]
mod tests;
