//! The cloud keys of `[voice]` (ADR-0047 decisions 9 and 12): the switch, each service the user switched on with the
//! day and terms number confirmed, the request's wait and tries in the frames' own ranges, and the facts' age.

use std::collections::BTreeMap;
use std::fmt;
use std::ops::RangeInclusive;

use kx_module_api::ports::{CloudAsk, Date};
use kx_module_api::{Redacted, SettingsError, within};
use serde::{Deserialize, Serialize};
use toml::value::Datetime;

use super::{SERVICES, VoiceSettings};
use crate::models::day;

/// Days after which a card's facts show as old.
pub const FACTS_DAYS: RangeInclusive<u16> = 30..=730;
/// `OpenRouter`'s keep-nothing switch starts on (design "Connections and services").
pub const KEEP_START: bool = true;

/// A service the user switched on: the day and terms number confirmed, its typed address and Other model, its
/// keep-nothing switch, and the terms number its card warning was seen at.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServiceOn {
    /// The day the user confirmed the card.
    pub on: Datetime,
    /// The terms number confirmed; a row with a newer one asks again before the next upload.
    pub terms: u16,
    /// The typed address, for a typed row such as Custom; empty for none, which the file leaves out.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub address: String,
    /// The Other model line's typed id; empty for none, which the file leaves out.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub model: String,
    /// The keep-nothing switch, for a row that has one.
    #[serde(default = "keep_start")]
    pub keep: bool,
    /// The terms number at which the first dictation showed the card's warning; 0 for never, which the file leaves out.
    #[serde(default, skip_serializing_if = "unseen")]
    pub warned: u16,
}

impl ServiceOn {
    /// A service confirmed on `day` at `terms`, with no typed line and the keep-nothing switch at its start.
    #[must_use]
    pub fn new(day: Date, terms: u16) -> Self {
        Self {
            on: datetime(day),
            terms,
            address: String::new(),
            model: String::new(),
            keep: KEEP_START,
            warned: 0,
        }
    }

    /// The day confirmed; `None` for a time or an offset, which a hand edit may write.
    #[must_use]
    pub fn day(&self) -> Option<Date> {
        day(&self.on)
    }
}

/// Shows the day, the terms number and the switch; the typed address and model are the user's own.
impl fmt::Debug for ServiceOn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ServiceOn")
            .field("on", &self.on)
            .field("terms", &self.terms)
            .field("address", &Redacted::new(&self.address))
            .field("model", &Redacted::new(&self.model))
            .field("keep", &self.keep)
            .field("warned", &self.warned)
            .finish()
    }
}

impl VoiceSettings {
    /// The cloud ranges: the wait and tries are the frames' own (`CloudAsk`), the facts' age the module's.
    pub(super) fn check_cloud(&self) -> Result<(), SettingsError> {
        within("net_wait_s", self.net_wait_s, &CloudAsk::WAIT_S)?;
        within("net_tries", self.net_tries, &CloudAsk::TRIES)?;
        within("facts_days", self.facts_days, &FACTS_DAYS)
    }

    /// `services` as a settings write takes it, each day a plain date; it goes through the file's text, as
    /// `toml::Value::try_from` would keep a day as toml's private table.
    #[must_use]
    pub fn services_value(&self) -> Option<toml::Value> {
        let text = toml::to_string(&BTreeMap::from([(SERVICES, &self.services)])).ok()?;
        toml::from_str::<toml::Table>(&text).ok()?.remove(SERVICES)
    }
}

/// `day` as the settings file writes it, a plain date.
pub(crate) fn datetime(day: Date) -> Datetime {
    Datetime {
        date: Some(toml::value::Date {
            year: day.year(),
            month: day.month(),
            day: day.day(),
        }),
        time: None,
        offset: None,
    }
}

/// The keep-nothing switch's start, for an entry written without it.
fn keep_start() -> bool {
    KEEP_START
}

/// True when no warning was shown yet, so the file leaves the number out.
#[expect(
    clippy::trivially_copy_pass_by_ref,
    reason = "serde passes a reference"
)]
fn unseen(terms: &u16) -> bool {
    *terms == 0
}
