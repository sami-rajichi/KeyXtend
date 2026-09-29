//! Per-value repair of a module's values over its defaults, and the values that differ from them.
//! Values are compared and replaced per top-level key.

use kx_module_api::SettingsError;
use std::collections::BTreeSet;
use toml::{Table, Value};

/// A section's checker, as in `SettingsSpec::validate`.
pub(crate) type Validate = fn(&Table) -> Result<(), SettingsError>;

/// The repaired values and the keys that went back to their defaults.
#[derive(Debug, PartialEq)]
pub(crate) struct Repair {
    /// The defaults with every kept user value over them.
    pub effective: Table,
    /// The user's keys that were rejected, in table order.
    pub rejected: Vec<String>,
}

/// `base` with `key` set to `value`.
pub(crate) fn with(base: &Table, key: &str, value: Value) -> Table {
    let mut next = base.clone();
    next.insert(key.to_owned(), value);
    next
}

/// Lays the user's values over the defaults: all at once, else the keys the defaults know, repaired.
/// A key the defaults lack is rejected, so the checks grow with the defaults, not with the file.
pub(crate) fn repair(defaults: &Table, user: Table, validate: Validate) -> Repair {
    let all = over(defaults, &user, None);
    if validate(&all).is_ok() {
        return keep_all(all);
    }
    let order: Vec<String> = user.keys().cloned().collect();
    let known: Table = (user.into_iter())
        .filter(|(key, _)| defaults.contains_key(key))
        .collect();
    let some_unknown = known.len() < order.len();
    let known_all = over(defaults, &known, None);
    let Repair {
        effective,
        rejected,
    } = if some_unknown && validate(&known_all).is_ok() {
        keep_all(known_all)
    } else {
        repair_known(defaults, known, validate)
    };
    let refused: BTreeSet<&str> = rejected.iter().map(String::as_str).collect();
    let rejected = (order.into_iter())
        .filter(|key| !defaults.contains_key(key) || refused.contains(key.as_str()))
        .collect();
    Repair {
        effective,
        rejected,
    }
}

/// Every user value kept.
fn keep_all(effective: Table) -> Repair {
    Repair {
        effective,
        rejected: Vec::new(),
    }
}

/// Repairs values whose keys the defaults all know: all but one bad key, else one key at a time.
fn repair_known(defaults: &Table, known: Table, validate: Validate) -> Repair {
    let one_bad = known.keys().find_map(|bad| {
        let rest = over(defaults, &known, Some(bad));
        validate(&rest).is_ok().then(|| Repair {
            effective: rest,
            rejected: vec![bad.clone()],
        })
    });
    one_bad.unwrap_or_else(|| key_by_key(defaults, known, validate))
}

/// `defaults` with every user value over them, except the one under `skip`.
fn over(defaults: &Table, user: &Table, skip: Option<&String>) -> Table {
    let mut all = defaults.clone();
    let kept = user.iter().filter(|(key, _)| Some(*key) != skip);
    all.extend(kept.map(|(key, value)| (key.clone(), value.clone())));
    all
}

/// Keeps each user value that fits, in table order, passing again over the rejected ones until a pass keeps none.
fn key_by_key(defaults: &Table, user: Table, validate: Validate) -> Repair {
    let mut effective = defaults.clone();
    let mut pending: Vec<(String, Value)> = user.into_iter().collect();
    loop {
        let before = pending.len();
        pending.retain(|(key, value)| {
            let next = with(&effective, key, value.clone());
            let fits = validate(&next).is_ok();
            if fits {
                effective = next;
            }
            !fits
        });
        if pending.len() == before {
            break;
        }
    }
    Repair {
        effective,
        rejected: pending.into_iter().map(|(key, _)| key).collect(),
    }
}

