//! The compiled-in models table (ADR-0043): each row's file, download, languages and memory, the tested order per
//! language and the general order, read and checked once. A bad table turns local models off; it never panics.

mod order;
mod other;
mod pick;
mod row;

pub(crate) use order::day;
pub use order::{OrderError, Ranked};
pub use other::{Other, code_in};
pub use pick::Pick;
pub(crate) use row::one_line;
pub use row::{Row, RowError};

use std::collections::BTreeMap;
use std::sync::LazyLock;

use kx_module_api::is_hidden;
use kx_module_api::ports::{Fingerprint, names_device};
use serde::Deserialize;
use thiserror::Error;

/// The marks a model file's name may not hold: either separator and the colon, which names a drive or a stream.
const PATH_MARKS: [char; 3] = ['/', '\\', ':'];

/// The table, read and checked at its first use.
static TABLE: LazyLock<Result<Table, TableError>> =
    LazyLock::new(|| Table::parse(include_str!("../models.toml")));

/// The compiled-in models table, read and checked once.
///
/// # Errors
/// The table's first fault; local models are then off.
pub fn table() -> Result<&'static Table, &'static TableError> {
    TABLE.as_ref()
}

/// Why a models table was refused.
#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum TableError {
    /// The text is not a models table.
    #[error("the models table does not parse: {0}")]
    Parse(String),
    /// It lists no model.
    #[error("the models table lists no model")]
    Empty,
    /// A row's id is not a plain name, or another row's.
    #[error("the row id {0} is not a plain name or is listed twice")]
    Id(String),
    /// A row breaks a rule.
    #[error("the row {id} is refused: {why}")]
    Row {
        /// The row's id.
        id: String,
        /// The rule it breaks.
        why: RowError,
    },
    /// A language's order breaks a rule.
    #[error("the order for {lang} is refused: {why}")]
    Order {
        /// The order's language.
        lang: String,
        /// The rule it breaks.
        why: OrderError,
    },
    /// The general order breaks a rule.
    #[error("the general order is refused: {0}")]
    General(OrderError),
}

/// The table as written.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Raw {
    /// The extensions a model file may have.
    exts: Vec<String>,
    /// The licences a row may name.
    licences: Vec<String>,
    /// The runners a row may name.
    runners: Vec<String>,
    /// The hex digits of the commit each address names.
    revision: usize,
    /// The rows.
    models: Vec<row::RawRow>,
    /// The tested orders.
    orders: Vec<order::RawOrder>,
    /// The general order.
    general: order::RawGeneral,
}

/// What every row is checked against.
struct Rules<'a> {
    /// The extensions a model file may have.
    exts: &'a [String],
    /// The licences a row may name.
    licences: &'a [String],
    /// The runners a row may name.
    runners: &'a [String],
    /// The hex digits of the commit each address names.
    revision: usize,
}

/// The checked table: its rows, the file extensions, the tested orders by language and the general order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Table {
    /// The rows, in the table's order.
    rows: Vec<Row>,
    /// The extensions a model file may have.
    exts: Vec<String>,
    /// Each tested language's order.
    orders: BTreeMap<String, order::Order>,
    /// The order for a language without its own.
    general: order::Order,
}

impl Table {
    /// Reads and checks a models table.
    ///
    /// # Errors
    /// The first fault found.
    pub fn parse(text: &str) -> Result<Self, TableError> {
        let raw: Raw = toml::from_str(text).map_err(|e| TableError::Parse(e.to_string()))?;
        let rules = Rules {
            exts: &raw.exts,
            licences: &raw.licences,
            runners: &raw.runners,
            revision: raw.revision,
        };
        let rows = checked_rows(raw.models, &rules)?;
        let orders = order::orders(raw.orders, &rows)?;
        let general = order::general(&raw.general, &rows, &orders)?;
        Ok(Self {
            rows,
            exts: raw.exts,
            orders,
            general,
        })
    }

    /// The rows, in the table's order.
    #[must_use]
    pub fn rows(&self) -> &[Row] {
        &self.rows
    }

    /// The row `id` names.
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&Row> {
        self.rows.iter().find(|r| r.id == id)
    }

    /// The row whose file is `name`, in any case, as the system's file names are.
    #[must_use]
    pub fn named(&self, name: &str) -> Option<&Row> {
        self.rows.iter().find(|r| r.file.eq_ignore_ascii_case(name))
    }

    /// The row whose file has fingerprint `print`, so a renamed copy is known.
    #[must_use]
    pub fn printed(&self, print: &Fingerprint) -> Option<&Row> {
        self.rows.iter().find(|r| r.print == *print)
    }

    /// The extensions a model file may have, which the folder scan asks for.
    #[must_use]
    pub fn exts(&self) -> &[String] {
        &self.exts
    }

    /// True when `name` ends in a model file's extension, in any case, after a stem.
    #[must_use]
    pub fn is_model_file(&self, name: &str) -> bool {
        has_ext(name, &self.exts)
    }
}

/// The raw rows, each checked and with an id no other row has.
fn checked_rows(raw: Vec<row::RawRow>, rules: &Rules<'_>) -> Result<Vec<Row>, TableError> {
    if raw.is_empty() {
        return Err(TableError::Empty);
    }
    let mut rows: Vec<Row> = Vec::with_capacity(raw.len());
    for one in raw {
        let id = one.id.clone();
        if !id_ok(&id) || rows.iter().any(|r| r.id == id) {
            return Err(TableError::Id(id));
        }
        rows.push(Row::checked(one, rules).map_err(|why| TableError::Row { id, why })?);
    }
    Ok(rows)
}

/// True for a row id: lower-case ASCII letters, digits, `-` and `.`; the store reads counts by it too.
pub(crate) fn id_ok(id: &str) -> bool {
    let plain = |b: u8| b.is_ascii_lowercase() || b.is_ascii_digit() || b"-.".contains(&b);
    !id.is_empty() && id.bytes().all(plain)
}

/// True when `name` has a stem and one of `exts`, in any case.
fn has_ext(name: &str, exts: &[String]) -> bool {
    name.rsplit_once('.').is_some_and(|(stem, ext)| {
        !stem.is_empty() && exts.iter().any(|e| e.eq_ignore_ascii_case(ext))
    })
}

/// True for a model file's name in a folder: a model extension, no separator, colon or hidden mark, and no device
/// name as its stem (`names_device`), which the system would open as the device.
pub(crate) fn file_ok(name: &str, exts: &[String]) -> bool {
    let odd = name
        .chars()
        .any(|c| is_hidden(c) || PATH_MARKS.contains(&c));
    has_ext(name, exts) && !odd && !names_device(name)
}

#[cfg(test)]
mod tests;
