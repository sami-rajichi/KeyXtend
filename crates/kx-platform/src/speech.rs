//! `Speech` over the worker's `voice` job (ADR-0042, ADR-0044): one streamed job per model, capped by its row and the
//! recording's room, which ends after `idle_min`; a model change hands the recording over in wiped buffers, and the
//! devices, a file's facts and a card's check each run in a job of their own. Text and sound stay redacted.

mod frame;
mod job;
mod note;
mod once;
mod router;
mod run;

use crate::worker::{Spawner, Split, Stream, VoiceFrames};
use job::{Key, Live};
use kx_module_api::Clock;
use kx_module_api::ports::{
    CheckAsk, CheckDone, Device, FactsAsk, JobLimits, ModelFacts, Speech, SpeechAsk, SpeechError,
    SpeechRun,
};
use router::{Router, Tell};
use run::WorkerRun;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

/// The value `again` names in its `Range` refusal when its ask names no model, as the fake does.
pub const NO_MODEL: &str = "model";

/// The dictation job's stream thread.
const THREAD: &str = "kx-voice";

/// The shipped voice files the asks name: the detector, and each self-test clip by its ISO 639 code. It has no
/// `Debug`, as a path may hold the user's name.
#[derive(Clone, Default)]
pub struct Assets {
    /// The voice detector's model file.
    pub detector: PathBuf,
    /// The self-test clips, by language.
    pub clips: HashMap<String, PathBuf>,
}

/// What the system tells the speech host beside the engine.
pub trait Hardware: Send + Sync {
    /// The processor's real cores, at least 1.
    fn cores(&self) -> usize;

    /// The driver version of the card the system names `name`, as the engine names it; `None` when unknown.
    fn driver(&self, name: &str) -> Option<String>;
}

/// What every run and job of one speech host share.
struct Shared<S: Spawner<VoiceFrames, Worker: Split<VoiceFrames>>> {
    spawner: Arc<S>,
    clock: Arc<dyn Clock>,
    assets: Assets,
    hardware: Arc<dyn Hardware>,
    router: Arc<Mutex<Router>>,
    /// The dictation job, held while a run changes it.
    live: Mutex<Live<S>>,
}

impl<S: Spawner<VoiceFrames, Worker: Split<VoiceFrames>>> Shared<S> {
    /// The dictation job, held.
    fn live(&self) -> MutexGuard<'_, Live<S>> {
        lock(&self.live)
    }

    /// The dictation job's stream for `key` with `extra_mb` more room, made, or relaunched when the job's key differs,
    /// which says goodbye to the job before; true when it is a new job.
    fn stream_for<'a>(
        &self,
        live: &'a mut Live<S>,
        key: &Key,
        extra_mb: u32,
    ) -> (&'a Stream<S, VoiceFrames>, bool) {
        let (launch, times) = (key.launch(extra_mb), key.times());
        let fresh = live.key.as_ref() != Some(key);
        if fresh {
            if let Some(stream) = &live.stream {
                stream.relaunch(launch.clone(), times);
            }
            live.key = Some(key.clone());
        }
        let stream = live.stream.get_or_insert_with(|| {
            let router = Arc::clone(&self.router);
            let tell = move |told| {
                let out = lock(&router).hear(told);
                router::tell(out);
            };
            let (spawner, clock) = (Arc::clone(&self.spawner), Arc::clone(&self.clock));
            Stream::open(THREAD, spawner, clock, times, launch, tell)
        });
        (stream, fresh)
    }
}

/// Speech by the worker `S` starts, with the shipped `Assets` and the system's `Hardware`.
pub struct WorkerSpeech<S: Spawner<VoiceFrames, Worker: Split<VoiceFrames>>> {
    shared: Arc<Shared<S>>,
}

impl<S: Spawner<VoiceFrames, Worker: Split<VoiceFrames>>> WorkerSpeech<S> {
    /// Speech over `spawner`, whose jobs read the time from `clock`; no worker starts before the first ask.
    #[must_use]
    pub fn new(
        spawner: S,
        clock: Arc<dyn Clock>,
        assets: Assets,
        hardware: Arc<dyn Hardware>,
    ) -> Self {
        let shared = Shared {
            spawner: Arc::new(spawner),
            clock,
            assets,
            hardware,
            router: Arc::default(),
            live: Mutex::new(Live::default()),
        };
        Self {
            shared: Arc::new(shared),
        }
    }

    /// Wakes the dictation job's thread, which then checks its waits and idle time against its clock.
    #[cfg(test)]
    fn poke(&self) {
        if let Some(stream) = &self.shared.live().stream {
            stream.poke();
        }
    }
}

impl<S: Spawner<VoiceFrames, Worker: Split<VoiceFrames>>> Speech for WorkerSpeech<S> {
    fn open(&self, ask: SpeechAsk, tell: Tell) -> Result<Box<dyn SpeechRun>, SpeechError> {
        ask.check()?;
        let run = WorkerRun::open(Arc::clone(&self.shared), &ask, tell)?;
        Ok(Box::new(run))
    }

    fn devices(&self, job: &JobLimits) -> Result<Vec<Device>, SpeechError> {
        job.check()?;
        once::devices(&self.shared, job)
    }

    fn facts(&self, ask: &FactsAsk) -> Result<ModelFacts, SpeechError> {
        ask.check()?;
        once::facts(&self.shared, ask)
    }

    fn check(&self, ask: CheckAsk, tell: Tell) -> Result<CheckDone, SpeechError> {
        ask.check()?;
        once::check(&self.shared, &ask, tell)
    }
}

/// Locks `m`, taking the value even when a panicking thread poisoned it.
fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(PoisonError::into_inner)
}

/// The processor threads to ask for: `asked`, or the real cores for 0, never past them or the frames' top.
fn threads(asked: u8, cores: usize) -> u8 {
    let top = u8::try_from(cores).unwrap_or(u8::MAX);
    let real = top.clamp(1, *SpeechAsk::THREADS.end());
    match asked {
        0 => real,
        n => n.min(real),
    }
}

#[cfg(test)]
mod tests;
