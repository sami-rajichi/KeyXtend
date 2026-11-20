//! The voice gate build only: speech that logs the device of each dictation a model writes, by id, so G22 can tell
//! whether a card or the processor wrote a language's text, and each stage of a run; a listen-only run hands its
//! speech on and logs no device. It never logs the text.

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use kx_ipc::gate::DEVICE;
use kx_module_api::SecretBytes;
use kx_module_api::ports::{
    CardId, CheckAsk, CheckDone, Device, DevicePick, FactsAsk, JobLimits, ModelFacts, Speech,
    SpeechAsk, SpeechError, SpeechNote, SpeechRun, SpeechStage,
};

use super::stage;

/// The message of the line logged as each dictation is written; kx-gates reads it back.
pub const WROTE_LINE: &str = "voice gate build: a dictation was written";

/// What a run's latest ask named, which its notes read.
#[derive(Clone)]
struct Ask {
    /// The card asked for.
    card: Option<CardId>,
    /// A model writes the text; a listen-only run's speech goes to a service instead.
    model: bool,
}

impl Ask {
    /// What `ask` names.
    fn of(ask: &SpeechAsk) -> Self {
        Self {
            card: ask.card.as_ref().map(|c| c.id.clone()),
            model: ask.model.is_some(),
        }
    }
}

/// The run's latest ask, shared with its notes.
type Asked = Arc<Mutex<Ask>>;

/// Speech `S`, logging the device of each dictation a model writes.
pub struct Told<S>(pub S);

impl<S: Speech> Speech for Told<S> {
    fn open(
        &self,
        ask: SpeechAsk,
        tell: Box<dyn Fn(SpeechNote) + Send>,
    ) -> Result<Box<dyn SpeechRun>, SpeechError> {
        stage::stage(stage::START);
        let asked: Asked = Arc::new(Mutex::new(Ask::of(&ask)));
        let seen = Arc::clone(&asked);
        let told = Box::new(move |note: SpeechNote| {
            heard(&note, &lock(&seen));
            tell(note);
        });
        let run = self.0.open(ask, told)?;
        Ok(Box::new(Run { run, asked }))
    }

    fn devices(&self, job: &JobLimits) -> Result<Vec<Device>, SpeechError> {
        self.0.devices(job)
    }

    fn facts(&self, ask: &FactsAsk) -> Result<ModelFacts, SpeechError> {
        self.0.facts(ask)
    }

    fn check(
        &self,
        ask: CheckAsk,
        tell: Box<dyn Fn(SpeechNote) + Send>,
    ) -> Result<CheckDone, SpeechError> {
        self.0.check(ask, tell)
    }

    fn clips(&self) -> Vec<String> {
        self.0.clips()
    }
}

/// A run whose `again` sets the ask its notes read.
struct Run {
    run: Box<dyn SpeechRun>,
    asked: Asked,
}

impl SpeechRun for Run {
    fn stop(&mut self) -> Result<(), SpeechError> {
        stage::stage(stage::STOP);
        self.run.stop()
    }

    fn cancel(&mut self) -> Result<(), SpeechError> {
        self.run.cancel()
    }

    /// Its notes read the new ask from now on, and the ask before again when it is refused.
    fn again(&mut self, ask: SpeechAsk) -> Result<(), SpeechError> {
        let before = std::mem::replace(&mut *lock(&self.asked), Ask::of(&ask));
        let asked = self.run.again(ask);
        if asked.is_err() {
            *lock(&self.asked) = before;
        }
        asked
    }

    fn keep(&mut self) -> Result<Vec<SecretBytes>, SpeechError> {
        self.run.keep()
    }

    fn pieces(&mut self) -> Result<Vec<SecretBytes>, SpeechError> {
        self.run.pieces()
    }

    fn close(&mut self) {
        self.run.close();
    }
}

/// Logs what `note` tells of a run that asked `ask`: the microphone opening, each piece, and the load and device of a
/// dictation a model wrote.
fn heard(note: &SpeechNote, ask: &Ask) {
    match note {
        SpeechNote::Stage(SpeechStage::Open) => stage::stage(stage::OPEN),
        SpeechNote::Piece { index, .. } => stage::piece(*index, None),
        SpeechNote::Done(done) if ask.model => {
            if done.load_ms > 0 {
                stage::load(done.load_ms);
            }
            wrote(device(ask.card.as_ref(), done.card));
        }
        _ => {}
    }
}

/// The device that wrote a dictation: the asked `card` when one `served` to the end, else the processor.
fn device(card: Option<&CardId>, served: bool) -> &str {
    card.filter(|_| served)
        .map_or(DevicePick::CPU, CardId::as_str)
}

/// Logs that `device` wrote a dictation; its Display form keeps the id one bare word on the line.
fn wrote(device: &str) {
    tracing::info!({ DEVICE } = %device, "{WROTE_LINE}");
}

/// `m`, even after a panic elsewhere held it.
fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(PoisonError::into_inner)
}

#[cfg(test)]
mod tests;
