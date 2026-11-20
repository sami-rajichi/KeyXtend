//! The flow's `Supply` in the actor: a language's local model from the panel's `Stock`, and the cloud service its pick
//! names, with whether a key is set from the blobs' names alone. A hidden run offers no service and reads no key.

use kx_module_api::Redacted;

use crate::VoiceSettings;
use crate::cloud::{Chosen, Cloud, pick_text};
use crate::flow::{Lack, Net, Ready, Supply};
use crate::folder::Stock;
use crate::keys::Keys;
use crate::services::piece_s;

/// What a language gets now.
pub(super) struct Offer<'a> {
    /// The local models.
    pub stock: Stock<'a>,
    /// `[voice]`: `cloud`, `services` and the picks.
    pub settings: &'a VoiceSettings,
    /// The keys on the voice shelf.
    pub keys: &'a Keys,
    /// The cloud rules, which offer no service in a hidden run (ADR-0037); `None` should the table fail.
    pub cloud: Option<Cloud<'a>>,
}

impl Supply for Offer<'_> {
    fn supply(&self, iso: &str) -> Result<Ready, Lack> {
        self.stock.supply(iso)
    }

    fn cloud(&self, iso: &str) -> Option<Net> {
        let choice = self.cloud?.choice(self.settings, iso)?;
        let row = choice.service;
        let model = match choice.model {
            Chosen::Listed(model) => model.id.as_str(),
            Chosen::Typed(id) => id,
        };
        let has = self.keys.has(&row.id);
        let has =
            has.inspect_err(|error| tracing::warn!(%error, "the cloud keys could not be listed"));
        Some(Net {
            service: row.id.clone(),
            name: row.shown(),
            model: Redacted::new(pick_text(&row.id, model)),
            piece_s: piece_s(&row.limits),
            key: has.unwrap_or(false),
            terms: choice.ask,
            unavailable: choice.unavailable,
            warn: choice.warn,
        })
    }
}
