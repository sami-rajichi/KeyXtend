//! The test build's stats lines: the numbers its mouse module logs at each hook stop, read back from its log.
#![cfg(windows)]

use kx_mod_mouse::STOPPED_LINE;
use kx_module_api::US_PER_MS;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::budget::{Part, stamp_us};

/// The lost-echo check's name in a result line: the field it reads.
pub const LOST: &str = "lost_echoes";

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
    /// The user's events heard inside the hook's own send, which passed without it.
    pub passed_inside: u32,
}

impl HookLine {
    /// The p99 in ms.
    pub fn p99_ms(&self) -> f64 {
        f64::from(self.p99_us) / f64::from(US_PER_MS)
    }
}

/// The field names, as `T` serialises them.
fn names<T: Serialize + Default>() -> Vec<String> {
    let line = serde_json::to_value(T::default()).unwrap_or_default();
    line.as_object()
        .map(|fields| fields.keys().cloned().collect())
        .unwrap_or_default()
}

/// The value of `name=value` among `line`'s words.
fn value_of<'a>(line: &'a str, name: &str) -> Option<&'a str> {
    let at = |word: &'a str| word.strip_prefix(name)?.strip_prefix('=');
    line.split_whitespace().find_map(at)
}

/// The numbers of `line` as `T`'s fields when it holds `marker`; none for any other line, and an error naming a field
/// that is missing or not a number. Nothing else on the line is kept.
pub fn numbers<T: Serialize + DeserializeOwned + Default>(
    line: &str,
    marker: &str,
) -> Result<Option<T>, String> {
    if !line.contains(marker) {
        return Ok(None);
    }
    let mut fields = Map::new();
    for name in names::<T>() {
        let value = value_of(line, &name).ok_or_else(|| format!("{marker}: no {name}"))?;
        let number: u64 = value
            .parse()
            .map_err(|_| format!("{marker}: {name} is not a number"))?;
        fields.insert(name, number.into());
    }
    serde_json::from_value(Value::Object(fields))
        .map(Some)
        .map_err(|e| format!("{marker}: {e}"))
}

/// The numbers of `line` when it is a stats line (it holds the mouse module's `STOPPED_LINE`); none for any other line,
/// and an error naming a field that is missing or not a number.
pub fn parse(line: &str) -> Result<Option<HookLine>, String> {
    numbers(line, STOPPED_LINE)
}

/// The lines of `log` stamped at or after `since_us` that `read` takes, oldest first.
pub fn read_since<T>(
    log: &str,
    since_us: i64,
    read: impl Fn(&str) -> Result<Option<T>, String>,
) -> Result<Vec<T>, String> {
    let fresh = log
        .lines()
        .filter(|l| stamp_us(l).is_some_and(|at| at >= since_us));
    let parsed: Vec<Option<T>> = fresh.map(read).collect::<Result<_, _>>()?;
    Ok(parsed.into_iter().flatten().collect())
}

/// The stats lines of `log` stamped at or after `since_us`, oldest first.
pub fn since(log: &str, since_us: i64) -> Result<Vec<HookLine>, String> {
    read_since(log, since_us, parse)
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
        passed_inside: a.passed_inside.saturating_add(b.passed_inside),
    })
}

/// The lost-echo check: `lines` may count at most `max` lost echoes, as an act id Windows dropped would lose every echo.
/// With no line, nothing is measured, so it fails.
pub fn lost(lines: &[HookLine], max: u32) -> Part {
    let got = (!lines.is_empty()).then(|| f64::from(sum(lines).lost_echoes));
    Part::at_most(LOST, got, f64::from(max))
}

