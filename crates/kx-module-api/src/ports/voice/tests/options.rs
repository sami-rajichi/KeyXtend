//! The Options page's rows: a factor, the auto count, each number's unit and each ends, and the units' names in the
//! options table.

use super::panel::{holds, shown};
use crate::ports::Hundredths;
use crate::ports::voice::*;

/// A live row for `key`.
const fn option(key: &'static str, value: OptionValue) -> OptionRow {
    OptionRow {
        key,
        value,
        live: true,
    }
}

/// A number in `unit`, at its lowest when `low`.
const fn number(value: u32, auto: Option<u32>, unit: OptionUnit, low: bool) -> OptionValue {
    OptionValue::Number {
        value,
        auto,
        unit,
        ends: Ends { low, high: false },
    }
}

#[test]
fn options_carry_a_factor_the_auto_count_and_each_ends() {
    let factor = OptionValue::Factor {
        value: Hundredths(150),
        ends: Ends::default(),
    };
    let place = OptionValue::Choice {
        name: "top",
        ends: Ends {
            low: false,
            high: true,
        },
    };
    let rows = vec![
        option("card_gain", factor),
        option("threads", number(0, Some(8), OptionUnit::Count, true)),
        option(
            "done_ms",
            number(9_000, None, OptionUnit::MsAsSeconds, false),
        ),
        option("place", place),
    ];
    let shown = shown(VoiceSheet::Options(rows));
    let parts = [
        "Hundredths(150)",
        "auto: Some(8)",
        "unit: Count",
        "unit: MsAsSeconds",
        "low: true",
        "high: true",
        "\"top\"",
    ];
    holds(&shown, &parts);
}

// Insert the text (owner, 2026-11-17): two chips, and the hint on the row below while the second is chosen.
#[test]
fn a_pair_names_both_its_choices_and_its_hint_takes_a_row_of_its_own() {
    let pair = OptionValue::Pair {
        names: ["type", "paste"],
        second: true,
    };
    let rows = vec![option("insert", pair), option("insert", OptionValue::Hint)];
    let shown = shown(VoiceSheet::Options(rows));
    holds(&shown, &["\"type\"", "\"paste\"", "second: true", "Hint"]);
}

#[test]
fn an_options_unit_reads_as_its_snake_case_name() {
    use OptionUnit::{
        Count, Mb, Minutes, Ms, MsAsSeconds, Seconds, SecondsOrMinutes, Texts, Words,
    };
    let all = [
        Count,
        Seconds,
        MsAsSeconds,
        SecondsOrMinutes,
        Minutes,
        Ms,
        Mb,
        Texts,
        Words,
    ];
    assert_eq!(OptionUnit::ALL, all);
    let names = [
        "count",
        "seconds",
        "ms_as_seconds",
        "seconds_or_minutes",
        "minutes",
        "ms",
        "mb",
        "texts",
        "words",
    ];
    for (unit, name) in all.into_iter().zip(names) {
        let read = serde_json::from_value::<OptionUnit>(serde_json::Value::from(name));
        assert_eq!(read.ok(), Some(unit), "{name}");
    }
    let bad = serde_json::from_value::<OptionUnit>(serde_json::Value::from("hours"));
    assert!(bad.is_err());
}
