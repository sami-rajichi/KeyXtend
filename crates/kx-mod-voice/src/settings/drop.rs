//! The repair each read of `[voice]` makes (edge case 19, ADR-0047 decision 12): a bad service entry, then a bad pick,
//! goes alone and is named, so the kernel never resets the rest. A cloud pick merely waiting stays (ADR-0054), and
//! nothing moves to a service by itself.

use std::collections::BTreeSet;

use kx_module_api::ports::{BlobName, fits_iso};

use super::{PICKS, SERVICES, VoiceSettings};
use crate::cloud::{Cloud, entry_ok};
use crate::models::{self, Table};
use crate::services::{self, Services};

impl VoiceSettings {
    /// Drops each service entry the services table no longer takes, then each pick that names no row that knows its
    /// language, no model file, and no cloud model Cloud On would serve or show waiting, alone, and names them as
    /// `services.<id>` and `picks.<lang>`. Nothing drops while a table does not load.
    pub fn drop_bad(&mut self) -> Vec<String> {
        let (Ok(models), Ok(services)) = (models::table(), services::table()) else {
            return Vec::new();
        };
        self.drop_bad_with(models, services)
    }

    /// `drop_bad` against these tables, as a later version's would load.
    pub(crate) fn drop_bad_with(&mut self, models: &Table, services: &Services) -> Vec<String> {
        let mut dropped = BTreeSet::new();
        self.services.retain(|id, on| {
            let row = services.get(id).filter(|row| row.offered);
            let good = row.is_some_and(|row| entry_ok(row, on));
            if !good {
                dropped.insert(named(SERVICES, id, BlobName::new(id).is_ok()));
            }
            good
        });
        for lang in self.bad_picks(models, services) {
            self.picks.remove(&lang);
            dropped.insert(named(PICKS, &lang, fits_iso(&lang)));
        }
        dropped.into_iter().collect()
    }

    /// The languages whose pick names no local row or file for them and no cloud model that Cloud On would serve or
    /// show waiting for its service's Confirm; a stale entry, dropped first, takes only its typed picks along.
    fn bad_picks(&self, models: &Table, services: &Services) -> Vec<String> {
        let (cloud, on) = (Cloud::new(services, false), self.cloud_on());
        let good = |lang: &str, pick: &str| match services.pick(pick) {
            Some(_) => cloud.choice(&on, lang).is_some() || cloud.waits(&on, lang).is_some(),
            None => models.pick(lang, pick).is_some(),
        };
        (self.picks.iter())
            .filter(|(lang, pick)| !(fits_iso(lang) && good(lang, pick)))
            .map(|(lang, _)| lang.clone())
            .collect()
    }

    /// These settings with cloud on, as Cloud On would find them.
    pub(crate) fn cloud_on(&self) -> Self {
        Self {
            cloud: true,
            ..self.clone()
        }
    }
}

/// `<key>.<name>` when `name` is plain enough to name, else the whole `key`.
fn named(key: &str, name: &str, plain: bool) -> String {
    if plain {
        format!("{key}.{name}")
    } else {
        key.to_owned()
    }
}
