//! The recorder behind the app's microphone seal (ADR-0056 decision 10): the app checks each pass once here, so a pass
//! from another seal never reaches the platform's recorder, whether or not that recorder checks it again.

use kx_module_api::ports::{
    MicPass, MicSeal, RecordAsk, RecordError, RecordFacts, RecordNote, RecordRun, Recorder,
    SnipFile, SpeedAsk,
};
use std::sync::Arc;

/// `BadPass` for a pass `seal` did not make; none is fine.
fn check(seal: &MicSeal, pass: Option<&MicPass>) -> Result<(), RecordError> {
    match pass {
        Some(pass) if !seal.opens(pass) => Err(RecordError::BadPass),
        _ => Ok(()),
    }
}

/// A recorder that hands the platform's only the asks and switches whose pass its seal made.
pub struct Sealed {
    seal: MicSeal,
    inner: Arc<dyn Recorder>,
}

impl Sealed {
    /// The platform's recorder `inner` behind `seal`.
    #[must_use]
    pub fn new(seal: MicSeal, inner: Arc<dyn Recorder>) -> Self {
        Self { seal, inner }
    }
}

impl Recorder for Sealed {
    fn facts(&self) -> RecordFacts {
        self.inner.facts()
    }

    /// Checks the ask's ranges first, then its pass, as the port's errors come.
    fn start(
        &self,
        ask: RecordAsk,
        tell: Box<dyn Fn(RecordNote) + Send>,
    ) -> Result<Box<dyn RecordRun>, RecordError> {
        ask.check()?;
        check(&self.seal, ask.mic.as_ref())?;
        let inner = self.inner.start(ask, tell)?;
        let seal = self.seal.clone();
        Ok(Box::new(SealedRun { seal, inner }))
    }

    fn speed(&self, ask: &SpeedAsk) -> Result<SnipFile, RecordError> {
        self.inner.speed(ask)
    }
}

/// A recording whose microphone switch takes only its seal's passes.
struct SealedRun {
    seal: MicSeal,
    inner: Box<dyn RecordRun>,
}

impl RecordRun for SealedRun {
    fn file(&self) -> &SnipFile {
        self.inner.file()
    }

    fn mic(&mut self, on: Option<MicPass>) -> Result<(), RecordError> {
        check(&self.seal, on.as_ref())?;
        self.inner.mic(on)
    }

    fn sound(&mut self, on: bool) -> Result<(), RecordError> {
        self.inner.sound(on)
    }

    fn stop(&mut self) -> Result<(), RecordError> {
        self.inner.stop()
    }

    fn cancel(&mut self) {
        self.inner.cancel();
    }
}
