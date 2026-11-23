//! A row's facts (ADR-0047 decision 8): the day they were read and their source page, and each card line as a text id
//! with its values by kind, so a card's sentence is a translation filled with the row's numbers and no price or claim
//! sits in code. Past `facts_days` their day takes the warning tone (edge 15).

use std::collections::{BTreeMap, BTreeSet};

use kx_module_api::ports::{Date, Fact, FactArg, FactValue, HttpsUrl, Hundredths};
use serde::Deserialize;
use thiserror::Error;
use toml::value::Datetime;

use super::ServiceError;
use crate::plain::{day, one_line};

/// How far a written amount may sit from a whole hundredth, for the float's own error.
const SLACK: f64 = 1e-6;
/// The letters of an ISO 4217 currency code.
const CURRENCY_LEN: usize = 3;
/// The marks a fact's id or a value's name may hold besides lower-case ASCII letters and digits.
const WORD_MARKS: &[u8] = b"-_";

/// Why a fact was refused.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Error)]
pub enum FactError {
    /// The day is no plain day.
    #[error("a day is no plain day")]
    Day,
    /// The source page is missing on a named row, or is not HTTPS.
    #[error("its source page is missing or not https")]
    Source,
    /// The currency is not three upper-case letters, or money has none.
    #[error("its currency is not an iso 4217 code, or money has none")]
    Currency,
    /// The text id is not a plain word.
    #[error("its text id is not a plain word")]
    Id,
    /// A value's name is not a plain word, or two values share it.
    #[error("a value's name is not a plain word or is given twice")]
    Arg,
    /// A number or an amount is negative, too large or not whole hundredths.
    #[error("a number or an amount is not whole hundredths")]
    Amount,
    /// A text value is empty or holds a hidden mark.
    #[error("a text value is empty or holds a hidden mark")]
    Text,
}

/// The facts as written.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RawFacts {
    /// The day they were read.
    read: Datetime,
    /// The page they were read on; none for Custom.
    #[serde(default)]
    source: Option<String>,
    /// The currency of every amount.
    #[serde(default)]
    currency: Option<String>,
    /// Where the voice goes.
    place: RawFact,
    /// Whether the service trains on it.
    trains: RawFact,
    /// What it keeps.
    keeps: RawFact,
    /// The free allowance.
    free: RawFact,
    /// The price of light use.
    light: RawFact,
    /// The price of heavy use.
    heavy: RawFact,
    /// The card's warning.
    #[serde(default)]
    warn: Option<RawFact>,
    /// The least a request bills, when the service rounds up.
    #[serde(default)]
    bills: Option<RawFact>,
    /// A rule the service sets on its use.
    #[serde(default)]
    rule: Option<RawFact>,
}

/// A fact as written: its text's id and its values by kind.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RawFact {
    /// Its text's id.
    id: String,
    /// Whole numbers.
    #[serde(default)]
    count: BTreeMap<String, u32>,
    /// Numbers in hundredths, such as a fee.
    #[serde(default)]
    number: BTreeMap<String, f64>,
    /// Amounts in the row's currency.
    #[serde(default)]
    money: BTreeMap<String, f64>,
    /// Days.
    #[serde(default)]
    day: BTreeMap<String, Datetime>,
    /// Names from the service's own pages.
    #[serde(default)]
    text: BTreeMap<String, String>,
    /// It takes the warning tone.
    #[serde(default)]
    warn: bool,
}

/// A row's checked facts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Facts {
    /// The day they were read on the service's own pages.
    pub read: Date,
    /// The page they were read on, which the card links; `None` for Custom.
    pub source: Option<HttpsUrl>,
    /// Where the voice goes.
    pub place: Fact,
    /// Whether the service trains on it.
    pub trains: Fact,
    /// What it keeps.
    pub keeps: Fact,
    /// The free allowance.
    pub free: Fact,
    /// The price of light use.
    pub light: Fact,
    /// The price of heavy use.
    pub heavy: Fact,
    /// The card's warning, in the warning tone; `None` for none.
    pub warn: Option<Fact>,
    /// The least a request bills when the service rounds up; `None` for none.
    pub bills: Option<Fact>,
    /// A rule the service sets on its use, such as a free plan for non-commercial use only; `None` for none.
    pub rule: Option<Fact>,
}

