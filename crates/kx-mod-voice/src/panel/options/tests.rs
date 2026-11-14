//! The Options page: every row names a setting of its kind, steppers walk their steps within the settings' ranges and
//! dim at their ends, switches flip, and a row under an off switch is not live.

use super::*;

/// The defaults with `key` set to `value`.
fn with(key: &str, value: Value) -> Table {
    let mut table = defaults();
    table.insert(key.to_owned(), value);
    table
}

/// The row of `key`.
fn opt(key: &str) -> &'static Opt {
    rows().iter().find(|o| o.key == key).unwrap()
}

#[test]
fn every_row_names_a_setting_of_its_kind() {
    let start = defaults();
    assert_eq!(rows().len(), 21, "the mock-up's options");
    for row in rows() {
        let value = start.get(&row.key).unwrap_or_else(|| panic!("{}", row.key));
        let fits = match row.kind {
            Kind::Switch => matches!(value, Value::Boolean(_) | Value::String(_)),
            Kind::Number => value.is_integer() && (row.steps.is_empty() != row.step.is_none()),
            Kind::Factor => value.is_float() && !row.steps.is_empty(),
            Kind::Place => PLACES.iter().any(|(n, _)| value.as_str() == Some(*n)),
        };
        assert!(fits, "{}", row.key);
        let under = row.under.as_ref().map(|u| opt(u).kind);
        assert!(under.is_none_or(|k| k == Kind::Switch), "{}", row.key);
    }
}

#[test]
fn each_step_is_ascending_and_within_its_setting() {
    let start = defaults();
    for row in rows() {
        let numbers: Vec<f64> = row.steps.iter().map(|s| number(s).unwrap()).collect();
        assert!(numbers.windows(2).all(|w| w[0] < w[1]), "{}", row.key);
        for step in &row.steps {
            assert!(
                accepted(&start, &row.key, step.clone()).is_some(),
                "{} {step}",
                row.key
            );
        }
    }
}

#[test]
fn a_stepper_walks_its_steps_and_dims_at_its_ends() {
    let max = opt("history_max");
    let start = defaults();
    assert_eq!(max.moved(&start, true), Some(Value::Integer(100)));
    assert_eq!(max.moved(&start, false), Some(Value::Integer(20)));
    let lowest = with("history_max", Value::Integer(1));
    assert_eq!(max.moved(&lowest, false), None);
    let between = with("history_max", Value::Integer(30));
    assert_eq!(
        max.moved(&between, true),
        Some(Value::Integer(50)),
        "off the list"
    );
    let row = max.row(&lowest, 8).unwrap();
    let OptionValue::Number { value, ends, .. } = row.value else {
        panic!("{row:?}");
    };
    assert_eq!((value, ends.low, ends.high), (1, true, false));
}

#[test]
fn a_step_row_stays_within_the_settings_range() {
    let nothing = opt("nothing_s");
    assert_eq!(nothing.moved(&defaults(), true), Some(Value::Integer(6)));
    let low = with("nothing_s", Value::Integer(2));
    assert_eq!(nothing.moved(&low, false), None, "NOTHING_S starts at 2");
    let done = opt("done_ms");
    assert_eq!(done.moved(&defaults(), false), Some(Value::Integer(8_000)));
}

#[test]
fn a_switch_flips_and_the_suffix_writes_empty_or_its_default() {
    let start = defaults();
    assert_eq!(opt("sounds").flipped(&start), Some(Value::Boolean(false)));
    let suffix = opt("type_suffix");
    assert_eq!(suffix.flipped(&start), Some(Value::String(String::new())));
    let off = with("type_suffix", Value::String(String::new()));
    assert_eq!(suffix.flipped(&off), start.get("type_suffix").cloned());
    assert_eq!(
        opt("history_max").flipped(&start),
        None,
        "a stepper does not flip"
    );
}

#[test]
fn a_row_under_an_off_switch_is_not_live() {
    let start = defaults();
    assert!(!opt("silence_stop_s").row(&start, 8).unwrap().live);
    let on = with("silence_stop", Value::Boolean(true));
    assert!(opt("silence_stop_s").row(&on, 8).unwrap().live);
    assert!(opt("history_max").row(&start, 8).unwrap().live);
}

#[test]
fn threads_zero_reads_as_the_cores_and_a_factor_as_hundredths() {
    let threads = opt("threads").row(&defaults(), 8).unwrap();
    assert!(matches!(
        threads.value,
        OptionValue::Number {
            value: 0,
            auto: Some(8),
            ..
        }
    ));
    let gain = opt("card_gain").row(&defaults(), 8).unwrap();
    assert!(matches!(
        gain.value,
        OptionValue::Factor {
            value: Hundredths(150),
            ..
        }
    ));
}

#[test]
fn the_place_walks_the_bar_places() {
    let place = opt("place");
    let start = defaults();
    assert_eq!(place.moved(&start, false), None);
    assert_eq!(
        place.moved(&start, true),
        Some(Value::String(PLACES[1].0.into()))
    );
    let row = place.row(&start, 8).unwrap();
    assert!(
        matches!(row.value, OptionValue::Choice { name, ends: Ends { low: true, high: false } } if name == PLACES[0].0)
    );
}

#[test]
fn changed_counts_the_rows_off_their_defaults() {
    let mut settings = VoiceSettings::from_defaults().unwrap();
    assert_eq!(changed(&settings), 0);
    settings.sounds = false;
    settings.history_max = 20;
    assert_eq!(changed(&settings), 2);
}
