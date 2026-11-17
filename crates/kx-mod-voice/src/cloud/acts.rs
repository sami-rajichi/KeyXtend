//! The acts that change a language's cloud pick or a service's entry (ADR-0047 decisions 9 and 12): the `cloud` switch,
//! a card's confirmation, a service off or its newer terms declined, a pick, and the typed lines. Only `pick` moves a
//! language to a service.

use kx_module_api::ports::{CloudAsk, Date};

use super::{Cloud, cloud_pick, entry_ok, pick_text};
use crate::services::{Address, Service};
use crate::settings::{ServiceOn, VoiceSettings, datetime};

impl<'a> Cloud<'a> {
    /// Turns cloud services on or off; off drops every cloud pick, so each language goes back to its local model and
    /// turning cloud on again brings none back (edge 21). A hidden run never turns it on.
    pub fn switch(&self, s: &mut VoiceSettings, on: bool) {
        if on && self.hidden {
            return;
        }
        s.cloud = on;
        if !on {
            s.picks.retain(|_, pick| cloud_pick(pick).is_none());
        }
    }

    /// Switches service `id` on as confirmed on `today` at its row's terms number, keeping its typed lines; false
    /// while cloud is off or for a row not offered. An entry the row no longer takes, as a hand edit leaves, starts
    /// afresh, and the picks that named its service go, so none comes back with it.
    pub fn confirm(&self, s: &mut VoiceSettings, id: &str, today: Date) -> bool {
        let Some(row) = self.table.get(id).filter(|row| row.offered && self.on(s)) else {
            return false;
        };
        let fresh = ServiceOn::new(today, row.terms);
        let entry = s
            .services
            .entry(id.to_owned())
            .or_insert_with(|| fresh.clone());
        let stale = !entry_ok(row, entry);
        if stale {
            *entry = fresh;
        }
        entry.on = datetime(today);
        entry.terms = row.terms;
        if stale {
            // A stale entry's typed lines go, and with them the picks that needed them.
            drop_picks(s, |service, _| service == id);
        }
        true
    }

    /// Switches service `id` off and drops its picks, as a declined newer terms number does (edge 16).
    pub fn turn_off(&self, s: &mut VoiceSettings, id: &str) {
        s.services.remove(id);
        drop_picks(s, |service, _| service == id);
    }

    /// Picks service `id`'s `model` for `lang`, the only act that moves a language to a service; false, changing
    /// nothing, unless the rules then take it.
    pub fn pick(&self, s: &mut VoiceSettings, lang: &str, id: &str, model: &str) -> bool {
        if self.resolve(s, lang, id, model).is_none() {
            return false;
        }
        s.picks.insert(lang.to_owned(), pick_text(id, model));
        true
    }

    /// Keeps `model` as service `id`'s typed Other model, empty to clear it; the picks of the id before go, so no
    /// language follows a new id by itself. False for a service not on, a row without the line, or an odd id.
    pub fn type_model(&self, s: &mut VoiceSettings, id: &str, model: &str) -> bool {
        let fits = model.is_empty() || CloudAsk::model_fits(model);
        let Some((row, entry)) = self.entry_mut(s, id).filter(|(row, _)| row.other && fits) else {
            return false;
        };
        let before = std::mem::replace(&mut entry.model, model.to_owned());
        if before != model && row.model(&before).is_none() {
            drop_picks(s, |service, typed| service == id && typed == before);
        }
        true
    }

    /// Keeps `text` as service `id`'s typed address, empty to clear it, which drops its picks so none comes back when
    /// one is typed again. False for a service not on, a row with its own address, or an address that is not HTTPS on
    /// a host name the row takes (edge 13).
    pub fn type_address(&self, s: &mut VoiceSettings, id: &str, text: &str) -> bool {
        let takes = |row: &Service| {
            matches!(row.address, Address::Typed { .. })
                && (text.is_empty() || row.url(text).is_some())
        };
        let Some((_, entry)) = self.entry_mut(s, id).filter(|(row, _)| takes(row)) else {
            return false;
        };
        text.clone_into(&mut entry.address);
        if text.is_empty() {
            drop_picks(s, |service, _| service == id);
        }
        true
    }

    /// Sets service `id`'s keep-nothing switch; on drops the picks of its models that every host keeps, so none waits
    /// to come back (edge 17). False for a service not on or a row without the switch.
    pub fn keep(&self, s: &mut VoiceSettings, id: &str, on: bool) -> bool {
        let Some((row, entry)) = self.entry_mut(s, id).filter(|(row, _)| row.keep.is_some()) else {
            return false;
        };
        entry.keep = on;
        let kept = |model: &str| row.model(model).is_some_and(|m| m.keeps);
        if on {
            drop_picks(s, |service, model| service == id && kept(model));
        }
        true
    }

    /// Row `id` and its entry, while the service is on.
    fn entry_mut<'s>(
        &self,
        s: &'s mut VoiceSettings,
        id: &str,
    ) -> Option<(&'a Service, &'s mut ServiceOn)> {
        let row = self.table.get(id)?;
        self.entry(s, row)?;
        Some((row, s.services.get_mut(id)?))
    }
}

/// Drops each cloud pick whose service and model `hit` names.
fn drop_picks(s: &mut VoiceSettings, hit: impl Fn(&str, &str) -> bool) {
    s.picks
        .retain(|_, pick| cloud_pick(pick).is_none_or(|(service, model)| !hit(service, model)));
}
