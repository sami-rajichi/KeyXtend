//! The fake's reads: the devices, each file's facts, each card's check and the clips' languages answer as the test set
//! them, and every checked ask is recorded.

use super::{FakeSpeech, SpeechError};
use crate::lock;
use kx_module_api::ports::{
    CardId, CheckAsk, CheckDone, Device, FactsAsk, JobLimits, ModelFacts, SpeechNote,
};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// A card's scripted check: the notes it tells, then its answer.
type Check = (Vec<SpeechNote>, Result<CheckDone, SpeechError>);

/// Whether the port lets `devices` return `error` past its ask's check: the worker's own faults.
fn worker_error(error: SpeechError) -> bool {
    use SpeechError::{Down, Gone, Timeout};
    matches!(error, Timeout | Gone | Down { .. })
}

/// Whether the port lets `facts` return `error` past its ask's check.
fn file_error(error: SpeechError) -> bool {
    use SpeechError::{BadFile, NoFile, Runner};
    matches!(error, NoFile | BadFile | Runner) || worker_error(error)
}

/// Whether the port lets `check` return `error` past its ask's check.
fn card_error(error: SpeechError) -> bool {
    use SpeechError::{CardFailed, CardRoom, NoCard};
    matches!(error, NoCard | CardRoom | CardFailed) || file_error(error)
}

/// Stops a debug build on an answer whose error the port never returns from `call`.
fn ported<T>(answer: &Result<T, SpeechError>, allowed: fn(SpeechError) -> bool, call: &str) {
    let error = answer.as_ref().err().copied();
    let fits = error.is_none_or(allowed);
    debug_assert!(fits, "the port never returns {error:?} from {call}");
}

/// What the test set for the reads, and the asks they took.
#[derive(Default)]
pub(super) struct Info {
    devices: Option<Result<Vec<Device>, SpeechError>>,
    facts: HashMap<PathBuf, Result<ModelFacts, SpeechError>>,
    checks: HashMap<CardId, Check>,
    pub(super) clips: Vec<String>,
    device_asks: Vec<JobLimits>,
    facts_asks: Vec<FactsAsk>,
    check_asks: Vec<CheckAsk>,
}

impl Info {
    /// Records `job`; the devices set, else `Gone`, as a worker that never started.
    pub(super) fn devices(&mut self, job: JobLimits) -> Result<Vec<Device>, SpeechError> {
        self.device_asks.push(job);
        self.devices.clone().unwrap_or(Err(SpeechError::Gone))
    }

    /// Records `ask`; the facts set for its file, else `NoFile`.
    pub(super) fn facts(&mut self, ask: &FactsAsk) -> Result<ModelFacts, SpeechError> {
        self.facts_asks.push(ask.clone());
        let set = self.facts.get(ask.file.expose()).cloned();
        set.unwrap_or(Err(SpeechError::NoFile))
    }

    /// Records `ask`; the check set for its card, else no notes and `NoCard`.
    pub(super) fn check(&mut self, ask: CheckAsk) -> Check {
        let set = self.checks.get(&ask.card.id).cloned();
        self.check_asks.push(ask);
        set.unwrap_or((Vec::new(), Err(SpeechError::NoCard)))
    }
}

impl FakeSpeech {
    /// Makes every later `devices` answer `answer`; `Gone` until set. A debug build stops on an error the port never
    /// returns there, as for each answer below.
    pub fn set_devices(&self, answer: Result<Vec<Device>, SpeechError>) {
        ported(&answer, worker_error, "devices");
        lock(&self.shared.log).info.devices = Some(answer);
    }

    /// Makes every later `facts` of `file` answer `answer`; a file never set is `NoFile`.
    pub fn set_facts(&self, file: &Path, answer: Result<ModelFacts, SpeechError>) {
        ported(&answer, file_error, "facts");
        let facts = &mut lock(&self.shared.log).info.facts;
        facts.insert(file.to_path_buf(), answer);
    }

    /// Makes every later `check` of `card` tell `notes`, then answer `answer`; a card never set is `NoCard`.
    pub fn set_check(
        &self,
        card: &CardId,
        notes: Vec<SpeechNote>,
        answer: Result<CheckDone, SpeechError>,
    ) {
        ported(&answer, card_error, "check");
        let checks = &mut lock(&self.shared.log).info.checks;
        checks.insert(card.clone(), (notes, answer));
    }

    /// Makes `clips` name these check clips' languages, in this order; none until set.
    pub fn set_clips(&self, langs: &[&str]) {
        let clips = langs.iter().map(|l| (*l).to_owned()).collect();
        lock(&self.shared.log).info.clips = clips;
    }

    /// The limits each `devices` was asked with, in order; refused ones are not kept.
    #[must_use]
    pub fn device_asks(&self) -> Vec<JobLimits> {
        lock(&self.shared.log).info.device_asks.clone()
    }

    /// Each `facts` ask, in order; refused ones are not kept.
    #[must_use]
    pub fn facts_asks(&self) -> Vec<FactsAsk> {
        lock(&self.shared.log).info.facts_asks.clone()
    }

    /// Each `check` ask, in order; refused ones are not kept.
    #[must_use]
    pub fn check_asks(&self) -> Vec<CheckAsk> {
        lock(&self.shared.log).info.check_asks.clone()
    }
}
