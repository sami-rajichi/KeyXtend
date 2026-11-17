//! A fake speech service: the test tells a run's notes on cue or scripts them per call, kills the worker, fails a call
//! once and sets the reads' answers, and the fake checks each ask by its own rule and records it. It never sleeps.

use crate::lock;
use kx_module_api::ports::{
    CheckAsk, CheckDone, Device, FactsAsk, JobLimits, ModelFacts, Speech, SpeechAsk, SpeechError,
    SpeechNote, SpeechRun,
};
use kx_module_api::{Redacted, SecretBytes, Zeroizing};
use std::fmt;
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

mod info;
mod log;
mod run;

use log::{Ear, Shared};
use run::FakeSpeechRun;

/// The value `again` names in its `Range` refusal when its ask names no model.
pub const NO_MODEL: &str = "model";

/// The callback a run or a check tells its notes to.
type Tell = Box<dyn Fn(SpeechNote) + Send>;
/// A run's callback, shared so it is called with the log unlocked.
type Hearer = Arc<Mutex<Tell>>;

/// One call of a run's life, to script notes after it or fail it once.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum SpeechCall {
    /// `Speech::open`.
    Open,
    /// `SpeechRun::stop`.
    Stop,
    /// `SpeechRun::cancel`.
    Cancel,
    /// `SpeechRun::again`.
    Again,
    /// `SpeechRun::keep`.
    Keep,
    /// `SpeechRun::pieces`.
    Pieces,
}

impl SpeechCall {
    /// Whether the port lets this call return `error`, besides its ask's own refusals.
    fn returns(self, error: SpeechError) -> bool {
        use SpeechError::{Down, Gone, Internal, NoSound, Timeout};
        match self {
            Self::Open => matches!(error, Down { .. }),
            Self::Stop | Self::Cancel => matches!(error, Gone | Timeout),
            Self::Again => matches!(error, NoSound | Gone | Timeout | Down { .. }),
            Self::Keep => matches!(error, NoSound | Gone | Timeout),
            Self::Pieces => matches!(error, NoSound | Internal),
        }
    }
}

/// What one run was asked and where it stands; failed calls are not counted.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RunLog {
    /// The ask it was opened with.
    pub ask: SpeechAsk,
    /// The asks `again` took, in order.
    pub agains: Vec<SpeechAsk>,
    /// `stop` calls.
    pub stops: usize,
    /// `cancel` calls.
    pub cancels: usize,
    /// `keep` calls.
    pub keeps: usize,
    /// `pieces` calls that handed the speech over.
    pub pieces: usize,
    /// `Done` notes told; after one, a listen-only run's speech can be handed over.
    pub dones: usize,
    /// The recording is held: from the open until a close, a `Wiped` note or a death.
    pub held: bool,
    /// The worker died: a `Gone` or `Timeout` was told as a fault or returned by a call.
    pub dead: bool,
    /// `close` was called or the run dropped.
    pub closed: bool,
}

impl RunLog {
    /// A run just opened with `ask`, holding its recording.
    fn new(ask: SpeechAsk) -> Self {
        Self {
            ask,
            agains: Vec::new(),
            stops: 0,
            cancels: 0,
            keeps: 0,
            pieces: 0,
            dones: 0,
            held: true,
            dead: false,
            closed: false,
        }
    }

    /// Whether the run still hears notes.
    fn live(&self) -> bool {
        !self.closed && !self.dead
    }

    /// Whether `pieces` hands the speech over: a listen-only run that never took `again` or Cancel, after its `Done`,
    /// still holding the recording.
    fn hands_over(&self) -> bool {
        let alone = self.ask.model.is_none() && self.agains.is_empty();
        alone && self.dones > 0 && self.cancels == 0 && self.held
    }
}

/// Speech shared by its clones: runs hear notes in order on the thread that tells them, and a `tell` must not call
/// back into the fake's run or `tell`.
#[derive(Clone, Default)]
pub struct FakeSpeech {
    shared: Arc<Shared>,
}

impl FakeSpeech {
    /// Speech with no run and nothing scripted.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Tells `note` to the newest live run at once on this thread; false when none hears it, as text after Cancel. A
    /// `Fault(Gone)` or `Fault(Timeout)` kills the worker and a `Wiped` drops the recording.
    #[must_use]
    pub fn tell(&self, note: SpeechNote) -> bool {
        let live = lock(&self.shared.log).runs.iter().rposition(RunLog::live);
        live.is_some_and(|id| self.shared.tell(id, vec![note]))
    }