impl Facts {
    /// True when they were read more than `days` days before `today`, so their day takes the warning tone (edge 15);
    /// facts with no source page, as Custom's, never age.
    #[must_use]
    pub fn old(&self, today: Date, days: u16) -> bool {
        self.source.is_some() && today.days_since(self.read) > i64::from(days)
    }
}

/// The row's facts: a plain day, a source page for a named row, a currency for money, and each fact checked.
pub(super) fn checked(raw: RawFacts, named: bool) -> Result<Facts, ServiceError> {
    let whole = |why| ServiceError::Fact("facts", why);
    let read = day(&raw.read).ok_or(whole(FactError::Day))?;
    let source = match raw.source.as_deref().map(HttpsUrl::new) {
        Some(url) => Some(url.ok_or(whole(FactError::Source))?),
        None if named => return Err(whole(FactError::Source)),
        None => None,
    };
    let money = raw.currency.as_deref();
    let currency_ok =
        money.is_none_or(|c| c.len() == CURRENCY_LEN && c.bytes().all(|b| b.is_ascii_uppercase()));
    if !currency_ok {
        return Err(whole(FactError::Currency));
    }
    let one = |raw: RawFact, name: &'static str| {
        fact(raw, money).map_err(|why| ServiceError::Fact(name, why))
    };
    let warn = raw.warn.map(|w| one(w, "warn")).transpose()?;
    let bills = raw.bills.map(|b| one(b, "bills")).transpose()?;
    let rule = raw.rule.map(|r| one(r, "rule")).transpose()?;
    Ok(Facts {
        read,
        source,
        place: one(raw.place, "place")?,
        trains: one(raw.trains, "trains")?,
        keeps: one(raw.keeps, "keeps")?,
        free: one(raw.free, "free")?,
        light: one(raw.light, "light")?,
        heavy: one(raw.heavy, "heavy")?,
        warn: warn.map(|w| Fact { warn: true, ..w }),
        bills,
        rule,
    })
}

/// One checked fact, its amounts in `currency`.
fn fact(raw: RawFact, currency: Option<&str>) -> Result<Fact, FactError> {
    if !word(&raw.id) {
        return Err(FactError::Id);
    }
    let mut args = Vec::new();
    let mut put = |name: String, value| args.push(FactArg { name, value });
    for (name, n) in raw.count {
        put(name, FactValue::Count(n));
    }
    for (name, n) in raw.number {
        put(name, FactValue::Number(hundredths(n)?));
    }
    for (name, n) in raw.money {
        let currency = currency.ok_or(FactError::Currency)?.to_owned();
        put(
            name,
            FactValue::Money {
                amount: hundredths(n)?,
                currency,
            },
        );
    }
    for (name, when) in raw.day {
        put(name, FactValue::Day(day(&when).ok_or(FactError::Day)?));
    }
    for (name, text) in raw.text {
        let value = one_line(&text)
            .then_some(FactValue::Text(text))
            .ok_or(FactError::Text)?;
        put(name, value);
    }
    let names: BTreeSet<&str> = args.iter().map(|a| a.name.as_str()).collect();
    if names.len() != args.len() || !names.iter().all(|n| word(n)) {
        return Err(FactError::Arg);
    }
    Ok(Fact {
        id: raw.id,
        args,
        warn: raw.warn,
    })
}

/// `n` in whole hundredths: finite, not negative, within `u32` and within `SLACK` of a whole hundredth.
fn hundredths(n: f64) -> Result<Hundredths, FactError> {
    let scaled = n * f64::from(Hundredths::PER_ONE);
    let whole = scaled.round();
    let fits = scaled.is_finite()
        && whole >= 0.0
        && whole <= f64::from(u32::MAX)
        && (scaled - whole).abs() < SLACK;
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "only kept when whole is a checked whole number within u32"
    )]
    let cents = whole as u32;
    fits.then_some(Hundredths(cents)).ok_or(FactError::Amount)
}

/// True for a plain word: a lower-case ASCII letter, then letters, digits and `WORD_MARKS`.
fn word(text: &str) -> bool {
    let plain = |b: u8| b.is_ascii_lowercase() || b.is_ascii_digit() || WORD_MARKS.contains(&b);
    text.bytes().next().is_some_and(|b| b.is_ascii_lowercase()) && text.bytes().all(plain)
}
