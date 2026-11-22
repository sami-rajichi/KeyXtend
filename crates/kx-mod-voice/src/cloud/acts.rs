//! The acts that change a language's cloud pick or a service's entry (ADR-0047 decision 12, ADR-0054): the `cloud`
//! switch, a card's confirmation, a service Off or its newer terms declined, a pick, the typed lines, and the card's
//! warning seen. Only `pick` moves a language to a service; Confirm and Cloud On bring back the picks that wait.

use kx_module_api::ports::{CloudAsk, Date};

use super::{Cloud, entry_ok, pick_text};
use crate::services::{Address, Service};
use crate::settings::{ServiceOn, VoiceSettings, datetime};

impl<'a> Cloud<'a> {
    /// Turns cloud services on or off; off keeps every cloud pick waiting while local models serve, and on brings each
    /// back (ADR-0054). A hidden run never turns it on.
    pub fn switch(&self, s: &mut VoiceSettings, on: bool) {
        if on && self.hidden {
            return;
        }
        s.cloud = on;
    }

    /// Switches service `id` on as confirmed on `today` at its row's terms number, keeping its typed lines; false
    /// while cloud is off or for a row not offered. An entry the row no longer takes, as a hand edit leaves, starts
    /// afresh, so only the typed picks its lines held go.
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
            // A stale entry's typed lines go, and with them the typed picks that needed them.
            self.drop_picks(s, |service, model| {
                service == id && row.model(model).is_none()
            });
        }
        true
    }

    /// Switches service `id` Off: its listed picks wait for its Confirm (owner, 2026-11-21), and a typed pick goes with
    /// its typed lines.
    pub fn turn_off(&self, s: &mut VoiceSettings, id: &str) {
        s.services.remove(id);
        let waits: Vec<String> = (s.picks.keys())
            .filter(|lang| self.waits(s, lang).is_some())
            .cloned()
            .collect();
        let its = |p: &String| self.table.pick(p).is_some_and(|(service, _)| service == id);
        s.picks.retain(|l, p| waits.contains(l) || !its(p));
    }

    /// Switches service `id` Off as its newer terms were declined: as a plain Off, its listed picks wait for its
    /// Confirm (ADR-0054).
    pub fn decline(&self, s: &mut VoiceSettings, id: &str) {
        self.turn_off(s, id);
    }

    /// Picks service `id`'s `model` for `lang`, the only act that moves a language to a service; false, changing
    /// nothing, unless the rules then take it and the model can be used now.
    pub fn pick(&self, s: &mut VoiceSettings, lang: &str, id: &str, model: &str) -> bool {
        let choice = self.resolve(s, lang, id, model);
        if choice.is_none_or(|c| c.unavailable.is_some()) {
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
            self.drop_picks(s, |service, typed| service == id && typed == before);
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
            self.drop_picks(s, |service, _| service == id);
        }
        true
    }

    /// Sets service `id`'s keep-nothing switch; a pick it rules out stays, marked unavailable, so nothing moves by
    /// itself (edge 17). False for a service not on or a row without the switch.
    pub fn keep(&self, s: &mut VoiceSettings, id: &str, on: bool) -> bool {
        let Some((_, entry)) = self.entry_mut(s, id).filter(|(row, _)| row.keep.is_some()) else {
            return false;
        };
        entry.keep = on;
        true
    }

    /// Notes that service `id`'s card warning showed at its row's terms number, so the next dictation shows none until
    /// newer terms are confirmed. False for a service not on.
    pub fn warned(&self, s: &mut VoiceSettings, id: &str) -> bool {
        let Some((row, entry)) = self.entry_mut(s, id) else {
            return false;
        };
        entry.warned = row.terms;
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

impl Cloud<'_> {
    /// Drops each cloud pick whose service and model `hit` names.
    fn drop_picks(&self, s: &mut VoiceSettings, hit: impl Fn(&str, &str) -> bool) {
        let kept = |pick: &String| {
            self.table
                .pick(pick)
                .is_none_or(|(id, model)| !hit(id, model))
        };
        s.picks.retain(|_, pick| kept(pick));
    }
}
