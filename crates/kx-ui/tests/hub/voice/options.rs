//! The Options page: every row of kx-mod-voice's `options.toml` has its label, a switch reads On or Off, a number reads
//! with its unit, a factor with ×, a place by its name, and a stepper names its minus and plus with the dimmed ends.

use super::{acting, heard, panel_json};
use crate::Rig;
use crate::texts::rig;
use kx_module_api::ports::{Ends, Hundredths, OptionRow, OptionValue, VoiceSheet};
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

/// A number row.
const fn number(key: &'static str, value: u32) -> OptionRow {
    row(
        key,
        OptionValue::Number {
            value,
            auto: None,
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

/// Each `(key, kind)` of the voice module's rows.
fn shipped() -> Vec<(&'static str, String)> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(OPTIONS);
    let text = std::fs::read_to_string(path).expect("the options table reads");
    let table: toml::Table = toml::from_str(&text).expect("the options table parses");
    let rows = table["rows"].as_array().cloned().unwrap_or_default();
    let field = |r: &toml::Value, k: &str| r[k].as_str().unwrap_or_default().to_owned();
    // Each key lives for the test run, as the module's own keys are static.
    let key = |r: &toml::Value| -> &'static str { field(r, "key").leak() };
    rows.iter().map(|r| (key(r), field(r, "kind"))).collect()
}

#[test]
fn every_shipped_row_has_its_label_and_reads_in_every_language() {
    let rows: Vec<OptionRow> = shipped()
        .into_iter()
        .map(|(key, kind)| match kind.as_str() {
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
            _ => number(key, 1),
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
fn each_number_reads_with_its_unit() {
    reads(vec![
        (number("silence_stop_s", 3), "3 s"),
        (number("done_ms", 9000), "9 s"),
        (number("history_max", 50), "50 texts"),
        (number("voice_max_mb", 1024), "1 GB"),
        (number("idle_min", 5), "5 min"),
        (number("record_max_s", 600), "10 min"),
        (number("record_max_s", 30), "30 s"),
        (number("nothing_s", 5), "5 s"),
        (number("debounce_ms", 400), "400 ms"),
        (number("type_gap_ms", 10), "10 ms"),
        (number("wave_ms", 300), "300 ms"),
        (number("min_free_mb", 2048), "2 GB"),
        (number("card_margin_mb", 512), "512 MB"),
        (number("card_diff", 1), "1 word"),
        (number("threads", 4), "4"),
    ]);
}

#[test]
fn whole_seconds_and_milliseconds_take_their_arabic_plural() {
    let (rig, _face, _host) = rig("en", "ar", "auto");
    let heard = heard(&rig);
    let rows = vec![number("done_ms", 9000), number("type_gap_ms", 5)];
    let p = panel_json(&rig, &heard, acting(VoiceSheet::Options(rows), &[]));
    let value = |at: usize| p["sheet"]["rows"][at]["value"].clone();
    assert_eq!(value(0), "9 ثوانٍ", "not 9 ثانية");
    assert_eq!(value(1), "5 ميلي ثوانٍ");
}

#[test]
fn auto_a_factor_a_place_and_a_switch_read_as_their_words() {
    let auto = OptionValue::Number {
        value: 0,
        auto: Some(8),
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
        (row("threads", auto), "Auto (8)"),
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
    let mut dim = number("history_max", 1);
    dim.value = OptionValue::Number {
        value: 1,
        auto: None,
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
