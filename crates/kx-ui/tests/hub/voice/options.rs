//! The Options page: every row of kx-mod-voice's `options.toml` has its label, a switch reads On or Off, a number reads
//! with its unit, a factor with ×, a place by its name, and a stepper names its minus and plus with the dimmed ends.

use super::{acting, heard, panel_json};
use crate::Rig;
use crate::texts::rig;
use kx_module_api::ports::{Ends, Hundredths, OptionRow, OptionUnit, OptionValue, VoiceSheet};
use serde_json::{Value, json};
use std::path::Path;

/// The voice module's Options rows, beside this crate.
const OPTIONS: &str = "../kx-mod-voice/options.toml";
/// Neither end.
const MID: Ends = Ends {
    low: false,
    high: false,
};

/// A row for setting `key` with `value`, live.
const fn row(key: &'static str, value: OptionValue) -> OptionRow {
    OptionRow {
        key,
        value,
        live: true,
    }
}

/// A number row in `unit`.
const fn number(key: &'static str, value: u32, unit: OptionUnit) -> OptionRow {
    row(
        key,
        OptionValue::Number {
            value,
            auto: None,
            unit,
            ends: MID,
        },
    )
}

/// The Options page QML reads for `rows`, in English.
fn page(rows: Vec<OptionRow>) -> Value {
    let rig = Rig::new();
    let heard = heard(&rig);
    panel_json(&rig, &heard, acting(VoiceSheet::Options(rows), &[]))
}

/// Each `(key, kind, unit)` of the voice module's rows; only a number has a unit.
fn shipped() -> Vec<(&'static str, String, Option<OptionUnit>)> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(OPTIONS);
    let text = std::fs::read_to_string(path).expect("the options table reads");
    let table: toml::Table = toml::from_str(&text).expect("the options table parses");
    let rows = table["rows"].as_array().cloned().unwrap_or_default();
    let field = |r: &toml::Value, k: &str| r[k].as_str().unwrap_or_default().to_owned();
    // Each key lives for the test run, as the module's own keys are static.
    let key = |r: &toml::Value| -> &'static str { field(r, "key").leak() };
    let unit = |r: &toml::Value| r.get("unit").and_then(|u| u.clone().try_into().ok());
    rows.iter()
        .map(|r| (key(r), field(r, "kind"), unit(r)))
        .collect()
}

#[test]
fn every_shipped_row_has_its_label_and_reads_in_every_language() {
    let rows: Vec<OptionRow> = shipped()
        .into_iter()
        .map(|(key, kind, unit)| match kind.as_str() {
            "switch" => row(key, OptionValue::Switch(true)),
            "factor" => row(
                key,
                OptionValue::Factor {
                    value: Hundredths(150),
                    ends: MID,
                },
            ),
            "place" => row(
                key,
                OptionValue::Choice {
                    name: "above",
                    ends: MID,
                },
            ),
            _ => number(key, 1, unit.expect("a number names its unit")),
        })
        .collect();
    assert!(rows.len() > 20, "the table is read: {}", rows.len());
    for tag in ["en", "fr", "ar"] {
        let (rig, _face, _host) = rig("en", tag, "auto");
        let heard = heard(&rig);
        let p = panel_json(&rig, &heard, acting(VoiceSheet::Options(rows.clone()), &[]));
        for (at, r) in rows.iter().enumerate() {
            let shown = &p["sheet"]["rows"][at];
            let label = shown["label"].as_str().unwrap_or_default();
            assert!(!label.starts_with("voice."), "{tag} {}: {label}", r.key);
            let value = shown["value"].as_str().unwrap_or_default();
            assert!(!value.starts_with("voice."), "{tag} {}: {value}", r.key);
        }
    }
}

/// Checks that each of `cases` reads as its text in English.
fn reads(cases: Vec<(OptionRow, &str)>) {
    let (rows, want): (Vec<OptionRow>, Vec<&str>) = cases.into_iter().unzip();
    let p = page(rows);
    let value = |at: usize| {
        p["sheet"]["rows"][at]["value"]
            .as_str()
            .unwrap_or_default()
            .to_owned()
    };
    let values: Vec<String> = (0..want.len()).map(value).collect();
    assert_eq!(values, want);
}

