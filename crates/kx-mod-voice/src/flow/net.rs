//! A cloud dictation's parts (ADR-0047): the service and model a language picked, what an upload tells, the upload the
//! actor carries out, and the bar's line for each cloud fault.

use kx_module_api::ports::{
    CloudFault, CloudNote, CloudWhy, Fact, LocalTime, LocalUse, ServiceName, SpeechDone,
    SpeechError, Unavailable, VoiceLang,
};
use kx_module_api::{Mono, Redacted};
use std::time::Duration;

use super::{Lack, Ready};

/// What writes a dictation: a model on this PC, or a cloud service.
#[derive(Clone, Debug)]
pub(super) enum Engine {
    /// A local model and device.
    Local(Ready),
    /// A cloud service, sent the speech a listen-only run handed over.
    Cloud(Net),
}

impl Engine {
    /// The name History and the counts keep: the local model's, or the pick.
    pub(super) fn name(&self) -> Redacted<String> {
        match self {
            Self::Local(ready) => ready.name.clone(),
            Self::Cloud(net) => net.model.clone(),
        }
    }

    /// The service, for a cloud dictation.
    pub(super) fn net(&self) -> Option<&Net> {
        match self {
            Self::Cloud(net) => Some(net),
            Self::Local(_) => None,
        }
    }

    /// The local model, for a local dictation.
    pub(super) fn ready(&self) -> Option<&Ready> {
        match self {
            Self::Local(ready) => Some(ready),
            Self::Cloud(_) => None,
        }
    }
}

/// The cloud side of a dictation opened listen-only, from Start until its bar closes.
#[derive(Clone, Debug, Default)]
pub(super) struct Wire {
    /// The run's end once it handed the speech over; nothing is sent before.
    pub(super) spoken: Option<SpeechDone>,
    /// The last upload's round.
    pub(super) round: u32,
    /// An upload runs, so its notes count.
    pub(super) sending: bool,
    /// When Stop or the last Try again came, which the counts' wait runs from.
    pub(super) since: Mono,
}

/// A cloud service and model that serve a language now, as the user picked them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Net {
    /// The service's row id, which the upload names.
    pub service: String,
    /// The service as the bar's cloud mark names it.
    pub name: ServiceName,
    /// The pick, `service:model`, which History and the counts keep.
    pub model: Redacted<String>,
    /// The longest piece the row takes, in seconds, within `SpeechAsk::PIECE_S`.
    pub piece_s: u8,
    /// A key is set for the service; a damaged one shows only when it is read for the upload.
    pub key: bool,
    /// The row's terms changed since they were confirmed, so nothing is sent until they are confirmed again.
    pub terms: bool,
    /// Why the pick's model cannot be used now, which Start says, sending nothing; `None` when it can.
    pub unavailable: Option<Unavailable>,
    /// The card's warning, which the bar shows while this dictation listens, as none at this terms number did.
    pub warn: Option<Fact>,
}

/// What an upload tells the flow.
#[derive(Clone, Debug)]
pub enum Sent {
    /// The run handed over no speech, so nothing was sent.
    Silent,
    /// The run's speech could not be had.
    Speech(SpeechError),
    /// A note from the service's run.
    Note(CloudNote),
}

/// Sends a dictation's speech to its service; the actor reads the key just before.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Upload {
    /// The dictation, whose run hands the speech over.
    pub take: u32,
    /// The upload's round, which its notes name; an older round's are dropped.
    pub round: u32,
    /// The service's row id, the one the bar shows.
    pub service: String,
    /// The keyboard language, an ISO 639 code, whose code the service's model wants.
    pub lang: String,
}

/// The reason the bar and a Compare row give for `fault`, about language `lang`, which came at wall time `at`.
pub(crate) fn why(fault: CloudFault, lang: &VoiceLang, at: LocalTime) -> CloudWhy {
    match fault {
        CloudFault::Network | CloudFault::Timeout => CloudWhy::Net,
        CloudFault::Key => CloudWhy::Key,
        CloudFault::Credit => CloudWhy::Credit,
        CloudFault::Account => CloudWhy::Account,
        CloudFault::Allowance { renews_in } => CloudWhy::Allowance {
            renews_at: renews(renews_in, at),
        },
        CloudFault::Rate => CloudWhy::Rate,
        CloudFault::TooBig => CloudWhy::TooBig,
        CloudFault::Language => CloudWhy::Language(lang.clone()),
        CloudFault::Region => CloudWhy::Region,
        CloudFault::Empty => CloudWhy::Empty,
        CloudFault::Strange => CloudWhy::Strange,
        CloudFault::Range(_) | CloudFault::Gone => CloudWhy::Fault,
    }
}

/// The wall time a used allowance renews at: `renews_in` after `at`, kept in `at`'s offset, so a note left open
/// never goes stale (owner, 2026-11-20); `None` when the service did not tell.
pub(crate) fn renews(renews_in: Option<Duration>, at: LocalTime) -> Option<LocalTime> {
    let ms = i64::try_from(renews_in?.as_millis()).ok()?;
    LocalTime::at(at.unix_ms.checked_add(ms)?, at.offset_min)
}

/// Why `shown`, the service on the bar, cannot have the speech now that the language picks `now`: a pick that moved
/// while `shown` stays `on`, a service off, a key gone, newer terms, or a model ruled out; `None` when it can.
pub(super) fn blocked(shown: &Net, now: Option<&Net>, on: bool) -> Option<CloudWhy> {
    match now {
        Some(net) if net.service == shown.service => unfit(net),
        _ => Some(left(on)),
    }
}

/// The note for a pick that left a service: it moved while that service stays `on`, else the service went off.
pub(super) fn left(on: bool) -> CloudWhy {
    if on { CloudWhy::Moved } else { CloudWhy::Off }
}

/// Why `net` cannot have the speech: no key, newer terms, or a model ruled out; `None` when it can.
pub(super) fn unfit(net: &Net) -> Option<CloudWhy> {
    match net {
        Net { key: false, .. } => Some(CloudWhy::Key),
        Net { terms: true, .. } => Some(CloudWhy::Terms),
        Net { unavailable, .. } => unavailable.map(CloudWhy::Unavailable),
    }
}

/// "Use the local model" when the language's local model is `local` and the recording is held with speech when
/// `voiced`; Download shows whenever the model is missing.
pub(super) fn local_use(local: &Result<Ready, Lack>, voiced: bool) -> LocalUse {
    match local {
        Ok(_) if voiced => LocalUse::Ready,
        Err(Lack::Missing { bytes }) => LocalUse::Download { bytes: *bytes },
        Ok(_) | Err(_) => LocalUse::Dimmed,
    }
}
