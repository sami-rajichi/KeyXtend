//! The voice gate build only: cloud speech that logs, by the service's id, each cloud dictation it wrote or the kind of
//! fault that ended it, and its send and pieces as stages, so G22 can tell which service wrote a language's text. It
//! never logs a key, an address, a model, the sound or the text.

use std::sync::Arc;

use kx_ipc::gate::{FAULT, SERVICE};
use kx_mod_voice::services::{Address, Service, table};
use kx_module_api::MS_PER_S;
use kx_module_api::ports::{
    CloudAsk, CloudFault, CloudNote, CloudRun, CloudSpeech, CloudTest, SAMPLE_BYTES, SOUND_HZ,
};

use super::stage;
use super::told::WROTE_LINE;

/// The message of the line logged as a cloud dictation fails; kx-gates reads it back.
pub const FAILED_LINE: &str = "voice gate build: a cloud dictation failed";

/// What a run's notes are told to.
type Tell = Box<dyn Fn(CloudNote) + Send>;

/// Cloud speech that logs each cloud dictation's service.
pub struct Sent(pub Arc<dyn CloudSpeech>);

impl CloudSpeech for Sent {
    fn send(&self, ask: CloudAsk, tell: Tell) -> Result<Box<dyn CloudRun>, CloudFault> {
        let service = service_of(&ask).unwrap_or_default();
        let sizes = ask.pieces.iter().map(|p| sound_ms(p.expose().len()));
        let lengths: Vec<u32> = sizes.collect();
        stage::stage(stage::SEND);
        let told = Box::new(move |note: CloudNote| {
            heard(&note, service, &lengths);
            tell(note);
        });
        self.0
            .send(ask, told)
            .inspect_err(|fault| failed(service, fault))
    }

    fn test(&self, ask: CloudTest) -> Result<(), CloudFault> {
        self.0.test(ask)
    }
}

/// Logs what `note` tells of a dictation to `service` whose pieces last `lengths` ms.
fn heard(note: &CloudNote, service: &str, lengths: &[u32]) {
    match note {
        CloudNote::Piece { index, .. } => {
            stage::piece(*index, lengths.get(usize::from(*index)).copied());
        }
        CloudNote::Done => tracing::info!({ SERVICE } = %service, "{WROTE_LINE}"),
        CloudNote::Fault(fault) => failed(service, fault),
    }
}

/// Logs that a dictation to `service` failed with `fault`, by its kind alone.
fn failed(service: &str, fault: &CloudFault) {
    tracing::warn!({ SERVICE } = %service, { FAULT } = %fault.kind(), "{FAILED_LINE}");
}

/// The id of the row that made `ask`: a typed row that could have, so Custom is named Custom at any address, else the
/// row at its fixed address that could have.
fn service_of(ask: &CloudAsk) -> Option<&'static str> {
    let rows = table().ok()?.rows();
    let typed = |r: &&Service| matches!(r.address, Address::Typed { .. });
    let mut fit = rows.iter().filter(|r| made(r, ask));
    let first = fit.clone().find(typed);
    first.or_else(|| fit.next()).map(|r| r.id.as_str())
}

/// True when `row` could have made `ask`: its address, header and limits, one of its shapes, and a model it sends.
fn made(row: &Service, ask: &CloudAsk) -> bool {
    let at = row.url(ask.url.expose()).as_ref() == Some(&ask.url);
    let model = match &ask.model {
        None => row.models.iter().any(|m| m.unnamed),
        Some(id) => {
            row.other
                || row
                    .models
                    .iter()
                    .any(|m| !m.unnamed && m.id == *id.expose())
        }
    };
    let listed = row.models.iter().map(Some).chain([None]);
    let mut shapes = listed.flat_map(|m| [false, true].map(|keep| row.shape_for(m, keep)));
    at && model
        && row.header == ask.header
        && row.limits == ask.limits
        && shapes.any(|s| s == ask.shape)
}

/// The length in ms of `bytes` of hand-over sound.
fn sound_ms(bytes: usize) -> u32 {
    let ms = bytes.saturating_mul(usize::try_from(MS_PER_S).unwrap_or(usize::MAX));
    u32::try_from(ms / (SOUND_HZ * SAMPLE_BYTES)).unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests;