#[test]
fn each_number_reads_with_the_unit_its_row_names_whatever_its_key() {
    use OptionUnit as U;
    reads(vec![
        (number("silence_stop_s", 3, U::Seconds), "3 s"),
        (number("done_ms", 9000, U::MsAsSeconds), "9 s"),
        (number("done_ms", 2500, U::MsAsSeconds), "2.5 s"),
        (number("history_max", 50, U::Texts), "50 texts"),
        (number("voice_max_mb", 1024, U::Mb), "1 GB"),
        (number("idle_min", 5, U::Minutes), "5 min"),
        (number("record_max_s", 600, U::SecondsOrMinutes), "10 min"),
        (number("record_max_s", 30, U::SecondsOrMinutes), "30 s"),
        (number("debounce_ms", 400, U::Ms), "400 ms"),
        (number("card_margin_mb", 512, U::Mb), "512 MB"),
        (number("card_diff", 1, U::Words), "1 word"),
        (number("threads", 4, U::Count), "4"),
        (number("threads", 4, U::Minutes), "4 min"),
    ]);
}

#[test]
fn whole_seconds_and_milliseconds_take_their_arabic_plural() {
    let (rig, _face, _host) = rig("en", "ar", "auto");
    let heard = heard(&rig);
    let rows = vec![
        number("done_ms", 9000, OptionUnit::MsAsSeconds),
        number("type_gap_ms", 5, OptionUnit::Ms),
    ];
    let p = panel_json(&rig, &heard, acting(VoiceSheet::Options(rows), &[]));
    let value = |at: usize| p["sheet"]["rows"][at]["value"].clone();
    assert_eq!(value(0), "9 ثوانٍ", "not 9 ثانية");
    assert_eq!(value(1), "5 ميلي ثوانٍ");
}

#[test]
fn auto_a_factor_a_place_and_a_switch_read_as_their_words() {
    let auto = |cores| OptionValue::Number {
        value: 0,
        auto: Some(cores),
        unit: OptionUnit::Count,
        ends: MID,
    };
    let factor = OptionValue::Factor {
        value: Hundredths(150),
        ends: MID,
    };
    let place = OptionValue::Choice {
        name: "top",
        ends: MID,
    };
    reads(vec![
        (row("threads", auto(8)), "Auto (8)"),
        (row("threads", auto(0)), "Automatic"),
        (row("card_gain", factor), "1.5×"),
        (row("place", place), "Top of the screen"),
        (row("sounds", OptionValue::Switch(false)), "Off"),
    ]);
}

#[test]
fn a_switch_reads_on_and_a_stepper_names_its_ends() {
    let low = Ends {
        low: true,
        high: false,
    };
    let mut dim = number("history_max", 1, OptionUnit::Texts);
    dim.value = OptionValue::Number {
        value: 1,
        auto: None,
        unit: OptionUnit::Texts,
        ends: low,
    };
    dim.live = false;
    let p = page(vec![row("history", OptionValue::Switch(true)), dim]);
    let rows = &p["sheet"]["rows"];
    let switch = json!({"key": "history", "label": "Keep a history of texts", "kind": "switch", "value": "On",
        "on": true, "live": true, "steps": null});
    assert_eq!(rows[0], switch);
    assert_eq!(
        (&rows[1]["kind"], &rows[1]["live"]),
        (&json!("number"), &json!(false))
    );
    let steps = json!({"low": true, "high": false, "minus": "Less: Texts kept", "plus": "More: Texts kept"});
    assert_eq!(rows[1]["steps"], steps);
    assert_eq!(
        (&p["page"], &p["sub"]),
        (&json!("options"), &json!("For every language"))
    );
    assert_eq!(p["line"], "Changes apply at once.");
    assert_eq!(p["foot"], json!([]));
}

#[test]
fn a_place_is_a_choice_with_chevrons() {
    let p = page(vec![row(
        "place",
        OptionValue::Choice {
            name: "above",
            ends: MID,
        },
    )]);
    let shown = &p["sheet"]["rows"][0];
    assert_eq!(
        (&shown["kind"], &shown["value"]),
        (&json!("choice"), &json!("Above the keyboard"))
    );
    assert_eq!(shown["label"], "Bar position");
}
