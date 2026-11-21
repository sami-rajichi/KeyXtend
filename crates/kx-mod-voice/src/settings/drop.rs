//! The repair each read of `[voice]` makes (edge case 19, ADR-0047 decision 12): a bad service entry, then a bad pick,
//! goes alone and is named, so the kernel never resets the rest, and a later table or switch never brings a cloud pick
//! back by itself.

use std::collections::BTreeSet;

use kx_module_api::ports::{BlobName, fits_iso};

use super::{PICKS, SERVICES, VoiceSettings};
use crate::cloud::{Cloud, entry_ok};
use crate::models::{self, Table};
use crate::services::{self, Services};

impl VoiceSettings {
    /// Drops each service entry the services table no longer takes, then each pick that names no row that knows its
    /// language, no model file, and no service switched on with a model for it while `cloud` is on, alone, and names
    /// them as `services.<id>` and `picks.<lang>`. Nothing drops while a table does not load.
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
        let cloud = Cloud::new(services, false);
        let bad: Vec<String> = self
            .picks
            .iter()
            .filter(|(lang, pick)| {
                let good = match services.pick(pick) {
                    Some(_) => cloud.choice(self, lang).is_some(),
                    None => models.pick(lang, pick).is_some(),
                };
                !(fits_iso(lang) && good)
            })
            .map(|(lang, _)| lang.clone())
            .collect();
        for lang in bad {
            self.picks.remove(&lang);
            dropped.insert(named(PICKS, &lang, fits_iso(&lang)));
        }
        dropped.into_iter().collect()
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
