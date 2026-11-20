//! The voice gate build only: cloud speech that logs, by the service's id, each cloud dictation it wrote or the kind of
//! fault that ended it, and its send and pieces as stages, so G22 can tell which service wrote a language's text. It
//! never logs a key, an address, a model, the sound or the text.

use std::sync::Arc;

use kx_mod_voice::services::{Address, table};
use kx_module_api::MS_PER_S;
use kx_module_api::ports::{
    CloudAsk, CloudFault, CloudNote, CloudRun, CloudSpeech, CloudTest, HttpsUrl, SAMPLE_BYTES,
    SOUND_HZ,
};

use super::stage;
use super::told::WROTE_LINE;

/// The message of the line logged as a cloud dictation fails; kx-gates reads it back.
pub const FAILED_LINE: &str = "voice gate build: a cloud dictation failed";
/// The field that holds the service's id.
#[cfg(test)]
pub const SERVICE: &str = "service";
/// The field that holds a fault's kind.
#[cfg(test)]
pub const FAULT: &str = "fault";

/// What a run's notes are told to.
type Tell = Box<dyn Fn(CloudNote) + Send>;

/// Cloud speech that logs each cloud dictation's service.
pub struct Sent(pub Arc<dyn CloudSpeech>);

impl CloudSpeech for Sent {
    fn send(&self, ask: CloudAsk, tell: Tell) -> Result<Box<dyn CloudRun>, CloudFault> {
        let service = service_of(&ask.url).unwrap_or_default();
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
        CloudNote::Done => tracing::info!(service = %service, "{WROTE_LINE}"),
        CloudNote::Fault(fault) => failed(service, fault),
    }
}

/// Logs that a dictation to `service` failed with `fault`, by its kind alone.
fn failed(service: &str, fault: &CloudFault) {
    tracing::warn!(service = %service, fault = %kind(fault), "{FAILED_LINE}");
}

/// The id of the row whose requests go to `url`: a fixed address first, else a typed row that takes it.
fn service_of(url: &HttpsUrl) -> Option<&'static str> {
    let rows = table().ok()?.rows();
    let fixed = rows
        .iter()
        .find(|r| r.address == Address::Fixed(url.clone()));
    let takes = |r: &&kx_mod_voice::services::Service| {
        matches!(r.address, Address::Typed { .. }) && r.url(url.expose()).as_ref() == Some(url)
    };
    fixed
        .or_else(|| rows.iter().find(takes))
        .map(|r| r.id.as_str())
}

/// A fault's kind: its variant's name alone, never a value it holds.
fn kind(fault: &CloudFault) -> String {
    let shown = format!("{fault:?}");
    shown
        .chars()
        .take_while(char::is_ascii_alphanumeric)
        .collect()
}

/// The length in ms of `bytes` of hand-over sound.
fn sound_ms(bytes: usize) -> u32 {
    let ms = bytes.saturating_mul(usize::try_from(MS_PER_S).unwrap_or(usize::MAX));
    u32::try_from(ms / (SOUND_HZ * SAMPLE_BYTES)).unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests;