/// The entries of `effective` that differ from `defaults`: the user's changes.
pub(crate) fn changes(effective: &Table, defaults: &Table) -> Table {
    effective
        .iter()
        .filter(|(key, value)| defaults.get(*key) != Some(*value))
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    const DEFAULTS: &str = "low = 1\nhigh = 5\nname = \"a\"\non = true\n";

    /// Unknown keys in the large test, as many as a big hand-edited file could hold.
    const UNKNOWN: usize = 5000;

    thread_local! {
        /// Checker calls made on this test's thread.
        static CALLS: Cell<usize> = const { Cell::new(0) };
    }

    fn table(text: &str) -> Table {
        text.parse().unwrap()
    }

    /// The defaults with `text` laid over them.
    fn defaults_with(text: &str) -> Table {
        let mut all = table(DEFAULTS);
        all.extend(table(text));
        all
    }

    /// The widest gap the checker allows between `low` and `high`.
    const SPAN: i64 = 10;

    /// Accepts integer `low <= high` at most `SPAN` apart, a string `name` and a bool `on`; nothing else.
    fn ordered(t: &Table) -> Result<(), SettingsError> {
        let int = |k: &str| t.get(k).and_then(Value::as_integer);
        let typed =
            t.get("name").is_some_and(Value::is_str) && t.get("on").is_some_and(Value::is_bool);
        let fits = typed && t.len() == table(DEFAULTS).len();
        match (int("low"), int("high")) {
            (Some(low), Some(high)) if fits && low <= high && high - low <= SPAN => Ok(()),
            _ => Err(SettingsError::Invalid("out of order".into())),
        }
    }

    /// `ordered`, counting each call on this thread.
    fn counted(t: &Table) -> Result<(), SettingsError> {
        CALLS.with(|calls| calls.set(calls.get() + 1));
        ordered(t)
    }

    #[test]
    fn values_valid_only_together_are_all_kept() {
        let got = repair(&table(DEFAULTS), table("low = 20\nhigh = 25"), ordered);
        assert_eq!(got.effective, defaults_with("low = 20\nhigh = 25"));
        assert!(got.rejected.is_empty());
    }

    #[test]
    fn a_pair_valid_only_together_survives_one_bad_value() {
        let user = table("low = 20\nhigh = 25\nname = 3");
        let got = repair(&table(DEFAULTS), user, ordered);
        assert_eq!(got.effective, defaults_with("low = 20\nhigh = 25"));
        assert_eq!(got.rejected, ["name"]);
    }

    #[test]
    fn a_pair_that_needs_a_second_pass_survives_two_bad_values() {
        let user = table("low = -1\nhigh = 0\nname = 3\non = 1");
        let got = repair(&table(DEFAULTS), user, ordered);
        assert_eq!(got.effective, defaults_with("low = -1\nhigh = 0"));
        assert_eq!(got.rejected, ["name", "on"]);
    }

    #[test]
    fn a_typo_beside_a_pair_valid_only_together_rejects_only_the_typo() {
        let user = table("low = 20\nhigh = 25\nlwo = 3");
        let got = repair(&table(DEFAULTS), user, ordered);
        assert_eq!(got.effective, defaults_with("low = 20\nhigh = 25"));
        assert_eq!(got.rejected, ["lwo"]);
    }

    #[test]
    fn a_typo_and_a_bad_value_beside_a_pair_keep_the_pair() {
        let user = table("low = 20\nhigh = 25\nname = 3\nzed = 1");
        let got = repair(&table(DEFAULTS), user, ordered);
        assert_eq!(got.effective, defaults_with("low = 20\nhigh = 25"));
        assert_eq!(got.rejected, ["name", "zed"]);
    }

    #[test]
    fn unknown_keys_never_multiply_the_checks() {
        let defaults = table(DEFAULTS);
        let mut user = table("low = 20\nhigh = 25\nname = 3");
        user.extend((0..UNKNOWN).map(|n| (format!("typo{n}"), Value::Integer(0))));
        let got = repair(&defaults, user, counted);
        let calls = CALLS.with(Cell::get);
        assert!(calls <= 2 * defaults.len() + 2, "{calls} checks");
        assert_eq!(got.effective, defaults_with("low = 20\nhigh = 25"));
        assert_eq!(got.rejected.len(), UNKNOWN + 1);
    }

    #[test]
    fn a_bad_value_is_rejected_alone() {
        let got = repair(&table(DEFAULTS), table("high = 9\nname = 3"), ordered);
        assert_eq!(got.effective, defaults_with("high = 9"));
        assert_eq!(got.rejected, ["name"]);
    }

    #[test]
    fn rejected_keys_come_in_table_order() {
        let got = repair(&table(DEFAULTS), table("zed = 1\nhigh = \"x\""), ordered);
        assert_eq!(got.effective, table(DEFAULTS));
        assert_eq!(got.rejected, ["high", "zed"]);
    }

    #[test]
    fn changes_leave_out_values_equal_to_their_defaults() {
        let effective = defaults_with("high = 9\nextra = true");
        assert_eq!(
            changes(&effective, &table(DEFAULTS)),
            table("high = 9\nextra = true")
        );
    }

    #[test]
    fn with_sets_one_key_and_keeps_the_rest() {
        let got = with(&table(DEFAULTS), "high", Value::Integer(7));
        assert_eq!(got, defaults_with("high = 7"));
    }
}
