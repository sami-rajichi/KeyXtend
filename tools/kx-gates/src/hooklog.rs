//! The test build's stats lines: the numbers its mouse module logs at each hook stop, read back from its log.
#![cfg(windows)]

use kx_module_api::US_PER_MS;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::budget::stamp_us;

/// One stats line's numbers; each field has the name the product logs it under.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HookLine {
    /// Events the hooks gave the engine.
    pub events: u64,
    /// The 99th percentile of the hook callback's time, in µs.
    pub p99_us: u32,
    /// The longest callback, in µs.
    pub max_us: u32,
    /// The longest an event waited before the hook saw it, in ms.
    pub max_wait_ms: u32,
    /// How many times the health check put the hooks back.
    pub reinstalls: u32,
    /// How many times Windows refused the hook's input.
    pub send_errors: u32,
    /// How many times its own input did not come back through the hook in time.
    pub lost_echoes: u32,
}

impl HookLine {
    /// The p99 in ms.
    pub fn p99_ms(&self) -> f64 {
        f64::from(self.p99_us) / f64::from(US_PER_MS)
    }
}

/// The field names, as `HookLine` serialises them.
fn names() -> Vec<String> {
    let line = serde_json::to_value(HookLine::default()).unwrap_or_default();
    line.as_object()
        .map(|fields| fields.keys().cloned().collect())
        .unwrap_or_default()
}

/// The value of `name=value` among `line`'s words.
fn value_of<'a>(line: &'a str, name: &str) -> Option<&'a str> {
    let at = |word: &'a str| word.strip_prefix(name)?.strip_prefix('=');
    line.split_whitespace().find_map(at)
}

/// The numbers of `line` when it is a stats line (it holds `hook_line`); none for any other line, and an error naming
/// a field that is missing or not a number.
pub fn parse(line: &str, hook_line: &str) -> Result<Option<HookLine>, String> {
    if !line.contains(hook_line) {
        return Ok(None);
    }
    let mut fields = Map::new();
    for name in names() {
        let value = value_of(line, &name).ok_or_else(|| format!("{hook_line}: no {name}"))?;
        let number: u64 = value
            .parse()
            .map_err(|_| format!("{hook_line}: {name} is not a number"))?;
        fields.insert(name, number.into());
    }
    serde_json::from_value(Value::Object(fields))
        .map(Some)
        .map_err(|e| format!("{hook_line}: {e}"))
}

/// The stats lines of `log` stamped at or after `since_us`, oldest first.
pub fn since(log: &str, since_us: i64, hook_line: &str) -> Result<Vec<HookLine>, String> {
    let fresh = log
        .lines()
        .filter(|l| stamp_us(l).is_some_and(|at| at >= since_us));
    let parsed: Vec<Option<HookLine>> = fresh
        .map(|l| parse(l, hook_line))
        .collect::<Result<_, _>>()?;
    Ok(parsed.into_iter().flatten().collect())
}

/// `lines` as one: the counts add up, and each time is the largest.
pub fn sum(lines: &[HookLine]) -> HookLine {
    lines.iter().fold(HookLine::default(), |a, b| HookLine {
        events: a.events.saturating_add(b.events),
        p99_us: a.p99_us.max(b.p99_us),
        max_us: a.max_us.max(b.max_us),
        max_wait_ms: a.max_wait_ms.max(b.max_wait_ms),
        reinstalls: a.reinstalls.saturating_add(b.reinstalls),
        send_errors: a.send_errors.saturating_add(b.send_errors),
        lost_echoes: a.lost_echoes.saturating_add(b.lost_echoes),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A stamp of the kernel's log format, and one a second later.
    const EARLY: &str = "2026-10-14T08:00:01.000000Z";
    const LATE: &str = "2026-10-14T08:00:02.000000Z";

    fn hook() -> String {
        crate::config::load()
            .expect("kx-gates.toml loads")
            .gate_input
            .hook_line
    }

    /// A stats line stamped `at` with `fields`, as the kernel writes it.
    fn line(at: &str, fields: &str) -> String {
        format!("{at}  INFO kx_mod_mouse::actor: {} {fields}", hook())
    }

    const FIELDS: &str =
        "events=12 p99_us=40 max_us=90 max_wait_ms=3 reinstalls=0 send_errors=0 lost_echoes=1";
    const SHUFFLED: &str =
        "lost_echoes=1 max_us=90 events=12 send_errors=0 p99_us=40 reinstalls=0 max_wait_ms=3";

    fn sample() -> HookLine {
        HookLine {
            events: 12,
            p99_us: 40,
            max_us: 90,
            max_wait_ms: 3,
            reinstalls: 0,
            send_errors: 0,
            lost_echoes: 1,
        }
    }

    #[test]
    fn a_stats_line_parses_in_any_field_order() {
        let hook = hook();
        assert_eq!(parse(&line(EARLY, FIELDS), &hook), Ok(Some(sample())));
        assert_eq!(parse(&line(EARLY, SHUFFLED), &hook), Ok(Some(sample())));
    }

    #[test]
    fn a_missing_or_non_number_field_is_an_error_naming_it() {
        let hook = hook();
        let gone = FIELDS.replace("p99_us=40 ", "");
        let e = parse(&line(EARLY, &gone), &hook).expect_err("no p99");
        assert!(e.contains("p99_us"), "{e}");
        let word = FIELDS.replace("events=12", "events=many");
        let e = parse(&line(EARLY, &word), &hook).expect_err("not a number");
        assert!(e.contains("events"), "{e}");
    }

    #[test]
    fn other_lines_and_older_stamps_are_skipped() {
        let hook = hook();
        let other = format!("{LATE}  WARN keyxtend::run: something else events=1");
        let text = [line(EARLY, FIELDS), other, line(LATE, SHUFFLED)].join("\n");
        let since_us = stamp_us(LATE).expect("a stamp");
        assert_eq!(since(&text, since_us, &hook), Ok(vec![sample()]));
        assert_eq!(parse("no stamp, no line", &hook), Ok(None));
    }

    #[test]
    fn the_sum_adds_the_counts_and_keeps_the_largest_times() {
        let a = sample();
        let b = HookLine {
            events: 5,
            p99_us: 70,
            max_us: 60,
            max_wait_ms: 9,
            reinstalls: 1,
            send_errors: 2,
            lost_echoes: 3,
        };
        let want = HookLine {
            events: 17,
            p99_us: 70,
            max_us: 90,
            max_wait_ms: 9,
            reinstalls: 1,
            send_errors: 2,
            lost_echoes: 4,
        };
        assert_eq!(sum(&[a, b]), want);
        assert_eq!(sum(&[]), HookLine::default());
    }

    #[test]
    fn the_p99_reads_in_ms() {
        let line = HookLine {
            p99_us: 1_500,
            ..HookLine::default()
        };
        assert!((line.p99_ms() - 1.5).abs() < f64::EPSILON);
    }
}
