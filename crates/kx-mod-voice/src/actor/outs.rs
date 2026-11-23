//! The flow's effects, carried out in order: the runs, the view and the light, the level, the text for the typer or to
//! paste, Copy, History and the counts, a download, a lost model file, a failed card, the bar's clicks Compare holds, a
//! cloud upload sent or dropped, a card's warning seen, Open service on a cloud note, and the notices the actor tells.

use kx_module_api::ports::{
    BarButton, CardId, ClipPart, Fingerprint, VoiceAct, VoiceBar, VoiceClick, VoiceView,
};
use kx_module_api::{Event, Notice, Redacted, Secret, Zeroizing};

use super::Actor;
use super::lanes::Owner;
use super::typing::Source;
use crate::ID;
use crate::flow::{In, Out};
use crate::settings::SERVICES;

/// The round a bar click the flow hands to Compare names: none, as such a click is never stale.
const NO_ROUND: u32 = 0;

impl Actor {
    /// Carries out one of the flow's effects.
    pub(super) fn out(&mut self, out: Out) {
        match out {
            Out::Open { take, ask } => self.open_run((Owner::Flow, take), ask),
            Out::Run { take, call } => self.call_run((Owner::Flow, take), call),
            Out::View(view) => self.view(&view),
            Out::Lit(lit) => self.publish(&lit),
            Out::Level(level) => self.publish(&level),
            Out::Type(ask) => self.type_text(Source::Flow(ask.round), ask),
            Out::Paste(ask) => self.paste_text(Source::Flow(ask.round), ask),
            Out::Copy(text) => self.copy(&text),
            Out::Keep(kept) => self.keep_text(kept),
            Out::Count(count) => self.count(&count),
            Out::Download(iso) => self.download(&iso),
            Out::Lost => {
                let dos = self.panel.lost();
                self.dos(dos);
            }
            Out::Fell { card, model } => self.fell(&card, &model),
            Out::Lent(button) => self.lent(button),
            Out::Send(up) => self.send(&up),
            Out::Halt { take } => self.halt(take),
            Out::Warned(id) => self.warned(&id),
            Out::Settle { service, key } => self.settle(&service, key),
        }
    }

    /// Tells the view, notes its rows for the panel's clicks, and wants presses while Undo insert can act.
    fn view(&mut self, view: &VoiceView) {
        self.panel.told(&view.0);
        self.publish(view);
        let undo = matches!(&view.0.bar, VoiceBar::Text(text) if text.undo);
        self.want_presses(undo);
    }

    /// Puts the bar's text on the clipboard, and tells the flow whether it went, so a refusal shows on the bar.
    fn copy(&mut self, text: &Secret) {
        let ok = self.clip_text(text.expose());
        self.step(In::Copied(ok));
    }

    /// Puts `text` on the clipboard within `copy_ms`, through a copy wiped after; false, logged by kind, when it could
    /// not.
    pub(super) fn clip_text(&mut self, text: &str) -> bool {
        let parts = [ClipPart::Text(Redacted::new(text.to_owned()))];
        let written = self
            .ports
            .clip
            .write(&parts, self.held.settings.copy_wait());
        for part in parts {
            if let ClipPart::Text(copy) = part {
                drop(Zeroizing::new(copy.into_inner()));
            }
        }
        written
            .inspect_err(|error| tracing::warn!(%error, "a dictated text could not be copied"))
            .is_ok()
    }

    /// Downloads language `iso`'s model: the panel opens on Models with the drives read now.
    fn download(&mut self, iso: &str) {
        self.held.drives = self.ports.files.drives();
        let facts = self.held.facts(self.ports.calendar.now(), &self.flow);
        let dos = self.panel.download_for(iso, &facts);
        self.dos(dos);
    }

    /// Open service on a cloud note: the panel opens on the service's card or key page, with the keys read now.
    fn settle(&mut self, service: &str, key: bool) {
        self.read_keyed();
        let facts = self.held.facts(self.ports.calendar.now(), &self.flow);
        let dos = self.panel.settle(service, key, &facts);
        self.dos(dos);
    }

    /// A card failed mid-run: its check of the model file is dropped, so the check runs again.
    pub(super) fn fell(&mut self, card: &CardId, model: &Fingerprint) {
        if self.held.checks.fell(card, model) {
            self.save_checks();
        }
    }

    /// The bar's Stop or Cancel while Compare holds the run, as a click on the panel.
    fn lent(&mut self, button: BarButton) {
        let act = VoiceAct {
            round: NO_ROUND,
            click: VoiceClick::Bar(button),
        };
        let facts = self.held.facts(self.ports.calendar.now(), &self.flow);
        let dos = self.panel.click(&act, &facts);
        self.dos(dos);
    }

    /// Saves service `id`'s card warning as seen at its row's terms number, so the next dictation shows none.
    fn warned(&mut self, id: &str) {
        let Some(rules) = self.held.cloud() else {
            return;
        };
        let mut next = self.held.settings.clone();
        if !rules.warned(&mut next, id) {
            return;
        }
        if let Some(value) = next.services_value() {
            self.set(SERVICES, value);
        }
    }

    /// Tells the user `key`.
    pub(super) fn say(&self, key: &'static str) {
        self.publish(&Notice::about(key, ID));
    }

    /// Publishes `event`; a refusal is the bus's gate on a capability the module holds, so it never happens.
    pub(super) fn publish<E: Event>(&self, event: &E) {
        let _ = self.ports.bus.publish(event);
    }
}
