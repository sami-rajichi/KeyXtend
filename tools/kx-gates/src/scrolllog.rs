//! The test build's scroll stats: the counts its scroll module logs once as it quits, read back from its log. They hold
//! numbers only, never a point or a box.
#![cfg(windows)]

use kx_mod_scroll::STOPPED_LINE;
use serde::{Deserialize, Serialize};

use crate::hooklog;

/// One stats line's counts; each field has the name the scroll module logs it under.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScrollLine {
    /// Pad taps.
    pub taps: u64,
    /// Glide ticks.
    pub ticks: u64,
    /// Turns that went out.
    pub turns: u64,
    /// Aims and turns the system refused.
    pub refused: u64,
    /// Stops by Stop, a third tap, or a tap that paged our list.
    pub user: u64,
    /// Taps with no point yet.
    pub no_point: u64,
    /// Taps with nothing to scroll there.
    pub nothing: u64,
    /// Taps already at the end.
    pub at_end: u64,
    /// Stops at the end.
    pub end: u64,
    /// Stops by a press outside our windows.
    pub press: u64,
    /// Stops at the limit.
    pub limit: u64,
    /// Stops as the target went.
    pub gone: u64,
}

/// The counts of `line` when it is a scroll stats line; none for any other line, and an error naming a field that is
/// missing or not a number.
pub fn parse(line: &str) -> Result<Option<ScrollLine>, String> {
    hooklog::numbers(line, STOPPED_LINE)
}

/// The scroll stats lines of `log` stamped at or after `since_us`, added up; none when there is no line.
pub fn since(log: &str, since_us: i64) -> Result<Option<ScrollLine>, String> {
    let lines = hooklog::read_since(log, since_us, parse)?;
    Ok(lines.into_iter().reduce(add))
}

/// Two stats lines' counts, added up.
fn add(a: ScrollLine, b: ScrollLine) -> ScrollLine {
    let sum = |x: u64, y: u64| x.saturating_add(y);
    ScrollLine {
        taps: sum(a.taps, b.taps),
        ticks: sum(a.ticks, b.ticks),
        turns: sum(a.turns, b.turns),
        refused: sum(a.refused, b.refused),
        user: sum(a.user, b.user),
        no_point: sum(a.no_point, b.no_point),
        nothing: sum(a.nothing, b.nothing),
        at_end: sum(a.at_end, b.at_end),
        end: sum(a.end, b.end),
        press: sum(a.press, b.press),
        limit: sum(a.limit, b.limit),
        gone: sum(a.gone, b.gone),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::budget::stamp_us;

    // A workspace path from `tools/kx-gates/src`; a moved file breaks the build here, never silently.
    /// The scroll module's stats, which it logs once as it quits.
    const STATS_RS: &str = include_str!("../../../crates/kx-mod-scroll/src/actor/stats.rs");

    /// A stamp of the kernel's log format, and one a second later.
    const EARLY: &str = "2026-10-16T08:00:01.000000Z";
    const LATE: &str = "2026-10-16T08:00:02.000000Z";
    const FIELDS: &str = "taps=9 ticks=400 turns=380 refused=0 user=2 no_point=0 nothing=0 at_end=0 end=1 \
                          press=1 limit=0 gone=0";

    /// A stats line stamped `at` with `fields`, as the kernel writes it.
    fn line(at: &str, fields: &str) -> String {
        format!("{at}  INFO kx_mod_scroll::actor::stats: {STOPPED_LINE} {fields}")
    }

    fn sample() -> ScrollLine {
        ScrollLine {
            taps: 9,
            ticks: 400,
            turns: 380,
            user: 2,
            end: 1,
            press: 1,
            ..ScrollLine::default()
        }
    }

    #[test]
    fn the_fields_are_the_ones_the_scroll_module_logs() {
        let line = serde_json::to_value(ScrollLine::default()).expect("serialises");
        let fields = line.as_object().expect("an object");
        assert_eq!(fields.len(), 12);
        for name in fields.keys() {
            let logged = format!("{name} = self.{name}");
            assert!(STATS_RS.contains(&logged), "stats.rs logs no {logged}");
        }
    }

    #[test]
    fn a_stats_line_parses_and_any_other_line_is_none() {
        assert_eq!(parse(&line(EARLY, FIELDS)), Ok(Some(sample())));
        assert_eq!(parse(&format!("{EARLY}  INFO x: started taps=1")), Ok(None));
        let gone = FIELDS.replace("press=1 ", "");
        let e = parse(&line(EARLY, &gone)).expect_err("no press");
        assert!(e.contains("press"), "{e}");
    }

    #[test]
    fn only_lines_since_the_start_count_and_they_add_up() {
        let text = [line(EARLY, FIELDS), line(LATE, FIELDS), line(LATE, FIELDS)].join("\n");
        let since_us = stamp_us(LATE).expect("a stamp");
        let got = since(&text, since_us).expect("reads").expect("two lines");
        assert_eq!((got.taps, got.press, got.end), (18, 2, 2));
        let after = stamp_us(LATE).expect("a stamp") + 1;
        assert_eq!(since(&text, after), Ok(None), "none since then");
    }
}
