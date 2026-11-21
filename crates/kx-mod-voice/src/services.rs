//! The compiled-in services table (ADR-0047 decisions 5 to 9), from its two files: each cloud service's request, host,
//! limits, models and dated facts, read and checked once. A bad table turns cloud services off; it never panics, and
//! nothing in it is a default, so a service stays off until the user confirms it.

mod address;
mod facts;
mod form;
mod limits;
mod model;
mod row;

pub use address::{Address, KeyTest};
pub use facts::{FactError, Facts};
pub use limits::piece_s;
pub use model::{Model, ModelError};
pub use row::{Service, ServiceError};

use std::collections::BTreeMap;
use std::sync::LazyLock;

use kx_module_api::ports::BlobName;
use serde::Deserialize;
use thiserror::Error;

use crate::cloud::PICK_MARK;

/// Bytes in the KiB `room_kb` counts in.
const KIB: u64 = 1 << 10;

/// The table's two files and its language sets, read and checked at their first use.
static TABLE: LazyLock<Result<Services, ServicesError>> = LazyLock::new(|| {
    let own = include_str!("../services-own.toml");
    let langs = include_str!("../langs.toml");
    Services::parse(include_str!("../services.toml"), own, langs)
});

/// The compiled-in services table, read and checked once.
///
/// # Errors
/// The table's first fault; cloud services are then off.
pub fn table() -> Result<&'static Services, &'static ServicesError> {
    TABLE.as_ref()
}

/// Why a services table was refused, naming the row and its rule.
#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum ServicesError {
    /// The text is not a services table.
    #[error("the services table does not parse: {0}")]
    Parse(String),
    /// The second file's text is not rows alone, as a table-wide key there.
    #[error("the services table's own-shape rows do not parse: {0}")]
    Own(String),
    /// The language sets' text is not a list of named sets.
    #[error("the language sets do not parse: {0}")]
    Langs(String),
    /// It lists no service.
    #[error("the services table lists no service")]
    Empty,
    /// A row's id is not a blob name, which its key's blob takes, or is another row's.
    #[error("the service id {0} is not a plain blob name or is listed twice")]
    Id(String),
    /// The light hours are not fewer than the heavy ones, or the room or the model count is zero.
    #[error("the table's hours, room or model count are out of order")]
    Table,
    /// A row breaks a rule.
    #[error("the service {id} is refused: {why}")]
    Row {
        /// The row's id.
        id: String,
        /// The rule it breaks.
        why: ServiceError,
    },
}

/// The table as written.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Raw {
    /// Hours of sound a month of light use.
    light_hours: u16,
    /// Hours of sound a month of heavy use.
    heavy_hours: u16,
    /// The KiB kept under a service's cap for the WAV header and the form's other parts.
    room_kb: u32,
    /// The most models a row offers.
    max_models: usize,
    /// What a form may ask about its reply.
    reply: form::Reply,
    /// The rows.
    services: Vec<row::RawService>,
}

/// The second file as written: the rows of services with their own shape, and nothing else.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawOwn {
    /// The rows.
    #[serde(default)]
    services: Vec<row::RawService>,
}

/// Each keyboard language (ISO 639) and the code a service wants for it.
pub type LangCodes = BTreeMap<String, String>;

/// What every row is checked against.
struct Rules<'a> {
    /// Bytes kept under a service's cap for the WAV header and the form's other parts.
    room: u64,
    /// The most models a row offers.
    max_models: usize,
    /// What a form may ask about its reply.
    reply: &'a form::Reply,
    /// The table's language sets by name.
    sets: &'a BTreeMap<String, LangCodes>,
}

/// The checked table: its rows in the Cloud page's order, and the hours of light and heavy use the cards price.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Services {
    /// The rows, Groq first.
    rows: Vec<Service>,
    /// Hours of sound a month of light use.
    light_hours: u16,
    /// Hours of sound a month of heavy use.
    heavy_hours: u16,
}

