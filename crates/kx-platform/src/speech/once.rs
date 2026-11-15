//! The asks that run in a job of their own, apart from the dictation: the devices in a job capped at `job_mb`, a file's
//! facts at its size plus `job_mb`, and a card's check at its model's memory plus `job_mb`, so its long warm-up never
//! holds a dictation back. Each job says goodbye once it answered.

use super::frame::{model_of, path};
use super::job::{single, times, waits};
use super::note::{device, lost, note, unsent};
use super::router::{Hearer, Tell};
use super::{Shared, threads};
use crate::ear::Ear;
use crate::worker::{Launch, Spawner, Split, Stream, Told, VoiceFrames, Wait};
use kx_ipc::{VoiceAsk, VoiceNote};
use kx_module_api::Redacted;
use kx_module_api::ports::{
    CheckAsk, CheckDone, Device, FactsAsk, JobLimits, ModelFacts, SpeechError,
};
use std::sync::{Arc, mpsc};

/// A one-ask job's stream thread.
const THREAD: &str = "kx-voice-once";

/// The processor and each card, with each card's driver from the system.
pub(super) fn devices<S: Spawner<VoiceFrames, Worker: Split<VoiceFrames>>>(
    shared: &Shared<S>,
    job: &JobLimits,
) -> Result<Vec<Device>, SpeechError> {
    let make = |id| VoiceAsk::Devices { id };
    let wait = Wait::Once(waits(job).read);
    match ask(shared, single(job, 0), job, make, wait, None)? {
        VoiceNote::Devices { list, .. } => Ok(list
            .into_iter()
            .filter_map(|found| device(found, shared.hardware.as_ref()))
            .collect()),
        other => Err(refusal(&other)),
    }
}

/// What the file `ask` names tells about itself.
pub(super) fn facts<S: Spawner<VoiceFrames, Worker: Split<VoiceFrames>>>(
    shared: &Shared<S>,
    ask: &FactsAsk,
) -> Result<ModelFacts, SpeechError> {
    let file = path(ask.file.expose())?;
    let make = move |id| VoiceAsk::Facts { id, path: file };
    let wait = Wait::Once(waits(&ask.job).read);
    let launch = single(&ask.job, ask.size_mb);
    match self::ask(shared, launch, &ask.job, make, wait, None)? {
        VoiceNote::Facts { runner, langs, .. } => Ok(ModelFacts { runner, langs }),
        other => Err(refusal(&other)),
    }
}

/// A card's check on the shipped clip for `ask.clip`; `tell` hears its loading and warm-up, and nothing once it
/// returns.
pub(super) fn check<S: Spawner<VoiceFrames, Worker: Split<VoiceFrames>>>(
    shared: &Shared<S>,
    ask: &CheckAsk,
    tell: Tell,
) -> Result<CheckDone, SpeechError> {
    let clip = (shared.assets.clips.iter()).find(|(lang, _)| *lang == ask.clip);
    let clip = path(&clip.ok_or(SpeechError::NoFile)?.1)?;
    let threads = threads(ask.threads, shared.hardware.cores());
    let model = model_of(&ask.model, Some(&ask.card), threads)?;
    let make = move |id| VoiceAsk::Check { id, model, clip };
    let wait = Wait::Each(waits(&ask.job).check);
    let launch = single(&ask.job, ask.model.cpu_mb);
    let ear = Arc::new(Ear::new(tell));
    let answered = self::ask(shared, launch, &ask.job, make, wait, Some(Arc::clone(&ear)));
    ear.close();
    match answered? {
        VoiceNote::Checked {
            card_ms,
            cpu_ms,
            card_text,
            cpu_text,
            ..
        } => Ok(CheckDone {
            card_ms,
            cpu_ms,
            card_text: Redacted::new(card_text),
            cpu_text: Redacted::new(cpu_text),
        }),
        other => Err(refusal(&other)),
    }
}

/// Runs the ask `make` builds in a job of its own, `tell` hearing its other notes until it closes; its answer, or why
/// none came.
fn ask<S: Spawner<VoiceFrames, Worker: Split<VoiceFrames>>>(
    shared: &Shared<S>,
    launch: Launch,
    job: &JobLimits,
    make: impl FnOnce(u32) -> VoiceAsk<'static> + Send + 'static,
    wait: Wait,
    tell: Option<Hearer>,
) -> Result<VoiceNote, SpeechError> {
    let (answer, heard) = mpsc::channel();
    let hear = move |told: Told<VoiceNote>| {
        let answered = match told {
            Told::Note(voice) if answers(&voice) => Ok(voice),
            Told::Note(voice) => return side(tell.as_ref(), voice),
            Told::Lost(gone) => Err(lost(gone)),
            Told::Idle => Err(SpeechError::Gone),
        };
        let _ = answer.send(answered);
    };
    let (spawner, clock) = (shared.spawner.clone(), shared.clock.clone());
    let stream = Stream::open(THREAD, spawner, clock, times(job), launch, hear);
    stream.begin(make, wait).map_err(unsent)?;
    let answered = heard.recv().unwrap_or(Err(SpeechError::Gone));
    drop(stream);
    answered
}

/// True for a note that answers a one-ask job: its answer, or any fault, as a check is of the card.
fn answers(voice: &VoiceNote) -> bool {
    matches!(
        voice,
        VoiceNote::Devices { .. }
            | VoiceNote::Facts { .. }
            | VoiceNote::Checked { .. }
            | VoiceNote::Fault { .. }
    )
}

/// Tells a check's loading and warm-up to `tell` while it is open; other notes are dropped.
fn side(tell: Option<&Hearer>, voice: VoiceNote) {
    let told = matches!(voice, VoiceNote::Stage { .. } | VoiceNote::Warm { .. });
    if let (true, Some(tell), Some(told)) = (told, tell, note(voice)) {
        tell.tell([told]);
    }
}

/// The error an answer of the wrong kind gives: its fault, or `Internal` for another answer.
fn refusal(voice: &VoiceNote) -> SpeechError {
    match voice {
        VoiceNote::Fault { fault, .. } => SpeechError::from(*fault),
        _ => SpeechError::Internal,
    }
}
