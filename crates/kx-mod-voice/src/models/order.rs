//! The orders (ADR-0043 decisions 4 and 5): each tested language's list, whose first is its default, and the general
//! order for a language without one, limited to the rows that know it and marked untested. A row that never leads is
//! first in no list.

use std::collections::{BTreeMap, BTreeSet};

use kx_module_api::ports::{Date, fits_iso};
use serde::Deserialize;
use thiserror::Error;
use toml::value::Datetime;

use super::{Row, Table, TableError};

/// Why an order was refused.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Error)]
pub enum OrderError {
    /// Its language is no ISO 639 code, or has two lists.
    #[error("its language is no code or is listed twice")]
    Lang,
    /// Its test date is no plain day.
    #[error("its test date is no plain day")]
    Date,
    /// It names a row the table lacks.
    #[error("it names a row the table lacks")]
    Unknown,
    /// It names a row twice.
    #[error("it names a row twice")]
    Twice,
    /// It names a row that does not know its language.
    #[error("it names a row that does not know its language")]
    Stranger,
    /// It leaves out a row it should hold.
    #[error("it leaves out a row")]
    Missing,
    /// A row that never leads would be first.
    #[error("a row that never leads would be first")]
    Lead,
}

/// A tested order as written.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RawOrder {
    /// The keyboard language.
    lang: String,
    /// The day of the tests it follows.
    tested: Datetime,
    /// The row ids, best first.
    models: Vec<String>,
}

/// The general order as written.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RawGeneral {
    /// The day of the tests it follows.
    tested: Datetime,
    /// Every row id, best first.
    models: Vec<String>,
}

/// A checked order: its test day and its rows by their place in the table.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Order {
    /// The day of the tests it follows.
    tested: Date,
    /// The rows' places in the table, best first.
    rows: Vec<usize>,
}

/// The models for one language, best first.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ranked<'a> {
    /// The rows that know the language, best first; empty when none does.
    pub rows: Vec<&'a Row>,
    /// The day of the tests the order follows.
    pub tested: Date,
    /// The language has no tested list of its own, so the general order serves it.
    pub untested: bool,
}

impl<'a> Ranked<'a> {
    /// The best row, the language's default.
    #[must_use]
    pub fn first(&self) -> Option<&'a Row> {
        self.rows.first().copied()
    }
}

impl Table {
    /// The models that know `lang`, best first: its tested list, else the general order limited to them.
    #[must_use]
    pub fn order(&self, lang: &str) -> Ranked<'_> {
        match self.orders.get(lang) {
            Some(own) => self.ranked(own, Some(lang), false),
            None => self.ranked(&self.general, Some(lang), true),
        }
    }

    /// Every row in the general order.
    #[must_use]
    pub fn general(&self) -> Ranked<'_> {
        self.ranked(&self.general, None, true)
    }

    /// The default for `lang`: the first of its order; `None` when no model knows it.
    #[must_use]
    pub fn default_for(&self, lang: &str) -> Option<&Row> {
        self.order(lang).first()
    }

    /// The languages with a tested order of their own, in code order.
    pub fn tested(&self) -> impl Iterator<Item = &str> {
        self.orders.keys().map(String::as_str)
    }

    /// Every keyboard language some row knows.
    #[must_use]
    pub fn langs(&self) -> BTreeSet<&str> {
        let codes = self.rows.iter().flat_map(|r| r.langs.keys());
        codes.map(String::as_str).collect()
    }

    /// `order`'s rows that know `lang`, or all of them for `None`.
    fn ranked(&self, order: &Order, lang: Option<&str>, untested: bool) -> Ranked<'_> {
        let knows = |r: &&Row| lang.is_none_or(|l| r.code(l).is_some());
        let rows = order.rows.iter().filter_map(|&at| self.rows.get(at));
        Ranked {
            rows: rows.filter(knows).collect(),
            tested: order.tested,
            untested,
        }
    }
}

/// The tested orders, each for one language and checked against `rows`.
pub(super) fn orders(
    raw: Vec<RawOrder>,
    rows: &[Row],
) -> Result<BTreeMap<String, Order>, TableError> {
    let mut out = BTreeMap::new();
    for one in raw {
        let fail = |why| TableError::Order {
            lang: one.lang.clone(),
            why,
        };
        if !fits_iso(&one.lang) || out.contains_key(&one.lang) {
            return Err(fail(OrderError::Lang));
        }
        let tested = day(&one.tested).ok_or_else(|| fail(OrderError::Date))?;
        let at = places(&one.models, rows, Some(&one.lang)).map_err(fail)?;
        out.insert(one.lang, Order { tested, rows: at });
    }
    Ok(out)
}

/// The general order, which names every row once and lets no follower head a language without its own list.
pub(super) fn general(
    raw: &RawGeneral,
    rows: &[Row],
    orders: &BTreeMap<String, Order>,
) -> Result<Order, TableError> {
    let tested = day(&raw.tested).ok_or(TableError::General(OrderError::Date))?;
    let at = places(&raw.models, rows, None).map_err(TableError::General)?;
    let langs = rows.iter().flat_map(|r| r.langs.keys());
    let mut untested = langs.filter(|l| !orders.contains_key(*l));
    let led = |lang: &String| {
        let first = at
            .iter()
            .filter_map(|&i| rows.get(i))
            .find(|r| r.code(lang).is_some());
        first.is_some_and(|r| !r.lead)
    };
    if untested.any(led) {
        return Err(TableError::General(OrderError::Lead));
    }
    Ok(Order { tested, rows: at })
}

/// The places of the rows `ids` names: each known, once, and knowing `lang`; all the rows that know it, or every row
/// for `None`; and, for a language, headed by a row that may lead.
fn places(ids: &[String], rows: &[Row], lang: Option<&str>) -> Result<Vec<usize>, OrderError> {
    let knows = |r: &Row| lang.is_none_or(|l| r.code(l).is_some());
    let mut at = Vec::with_capacity(ids.len());
    for id in ids {
        let (place, row) = (rows.iter().enumerate())
            .find(|(_, r)| r.id == *id)
            .ok_or(OrderError::Unknown)?;
        if at.contains(&place) {
            return Err(OrderError::Twice);
        }
        if !knows(row) {
            return Err(OrderError::Stranger);
        }
        at.push(place);
    }
    if at.is_empty() || at.len() != rows.iter().filter(|r| knows(r)).count() {
        return Err(OrderError::Missing);
    }
    let led = at
        .first()
        .and_then(|&i| rows.get(i))
        .is_some_and(|r| !r.lead);
    if lang.is_some() && led {
        return Err(OrderError::Lead);
    }
    Ok(at)
}

/// The plain day `when` names; `None` for a time or an offset, or a day that does not exist.
pub(crate) fn day(when: &Datetime) -> Option<Date> {
    let date = when
        .date
        .filter(|_| when.time.is_none() && when.offset.is_none())?;
    Date::new(date.year, date.month, date.day)
}
