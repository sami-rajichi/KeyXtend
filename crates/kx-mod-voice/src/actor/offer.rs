//! The flow's `Supply` in the actor: a language's local model from the panel's `Stock`, and the cloud service its pick
//! names, with whether a key is set from the blobs' names alone. A hidden run offers no service and reads no key.

use kx_module_api::Redacted;
use kx_module_api::ports::{CloudLimits, SAMPLE_BYTES, SOUND_HZ, SpeechAsk};

use crate::VoiceSettings;
use crate::cloud::{Chosen, Cloud, pick_text};
use crate::flow::{Lack, Net, Ready, Supply};
use crate::folder::Stock;
use crate::keys::Keys;
use crate::services;

/// What a language gets now.
pub(super) struct Offer<'a> {
    /// The local models.
    pub stock: Stock<'a>,
    /// `[voice]`: `cloud`, `services` and the picks.
    pub settings: &'a VoiceSettings,
    /// The keys on the voice shelf.
    pub keys: &'a Keys,
    /// The run's saves would be hidden (ADR-0037), so no service is offered.
    pub hidden: bool,
}

impl Supply for Offer<'_> {
    fn supply(&self, iso: &str) -> Result<Ready, Lack> {
        self.stock.supply(iso)
    }

    fn cloud(&self, iso: &str) -> Option<Net> {
        let table = services::table().ok()?;
        let choice = Cloud::new(table, self.hidden).choice(self.settings, iso)?;
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
        })
    }
}

/// The longest piece a listen-only run cuts for a row's `limits`: the whole seconds its size and time caps both
/// allow (`piece_bytes`), within `SpeechAsk::PIECE_S`.
pub(super) fn piece_s(limits: &CloudLimits) -> u8 {
    let (low, high) = (*SpeechAsk::PIECE_S.start(), *SpeechAsk::PIECE_S.end());
    let secs = limits.piece_bytes() / (SOUND_HZ * SAMPLE_BYTES);
    u8::try_from(secs).map_or(high, |s| s.clamp(low, high))
}
