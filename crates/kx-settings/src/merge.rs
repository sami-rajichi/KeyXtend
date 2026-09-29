//! Per-value repair of a module's values over its defaults, and the values that differ from them.
//! Values are compared and replaced per top-level key.

use kx_module_api::SettingsError;
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

/// Lays the user's values over the defaults: all at once when valid together, else one at a time.
///
/// The one-at-a-time passes go in table order and repeat over the rejected keys until a pass keeps none.
pub(crate) fn repair(defaults: &Table, user: Table, validate: Validate) -> Repair {
    let mut all = defaults.clone();
    all.extend(user.iter().map(|(k, v)| (k.clone(), v.clone())));
    if validate(&all).is_ok() {
        return Repair {
            effective: all,
            rejected: Vec::new(),
        };
    }
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

    const DEFAULTS: &str = "low = 1\nhigh = 5\nname = \"a\"\n";

    fn table(text: &str) -> Table {
        text.parse().unwrap()
    }

    /// The widest gap the checker allows between `low` and `high`.
    const SPAN: i64 = 10;

    /// Accepts integer `low <= high` at most `SPAN` apart, and a string `name`; nothing else.
    fn ordered(t: &Table) -> Result<(), SettingsError> {
        let int = |k: &str| t.get(k).and_then(Value::as_integer);
        let fits = t.len() == 3 && t.get("name").is_some_and(Value::is_str);
        match (int("low"), int("high")) {
            (Some(low), Some(high)) if fits && low <= high && high - low <= SPAN => Ok(()),
            _ => Err(SettingsError::Invalid("out of order".into())),
        }
    }

    #[test]
    fn values_valid_only_together_are_all_kept() {
        let got = repair(&table(DEFAULTS), table("low = 20\nhigh = 25"), ordered);
        assert_eq!(got.effective, table("low = 20\nhigh = 25\nname = \"a\""));
        assert!(got.rejected.is_empty());
    }

    #[test]
    fn a_pair_that_needs_a_second_pass_survives_a_bad_value() {
        let user = table("low = -1\nhigh = 0\nname = 3");
        let got = repair(&table(DEFAULTS), user, ordered);
        assert_eq!(got.effective, table("low = -1\nhigh = 0\nname = \"a\""));
        assert_eq!(got.rejected, ["name"]);
    }

    #[test]
    fn a_bad_value_is_rejected_alone() {
        let got = repair(&table(DEFAULTS), table("high = 9\nname = 3"), ordered);
        assert_eq!(got.effective, table("low = 1\nhigh = 9\nname = \"a\""));
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
        let effective = table("low = 1\nhigh = 9\nname = \"a\"\nextra = true");
        assert_eq!(
            changes(&effective, &table(DEFAULTS)),
            table("high = 9\nextra = true")
        );
    }

    #[test]
    fn with_sets_one_key_and_keeps_the_rest() {
        let got = with(&table(DEFAULTS), "high", Value::Integer(7));
        assert_eq!(got, table("low = 1\nhigh = 7\nname = \"a\""));
    }
}