impl Services {
    /// Reads and checks a services table: `text`'s values and rows, then `own`'s rows of their own shape, as one table
    /// whose models may name the language sets in `langs`.
    ///
    /// # Errors
    /// The first fault found.
    pub fn parse(text: &str, own: &str, langs: &str) -> Result<Self, ServicesError> {
        let raw: Raw = toml::from_str(text).map_err(|e| ServicesError::Parse(e.to_string()))?;
        let own: RawOwn = toml::from_str(own).map_err(|e| ServicesError::Own(e.to_string()))?;
        let sets: BTreeMap<String, LangCodes> =
            toml::from_str(langs).map_err(|e| ServicesError::Langs(e.to_string()))?;
        let room = u64::from(raw.room_kb) * KIB;
        let ordered = raw.light_hours > 0 && raw.light_hours < raw.heavy_hours;
        if !ordered || room == 0 || raw.max_models == 0 {
            return Err(ServicesError::Table);
        }
        let rules = Rules {
            room,
            max_models: raw.max_models,
            reply: &raw.reply,
            sets: &sets,
        };
        let all = raw.services.into_iter().chain(own.services).collect();
        let rows = checked_rows(all, &rules)?;
        Ok(Self {
            rows,
            light_hours: raw.light_hours,
            heavy_hours: raw.heavy_hours,
        })
    }

    /// The rows, in the Cloud page's order.
    #[must_use]
    pub fn rows(&self) -> &[Service] {
        &self.rows
    }

    /// The row `id` names.
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&Service> {
        self.rows.iter().find(|r| r.id == id)
    }

    /// The service id and model id cloud pick `value` names, parted at the first `PICK_MARK` only when a row id of
    /// this table comes before it, so a local file whose name holds the mark stays local (P15).
    #[must_use]
    pub fn pick<'v>(&self, value: &'v str) -> Option<(&'v str, &'v str)> {
        value
            .split_once(PICK_MARK)
            .filter(|(id, _)| self.get(id).is_some())
    }

    /// The row, the model id and, when the row lists it, the listed model that cloud pick `value` names.
    #[must_use]
    pub fn pick_row<'v>(&self, value: &'v str) -> Option<(&Service, &'v str, Option<&Model>)> {
        let (id, model) = self.pick(value)?;
        let row = self.get(id)?;
        Some((row, model, row.model(model)))
    }

    /// The rows the Cloud page may show, in order, Groq first (edge 24).
    pub fn offered(&self) -> impl Iterator<Item = &Service> {
        self.rows.iter().filter(|r| r.offered)
    }

    /// The hours of sound a month of light use.
    #[must_use]
    pub fn light_hours(&self) -> u16 {
        self.light_hours
    }

    /// The hours of sound a month of heavy use.
    #[must_use]
    pub fn heavy_hours(&self) -> u16 {
        self.heavy_hours
    }
}

/// The raw rows, each checked and with an id that names a blob and no other row has.
fn checked_rows(
    raw: Vec<row::RawService>,
    rules: &Rules<'_>,
) -> Result<Vec<Service>, ServicesError> {
    if raw.is_empty() {
        return Err(ServicesError::Empty);
    }
    let mut rows: Vec<Service> = Vec::with_capacity(raw.len());
    for one in raw {
        let id = one.id.clone();
        if BlobName::new(&id).is_err() || rows.iter().any(|r| r.id == id) {
            return Err(ServicesError::Id(id));
        }
        // Only the one Custom row may go unnamed, so a second is refused by the name rule.
        let unnamed = rows.iter().any(|r| r.name.is_none());
        let row = Service::checked(one, rules).and_then(|r| {
            (r.name.is_some() || !unnamed)
                .then_some(r)
                .ok_or(ServiceError::Name)
        });
        rows.push(row.map_err(|why| ServicesError::Row { id, why })?);
    }
    Ok(rows)
}

#[cfg(test)]
pub(crate) mod tests;
