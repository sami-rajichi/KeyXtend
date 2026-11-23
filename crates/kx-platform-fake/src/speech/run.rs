//! The fake's live run: each call refused as the run stands or as scripted, else counted, then its cue told.

use super::log::{Log, Shared};
use super::{NO_MODEL, SpeechCall};
use crate::lock;
use kx_module_api::SecretBytes;
use kx_module_api::ports::{SpeechAsk, SpeechError, SpeechRun};
use std::sync::Arc;

/// A live fake run; dropping it closes it.
pub(super) struct FakeSpeechRun {
    pub(super) shared: Arc<Shared>,
    /// Its place in the log's runs.
    pub(super) id: usize,
}

impl FakeSpeechRun {
    /// Refuses `call` as the run stands, else does `work` on the log and tells the call's cue.
    fn act<T>(
        &self,
        call: SpeechCall,
        work: impl FnOnce(&mut Log, usize) -> T,
    ) -> Result<T, SpeechError> {
        let mut log = lock(&self.shared.log);
        log.refuse(self.id, call)?;
        let done = work(&mut log, self.id);
        let notes = log.cue(self.id, call);
        drop(log);
        self.shared.tell(self.id, notes);
        Ok(done)
    }
}

impl SpeechRun for FakeSpeechRun {
    fn stop(&mut self) -> Result<(), SpeechError> {
        self.act(SpeechCall::Stop, |log, id| log.runs[id].stops += 1)
    }

    /// Mutes the run's text until `again`, as the job drops it.
    fn cancel(&mut self) -> Result<(), SpeechError> {
        self.act(SpeechCall::Cancel, |log, id| {
            log.runs[id].cancels += 1;
            log.ears[id].muted = true;
        })
    }

    /// An ask with no model is refused as `Range(NO_MODEL)`; a taken ask writes text again after a Cancel.
    fn again(&mut self, ask: SpeechAsk) -> Result<(), SpeechError> {
        ask.check()?;
        if ask.model.is_none() {
            return Err(SpeechError::Range(NO_MODEL));
        }
        self.act(SpeechCall::Again, |log, id| {
            log.runs[id].agains.push(ask);
            log.ears[id].muted = false;
        })
    }

    fn keep(&mut self) -> Result<Vec<SecretBytes>, SpeechError> {
        self.act(SpeechCall::Keep, |log, id| {
            log.runs[id].keeps += 1;
            log.sound.clone()
        })
    }

    /// A listen-only run's speech as set, in fresh copies, once its `Done` was told.
    fn pieces(&mut self) -> Result<Vec<SecretBytes>, SpeechError> {
        self.act(SpeechCall::Pieces, |log, id| {
            log.runs[id].pieces += 1;
            log.speech.clone()
        })
    }

    /// Waits out a note being told on another thread, so none comes after it returns.
    fn close(&mut self) {
        let mut log = lock(&self.shared.log);
        let run = &mut log.runs[self.id];
        run.closed = true;
        run.held = false;
        let gone = log.ears[self.id].hearer.take();
        drop(log);
        self.shared.changed.notify_all();
        if let Some(hearer) = gone {
            drop(lock(&hearer));
        }
    }
}

impl Drop for FakeSpeechRun {
    fn drop(&mut self) {
        let closed = lock(&self.shared.log).runs[self.id].closed;
        if !closed {
            self.close();
        }
    }
}