/// `lines` summed for a result line, with the p99 also in ms.
pub fn summed(lines: &[HookLine]) -> Value {
    let sum = self::sum(lines);
    let mut v = serde_json::to_value(sum).unwrap_or_default();
    if let Some(fields) = v.as_object_mut() {
        fields.insert("p99_ms".to_string(), sum.p99_ms().into());
    }
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    // A workspace path from `tools/kx-gates/src`; a moved file breaks the build here, never silently.
    /// The mouse module's actor, which logs the hook's numbers at each hook stop.
    const ACTOR_RS: &str = include_str!("../../../crates/kx-mod-mouse/src/actor.rs");

    /// A stamp of the kernel's log format, and one a second later.
    const EARLY: &str = "2026-10-14T08:00:01.000000Z";
    const LATE: &str = "2026-10-14T08:00:02.000000Z";

    /// A stats line stamped `at` with `fields`, as the kernel writes it.
    fn line(at: &str, fields: &str) -> String {
        format!("{at}  INFO kx_mod_mouse::actor: {STOPPED_LINE} {fields}")
    }

    const FIELDS: &str = "events=12 p99_us=40 max_us=90 max_wait_ms=3 reinstalls=0 send_errors=0 \
                          lost_echoes=1 passed_inside=2";
    const SHUFFLED: &str = "passed_inside=2 lost_echoes=1 max_us=90 events=12 send_errors=0 \
                            p99_us=40 reinstalls=0 max_wait_ms=3";

    fn sample() -> HookLine {
        HookLine {
            events: 12,
            p99_us: 40,
            max_us: 90,
            max_wait_ms: 3,
            reinstalls: 0,
            send_errors: 0,
            lost_echoes: 1,
            passed_inside: 2,
        }
    }

    #[test]
    fn the_fields_are_the_ones_the_mouse_module_logs() {
        let fields = names::<HookLine>();
        assert!(!fields.is_empty());
        for name in fields {
            let logged = format!("{name} = s.{name}");
            assert!(ACTOR_RS.contains(&logged), "actor.rs logs no {logged}");
        }
    }

    #[test]
    fn a_stats_line_parses_in_any_field_order() {
        assert_eq!(parse(&line(EARLY, FIELDS)), Ok(Some(sample())));
        assert_eq!(parse(&line(EARLY, SHUFFLED)), Ok(Some(sample())));
    }

    #[test]
    fn a_missing_or_non_number_field_is_an_error_naming_it() {
        let gone = FIELDS.replace("p99_us=40 ", "");
        let e = parse(&line(EARLY, &gone)).expect_err("no p99");
        assert!(e.contains("p99_us"), "{e}");
        let word = FIELDS.replace("events=12", "events=many");
        let e = parse(&line(EARLY, &word)).expect_err("not a number");
        assert!(e.contains("events"), "{e}");
    }

    #[test]
    fn other_lines_and_older_stamps_are_skipped() {
        let other = format!("{LATE}  WARN keyxtend::run: something else events=1");
        let text = [line(EARLY, FIELDS), other, line(LATE, SHUFFLED)].join("\n");
        let since_us = stamp_us(LATE).expect("a stamp");
        assert_eq!(since(&text, since_us), Ok(vec![sample()]));
        assert_eq!(parse("no stamp, no line"), Ok(None));
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
            passed_inside: 5,
        };
        let want = HookLine {
            events: 17,
            p99_us: 70,
            max_us: 90,
            max_wait_ms: 9,
            reinstalls: 1,
            send_errors: 2,
            lost_echoes: 4,
            passed_inside: 7,
        };
        assert_eq!(sum(&[a, b]), want);
        assert_eq!(sum(&[]), HookLine::default());
    }

    #[test]
    fn the_summed_line_for_a_result_adds_its_p99_in_ms() {
        let v = summed(&[sample(), sample()]);
        assert_eq!(v["events"].as_u64(), Some(24));
        assert_eq!(v["p99_ms"].as_f64(), Some(sample().p99_ms()));
    }

    #[test]
    fn lost_echoes_pass_at_the_limit_and_fail_over_it_or_with_no_line() {
        assert!(
            names::<HookLine>().iter().any(|n| n == LOST),
            "the field it names"
        );
        let with = |lost_echoes| HookLine {
            lost_echoes,
            ..sample()
        };
        let at = lost(&[with(1), with(2)], 3);
        assert_eq!(
            (at.name, at.got, at.limit, at.pass),
            (LOST, Some(3.0), 3.0, true)
        );
        let over = lost(&[with(4)], 3);
        assert_eq!((over.got, over.pass), (Some(4.0), false), "names the count");
        assert!(!lost(&[], 0).pass, "no line, not measured");
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