    /// Kills the worker under the newest live run, telling it `Fault(Gone)`; false when none runs.
    #[must_use]
    pub fn die(&self) -> bool {
        self.tell(SpeechNote::Fault(SpeechError::Gone))
    }

    /// Scripts `notes` for the next live run's `call` that succeeds, after those already scripted for it; they are
    /// told after its work and before it returns, cut as `tell` cuts them.
    pub fn on(&self, call: SpeechCall, notes: Vec<SpeechNote>) {
        let mut log = lock(&self.shared.log);
        log.cues.entry(call).or_default().push_back(notes);
    }

    /// Fails the next `call` with `error`, once, a `Gone` or `Timeout` killing the worker; a debug build stops on an
    /// error the port never returns there.
    pub fn fail(&self, call: SpeechCall, error: SpeechError) {
        let ported = call.returns(error);
        debug_assert!(ported, "the port never returns {error:?} from {call:?}");
        lock(&self.shared.log).fails.insert(call, error);
    }

    /// Sets the recording `keep` hands back, in these pieces; none until set.
    pub fn set_sound(&self, pieces: Vec<Vec<u8>>) {
        lock(&self.shared.log).sound = secret_bytes(pieces);
    }

    /// Sets the speech a listen-only run's `pieces` hands over once its `Done` was told, in these pieces; none until
    /// set, as a run that heard no speech.
    pub fn set_speech(&self, pieces: Vec<Vec<u8>>) {
        lock(&self.shared.log).speech = secret_bytes(pieces);
    }

    /// Each run opened, in order, with what it was asked.
    #[must_use]
    pub fn runs(&self) -> Vec<RunLog> {
        lock(&self.shared.log).runs.clone()
    }

    /// Waits up to `within` until `until` holds for the runs, so a test never sleeps; true when it does.
    #[must_use]
    pub fn wait(&self, within: Duration, until: impl Fn(&[RunLog]) -> bool) -> bool {
        let log = lock(&self.shared.log);
        let waited = (self.shared.changed).wait_timeout_while(log, within, |l| !until(&l.runs));
        let (log, _) = waited.unwrap_or_else(PoisonError::into_inner);
        until(&log.runs)
    }
}

/// `pieces` as secret bytes, wiped on drop.
fn secret_bytes(pieces: Vec<Vec<u8>>) -> Vec<SecretBytes> {
    let wiped = pieces.into_iter().map(|p| Redacted::new(Zeroizing::new(p)));
    wiped.collect()
}

/// Shows the counts, never a text or the sound.
impl fmt::Debug for FakeSpeech {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let log = lock(&self.shared.log);
        let live = log.runs.iter().filter(|r| r.live()).count();
        f.debug_struct("FakeSpeech")
            .field("runs", &log.runs.len())
            .field("live", &live)
            .field("sound_pieces", &log.sound.len())
            .finish_non_exhaustive()
    }
}

impl Speech for FakeSpeech {
    fn open(&self, ask: SpeechAsk, tell: Tell) -> Result<Box<dyn SpeechRun>, SpeechError> {
        ask.check()?;
        let mut log = lock(&self.shared.log);
        if let Some(error) = log.fails.remove(&SpeechCall::Open) {
            return Err(error);
        }
        let id = log.runs.len();
        log.runs.push(RunLog::new(ask));
        log.ears.push(Ear::new(tell));
        let notes = log.cue(id, SpeechCall::Open);
        drop(log);
        self.shared.tell(id, notes);
        let shared = Arc::clone(&self.shared);
        Ok(Box::new(FakeSpeechRun { shared, id }))
    }

    fn devices(&self, job: &JobLimits) -> Result<Vec<Device>, SpeechError> {
        job.check()?;
        lock(&self.shared.log).info.devices(*job)
    }

    fn facts(&self, ask: &FactsAsk) -> Result<ModelFacts, SpeechError> {
        ask.check()?;
        lock(&self.shared.log).info.facts(ask)
    }

    fn check(&self, ask: CheckAsk, tell: Tell) -> Result<CheckDone, SpeechError> {
        ask.check()?;
        let (notes, answer) = lock(&self.shared.log).info.check(ask);
        notes.into_iter().for_each(tell);
        answer
    }

    fn clips(&self) -> Vec<String> {
        lock(&self.shared.log).info.clips.clone()
    }
}

#[cfg(test)]
mod tests;
