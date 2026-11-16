//! The voice gate build only: speech that logs the device of each dictation it writes, by id, so G22 can tell whether a
//! card or the processor wrote a language's text. It never logs the text.

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use kx_module_api::SecretBytes;
use kx_module_api::ports::{
    CardId, CheckAsk, CheckDone, Device, DevicePick, FactsAsk, JobLimits, ModelFacts, Speech,
    SpeechAsk, SpeechError, SpeechNote, SpeechRun,
};

/// The message of the line logged as each dictation is written; kx-gates reads it back.
pub const WROTE_LINE: &str = "voice gate build: a dictation was written";

/// The card a run's latest ask named, which its notes read.
type Asked = Arc<Mutex<Option<CardId>>>;

/// Speech `S`, logging the device of each dictation it writes.
pub struct Told<S>(pub S);

impl<S: Speech> Speech for Told<S> {
    fn open(
        &self,
        ask: SpeechAsk,
        tell: Box<dyn Fn(SpeechNote) + Send>,
    ) -> Result<Box<dyn SpeechRun>, SpeechError> {
        let asked: Asked = Arc::new(Mutex::new(card_of(&ask)));
        let seen = Arc::clone(&asked);
        let told = Box::new(move |note: SpeechNote| {
            if let SpeechNote::Done(done) = &note {
                wrote(device(lock(&seen).as_ref(), done.card));
            }
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

/// A run whose `again` sets the card its notes read.
struct Run {
    run: Box<dyn SpeechRun>,
    asked: Asked,
}

impl SpeechRun for Run {
    fn stop(&mut self) -> Result<(), SpeechError> {
        self.run.stop()
    }

    fn cancel(&mut self) -> Result<(), SpeechError> {
        self.run.cancel()
    }

    /// Its notes read the new ask's card from now on, and the card before again when the ask is refused.
    fn again(&mut self, ask: SpeechAsk) -> Result<(), SpeechError> {
        let before = std::mem::replace(&mut *lock(&self.asked), card_of(&ask));
        let asked = self.run.again(ask);
        if asked.is_err() {
            *lock(&self.asked) = before;
        }
        asked
    }

    fn keep(&mut self) -> Result<Vec<SecretBytes>, SpeechError> {
        self.run.keep()
    }

    fn close(&mut self) {
        self.run.close();
    }
}

/// The card `ask` names.
fn card_of(ask: &SpeechAsk) -> Option<CardId> {
    ask.card.as_ref().map(|c| c.id.clone())
}

/// The device that wrote a dictation: the asked `card` when one `served` to the end, else the processor.
fn device(card: Option<&CardId>, served: bool) -> &str {
    card.filter(|_| served)
        .map_or(DevicePick::CPU, CardId::as_str)
}

/// Logs that `device` wrote a dictation; its Display form keeps the id one bare word on the line.
fn wrote(device: &str) {
    tracing::info!(device = %device, "{WROTE_LINE}");
}

/// `m`, even after a panic elsewhere held it.
fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(PoisonError::into_inner)
}

#[cfg(test)]
mod tests;
