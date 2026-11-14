//! A platform for the voice contract that answers every port at once: a run tells the microphone opening and its end,
//! a download its end, and a watch one lock.

use kx_module_api::ports::{
    CheckAsk, CheckDone, Device, Drive, FactsAsk, FetchAsk, FetchNote, FetchRun, Fingerprint,
    FolderScan, JobLimits, ModelError, ModelFacts, ModelFiles, SessionError, SessionNote,
    SessionRun, SessionWatch, Speech, SpeechAsk, SpeechDone, SpeechError, SpeechNote, SpeechRun,
    SpeechStage,
};
use kx_module_api::{Redacted, SecretBytes, Zeroizing};
use std::path::Path;

/// A platform that answers every port at once: a run tells the mic opening and its end, a watch one lock.
pub(super) struct Stub;

/// The run, the download and the watch of `Stub`.
struct StubRun;

impl SpeechRun for StubRun {
    fn stop(&mut self) -> Result<(), SpeechError> {
        Ok(())
    }
    fn cancel(&mut self) -> Result<(), SpeechError> {
        Ok(())
    }
    fn again(&mut self, _: SpeechAsk) -> Result<(), SpeechError> {
        Err(SpeechError::NoSound)
    }
    fn keep(&mut self) -> Result<Vec<SecretBytes>, SpeechError> {
        Ok(vec![Redacted::new(Zeroizing::new(vec![1, 2]))])
    }
    fn close(&mut self) {}
}

impl FetchRun for StubRun {
    fn pause(&mut self) {}
    fn resume(&mut self) {}
    fn cancel(&mut self) {}
}

impl SessionRun for StubRun {}

impl Speech for Stub {
    fn open(
        &self,
        ask: SpeechAsk,
        tell: Box<dyn Fn(SpeechNote) + Send>,
    ) -> Result<Box<dyn SpeechRun>, SpeechError> {
        ask.check()?;
        tell(SpeechNote::Stage(SpeechStage::Open));
        tell(SpeechNote::Done(SpeechDone {
            pieces: 0,
            sound_ms: 1,
            wait_ms: 0,
            load_ms: 0,
            memory_mb: 1,
            card: false,
        }));
        Ok(Box::new(StubRun))
    }
    fn devices(&self, _: &JobLimits) -> Result<Vec<Device>, SpeechError> {
        Ok(Vec::new())
    }
    fn facts(&self, ask: &FactsAsk) -> Result<ModelFacts, SpeechError> {
        ask.check()?;
        Err(SpeechError::BadFile)
    }
    fn check(
        &self,
        _: CheckAsk,
        _: Box<dyn Fn(SpeechNote) + Send>,
    ) -> Result<CheckDone, SpeechError> {
        Err(SpeechError::Runner)
    }
}
impl ModelFiles for Stub {
    fn drives(&self) -> Vec<Drive> {
        Vec::new()
    }
    fn make(&self, _: &Path) -> Result<(), ModelError> {
        Err(ModelError::ReadOnly)
    }
    fn scan(&self, _: &Path, _: &[String]) -> Result<FolderScan, ModelError> {
        Ok(FolderScan::default())
    }
    fn fingerprint(&self, _: &Path) -> Result<Fingerprint, ModelError> {
        Ok(Fingerprint::new([0; Fingerprint::BYTES]))
    }
    fn fetch(
        &self,
        ask: FetchAsk,
        tell: Box<dyn Fn(FetchNote) + Send>,
    ) -> Result<Box<dyn FetchRun>, ModelError> {
        ask.check()?;
        tell(FetchNote::Done);
        Ok(Box::new(StubRun))
    }
    fn remove(&self, _: &Path) -> Result<(), ModelError> {
        Ok(())
    }
}

impl SessionWatch for Stub {
    fn watch(
        &self,
        tell: Box<dyn Fn(SessionNote) + Send>,
    ) -> Result<Box<dyn SessionRun>, SessionError> {
        tell(SessionNote::Locked);
        Ok(Box::new(StubRun))
    }
}
