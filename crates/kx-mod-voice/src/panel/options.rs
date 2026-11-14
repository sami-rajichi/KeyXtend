//! The Options page (`options.toml`): each row's value from `[voice]`, a switch's flip and a stepper's move. A move
//! takes the next step the settings' own check accepts, so each range is written once, in the settings.

use std::sync::LazyLock;

use kx_module_api::ports::{Ends, Hundredths, OptionRow, OptionValue};
use serde::Deserialize;
use toml::{Table, Value};

use crate::settings::{PLACES, VoiceSettings};
use kx_module_api::Settings as _;

/// The rows, read once.
static ROWS: LazyLock<Result<Vec<Opt>, String>> = LazyLock::new(|| {
    #[derive(Deserialize)]
    struct File {
        rows: Vec<Opt>,
    }
    let file: Result<File, _> = toml::from_str(include_str!("../../options.toml"));
    file.map(|f| f.rows).map_err(|e| e.to_string())
});

/// How a row changes its value.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    /// On or off.
    Switch,
    /// A whole number, with minus and plus.
    Number,
    /// A factor kept in hundredths, with minus and plus.
    Factor,
    /// One of the bar's places, with chevrons.
    Place,
}

/// One row of the Options page.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Opt {
    /// The setting, which also names the row's text.
    pub key: String,
    /// How it changes.
    pub kind: Kind,
    /// The values minus and plus walk through, ascending.
    #[serde(default)]
    pub steps: Vec<Value>,
    /// What minus and plus take or add without steps.
    pub step: Option<i64>,
    /// The switch that makes it live.
    pub under: Option<String>,
    /// 0 reads as the processor's real cores.
    #[serde(default)]
    pub auto: bool,
}

/// The rows in order; none when `options.toml` does not parse, which a test rules out.
#[must_use]
pub fn rows() -> &'static [Opt] {
    ROWS.as_deref().unwrap_or_default()
}

/// The settings as a table of values.
#[must_use]
pub fn values(settings: &VoiceSettings) -> Table {
    match Value::try_from(settings) {
        Ok(Value::Table(table)) => table,
        _ => Table::new(),
    }
}

/// How many rows differ from their defaults, as the home list's Changed: 2.
#[must_use]
pub fn changed(settings: &VoiceSettings) -> u16 {
    let (now, start) = (values(settings), defaults());
    let differs = |o: &&Opt| now.get(&o.key) != start.get(&o.key);
    u16::try_from(rows().iter().filter(differs).count()).unwrap_or(u16::MAX)
}

impl Opt {
    /// Its row for the page, with `cores` for what 0 threads means; `None` for a key the settings lack.
    #[must_use]
    pub fn row(&'static self, table: &Table, cores: u16) -> Option<OptionRow> {
        let now = table.get(&self.key)?;
        let ends = || Ends {
            low: self.moved(table, false).is_none(),
            high: self.moved(table, true).is_none(),
        };
        let value = match self.kind {
            Kind::Switch => OptionValue::Switch(on(now)),
            Kind::Number => {
                let value = now.as_integer().and_then(|v| u32::try_from(v).ok())?;
                let auto = (self.auto && value == 0).then_some(u32::from(cores));
                OptionValue::Number {
                    value,
                    auto,
                    ends: ends(),
                }
            }
            Kind::Factor => OptionValue::Factor {
                value: hundredths(now.as_float()?),
                ends: ends(),
            },
            Kind::Place => place(now)?,
        };
        let live = self
            .under
            .as_ref()
            .is_none_or(|u| table.get(u).is_some_and(on));
        Some(OptionRow {
            key: self.key.as_str(),
            value,
            live,
        })
    }

    /// The value a flip of this switch writes; `None` for another kind or a value the settings refuse.
    #[must_use]
    pub fn flipped(&self, table: &Table) -> Option<Value> {
        let next = match table.get(&self.key)? {
            Value::Boolean(b) if self.kind == Kind::Switch => Value::Boolean(!b),
            Value::String(s) if self.kind == Kind::Switch && !s.is_empty() => {
                Value::String(String::new())
            }
            Value::String(_) if self.kind == Kind::Switch => defaults().get(&self.key)?.clone(),
            _ => return None,
        };
        accepted(table, &self.key, next)
    }

    /// The value minus (or plus when `up`) writes: the next step the settings accept; `None` at an end.
    #[must_use]
    pub fn moved(&self, table: &Table, up: bool) -> Option<Value> {
        let now = table.get(&self.key)?;
        if self.kind == Kind::Place {
            let at = PLACES.iter().position(|(n, _)| now.as_str() == Some(*n))?;
            let next = if up {
                at.checked_add(1)
            } else {
                at.checked_sub(1)
            };
            let name = PLACES.get(next?)?.0;
            return accepted(table, &self.key, Value::String(name.to_owned()));
        }
        if self.steps.is_empty() {
            let (now, step) = (now.as_integer()?, self.step?);
            let next = if up {
                now.checked_add(step)
            } else {
                now.checked_sub(step)
            };
            return accepted(table, &self.key, Value::Integer(next?));
        }
        let at = number(now)?;
        let beyond = |s: &&Value| number(s).is_some_and(|x| if up { x > at } else { x < at });
        let mut steps: Vec<&Value> = self.steps.iter().filter(beyond).collect();
        if !up {
            steps.reverse();
        }
        steps
            .into_iter()
            .find_map(|s| accepted(table, &self.key, s.clone()))
    }
}

/// `value` when `table` with `key` set to it passes the settings' check.
fn accepted(table: &Table, key: &str, value: Value) -> Option<Value> {
    let mut next = table.clone();
    next.insert(key.to_owned(), value.clone());
    let settings: VoiceSettings = Value::Table(next).try_into().ok()?;
    settings.check().ok().map(|()| value)
}

/// The defaults as a table of values.
fn defaults() -> Table {
    VoiceSettings::from_defaults()
        .map(|s| values(&s))
        .unwrap_or_default()
}

/// A switch's state: a flag, or a text that is not empty, as `type_suffix`.
fn on(value: &Value) -> bool {
    match value {
        Value::Boolean(b) => *b,
        Value::String(s) => !s.is_empty(),
        _ => false,
    }
}

/// The bar place `now` names, with its chevrons dimmed at the first and last place.
fn place(now: &Value) -> Option<OptionValue> {
    let at = PLACES.iter().position(|(n, _)| now.as_str() == Some(*n))?;
    let ends = Ends {
        low: at == 0,
        high: at + 1 == PLACES.len(),
    };
    Some(OptionValue::Choice {
        name: PLACES.get(at)?.0,
        ends,
    })
}

/// A number to order steps by.
fn number(value: &Value) -> Option<f64> {
    match value {
        Value::Integer(i) => i32::try_from(*i).ok().map(f64::from),
        Value::Float(f) => Some(*f),
        _ => None,
    }
}

/// A factor in hundredths, as the view carries it.
fn hundredths(factor: f64) -> Hundredths {
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a factor the settings accept is small and positive, so its hundredths fit"
    )]
    let value = (factor * f64::from(Hundredths::PER_ONE)).round().max(0.0) as u32;
    Hundredths(value)
}

#[cfg(test)]
mod tests;
