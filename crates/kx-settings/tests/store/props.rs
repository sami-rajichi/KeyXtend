//! Properties over random sample-module sections of good and bad values, versions and shapes.

use crate::spec::{DEFAULTS, HOLD, SPEC, V1_DELAY, V2_DELAY, VERSION, open, table, with_file};
use kx_module_api::keys;
use kx_settings::VERSION_KEY;
use proptest::prelude::*;
use std::fs;
use toml::{Table, Value};

/// Keys the generator draws: the current ones, older ones and an unknown one.
const KEYS: [&str; 6] = ["min_ms", "max_ms", "sound", V1_DELAY, V2_DELAY, "extra"];

/// Good and bad values for the sample settings.
fn value() -> impl Strategy<Value = Value> {
    prop_oneof![
        (0_i64..=9000).prop_map(Value::Integer),
        (-3_i64..0).prop_map(Value::Integer),
        any::<bool>().prop_map(Value::Boolean),
        "[a-z]{0,3}".prop_map(Value::String),
        (-2.0_f64..2.0).prop_map(Value::Float),
    ]
}

/// No version, bad, older, current and newer versions, or a version that is not an integer.
fn version() -> impl Strategy<Value = Option<Value>> {
    prop_oneof![
        Just(None),
        (-1_i64..=5).prop_map(|v| Some(Value::Integer(v))),
        "[0-9]".prop_map(|s| Some(Value::String(s))),
    ]
}

/// A section: mostly a table of drawn keys, sometimes a lone value.
fn section() -> impl Strategy<Value = Value> {
    let keys = prop::sample::select(KEYS.to_vec());
    let values = prop::collection::btree_map(keys, value(), 0..=KEYS.len());
    let table = (version(), values).prop_map(|(version, values)| {
        let mut t: Table = values.into_iter().map(|(k, v)| (k.to_owned(), v)).collect();
        t.extend(version.map(|v| (VERSION_KEY.to_owned(), v)));
        Value::Table(t)
    });
    prop_oneof![4 => table, 1 => value()]
}

/// True when `section` comes from a newer version, so it is kept as it is.
fn is_newer(section: &Value) -> bool {
    let version = section.get(VERSION_KEY).and_then(Value::as_integer);
    version.is_some_and(|v| v > i64::from(VERSION))
}

proptest! {
    #[test]
    fn repair_keeps_a_valid_table_and_saving_is_idempotent(section in section()) {
        let mut file = Table::new();
        file.insert(HOLD.to_string(), section.clone());
        let (_dir, files) = with_file("prop", &toml::to_string(&file).unwrap());
        let (first, _) = open(&files);
        let effective = first.get(HOLD).unwrap().clone();
        prop_assert!((SPEC.validate)(&effective).is_ok());

        let text = first.to_text().unwrap();
        fs::write(&files.settings, &text).unwrap();
        let (second, report) = open(&files);
        prop_assert_eq!(second.get(HOLD), Some(&effective));
        prop_assert_eq!(second.to_text().unwrap(), text.clone());
        prop_assert!(!second.needs_save());
        prop_assert!(report.notices.iter().all(|n| n.key == keys::SETTINGS_NEWER));

        let saved = table(&text);
        let changes = saved.get(HOLD.as_str()).and_then(Value::as_table);
        if let (Some(changes), false) = (changes, is_newer(&section)) {
            prop_assert!(changes.len() > 1, "a section holds a change besides its version");
            let defaults = table(DEFAULTS);
            for (key, value) in changes.iter().filter(|(k, _)| *k != VERSION_KEY) {
                prop_assert_ne!(defaults.get(key), Some(value));
            }
        }
    }
}
