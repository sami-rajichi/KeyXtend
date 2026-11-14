//! The speech port (ADR-0042, ADR-0044): a dictation run in the worker's `voice` job and the notes it tells, the
//! devices, a model file's facts and a card's check. Only `Microphone` reaches it, and text and sound stay redacted.

mod ask;
mod device;
mod note;

pub use ask::{CardUse, CheckAsk, FactsAsk, JobLimits, SpeechAsk, SpeechModel};
pub use device::{CardId, Device, DeviceKind, DevicePick, Memory, ModelFacts};
pub use note::{
    CheckDone, Hundredths, LEVEL_FULL, MAX_PERCENT, SpeechDone, SpeechNote, SpeechStage, Verdict,
};

use crate::{Capability, SecretBytes, ServiceId, ServiceKey};
use kx_ipc::VoiceFault;
use std::ops::RangeInclusive;
use std::time::Duration;
use thiserror::Error;

/// How many bytes a tag holds, each plain ASCII: a language code, a runner or a card's id, as the frames cap it.
pub const TAG_LEN: RangeInclusive<usize> = 1..=kx_ipc::MAX_TAG_BYTES;

/// Why a run, a list, a read or a check failed, or a fault a run told; no variant holds text or sound.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Error)]
pub enum SpeechError {
    /// The ask's value of this name is outside its range.
    #[error("the ask's {0} is outside its range")]
    Range(&'static str),
    /// A file to read is not a plain drive path (`path_ok`).
    #[error("the path is not a plain drive path")]
    BadPath,
    /// No microphone was found.
    #[error("no microphone was found")]
    NoMic,
    /// Another app holds the microphone.
    #[error("another app holds the microphone")]
    MicBusy,
    /// The system's privacy switch refused the microphone.
    #[error("the system refused the microphone")]
    MicRefused,
    /// The microphone was unplugged while listening.
    #[error("the microphone was unplugged")]
    Unplugged,
    /// The model, the detector or the clip is missing.
    #[error("a voice file is missing")]
    NoFile,
    /// The runner or the detector cannot read its file.
    #[error("a voice file cannot be read")]
    BadFile,
    /// The model does not know the asked language code.
    #[error("the model does not know this language")]
    Language,
    /// The runner is unknown, or not in this build.
    #[error("the model's runner is not in this build")]
    Runner,
    /// The named card is gone; the processor serves.
    #[error("the card is gone")]
    NoCard,
    /// The card lacks the free memory now; the processor serves.
    #[error("the card has too little free memory")]
    CardRoom,
    /// The card failed; the processor writes the same recording.
    #[error("the card failed")]
    CardFailed,
    /// No recording is held: the run closed, or its job ended and wiped it.
    #[error("no recording is held")]
    NoSound,
    /// The job cannot take this ask now, such as a second model.
    #[error("the voice job cannot take this now")]
    OutOfStep,
    /// The job reached its memory cap.
    #[error("the voice job ran out of memory")]
    Memory,
    /// The job failed inside.
    #[error("the voice job failed")]
    Internal,
    /// The worker gave no answer within its wait.
    #[error("the voice worker took too long")]
    Timeout,
    /// The worker ended or answered badly.
    #[error("the voice worker ended")]
    Gone,
    /// The worker rests after failures or could not start, so nothing was tried.
    #[error("the voice worker is down")]
    Down {
        /// How long until the worker may start again.
        retry_in: Duration,
    },
}

impl SpeechError {
    /// False for a card's fault, after which the processor carries on with the same run.
    #[must_use]
    pub fn ends(self) -> bool {
        !matches!(self, Self::NoCard | Self::CardRoom | Self::CardFailed)
    }
}

/// Each fault the `voice` job sends, as the same error.
impl From<VoiceFault> for SpeechError {
    fn from(fault: VoiceFault) -> Self {
        match fault {
            VoiceFault::NoMic => Self::NoMic,
            VoiceFault::MicBusy => Self::MicBusy,
            VoiceFault::MicRefused => Self::MicRefused,
            VoiceFault::Unplugged => Self::Unplugged,
            VoiceFault::NoFile => Self::NoFile,
            VoiceFault::BadFile => Self::BadFile,
            VoiceFault::Language => Self::Language,
            VoiceFault::Runner => Self::Runner,
            VoiceFault::NoCard => Self::NoCard,
            VoiceFault::CardRoom => Self::CardRoom,
            VoiceFault::CardFailed => Self::CardFailed,
            VoiceFault::NoSound => Self::NoSound,
            VoiceFault::OutOfStep => Self::OutOfStep,
            VoiceFault::Memory => Self::Memory,
            VoiceFault::Internal => Self::Internal,
        }
    }
}

/// One dictation on the platform's side; it holds the recording until `close`. Dropping it closes it.
pub trait SpeechRun: Send {
    /// Stops listening: the last piece is written with its silence cut, then `Done` follows.
    ///
    /// # Errors
    /// `Gone` or `Timeout` when the worker does not answer.
    fn stop(&mut self) -> Result<(), SpeechError>;

    /// Stops listening or writing at once, telling no more text; the recording stays until `close`.
    ///
    /// # Errors
    /// `Gone` or `Timeout` when the worker does not answer.
    fn cancel(&mut self) -> Result<(), SpeechError>;

    /// Writes the held recording again as `ask` says, in a new job when its model differs; notes follow as before.
    ///
    /// # Errors
    /// `Range` when `ask.check` refuses it or it names no model, `NoSound` when the recording is gone.
    fn again(&mut self, ask: SpeechAsk) -> Result<(), SpeechError>;

    /// The held recording, in pieces under the frame limit, for Keep recordings or a later service.
    ///
    /// # Errors
    /// `NoSound` when the recording is gone, `Gone` or `Timeout` when the worker does not answer.
    fn keep(&mut self) -> Result<Vec<SecretBytes>, SpeechError>;

    /// Wipes the recording and ends the run; the job stays for `JobLimits::idle_min`.
    fn close(&mut self);
}

/// Turns speech into text in the worker's sandboxed `voice` job; the platform provides it.
pub trait Speech: Send + Sync {
    /// Starts a run as `ask` says: the microphone opens at once and the model loads meanwhile. `tell` hears each note
    /// in order, on any thread.
    ///
    /// # Errors
    /// `Range` when `ask.check` refuses it, `Down` while the worker rests; the rest come as `Fault` notes.
    fn open(
        &self,
        ask: SpeechAsk,
        tell: Box<dyn Fn(SpeechNote) + Send>,
    ) -> Result<Box<dyn SpeechRun>, SpeechError>;

    /// The processor and each card the engine finds, with their memory now, read in a job of `job`'s limits.
    ///
    /// # Errors
    /// `Range` when `job.check` refuses it, `Timeout`, `Gone` or `Down`.
    fn devices(&self, job: &JobLimits) -> Result<Vec<Device>, SpeechError>;

    /// What the model file `ask` names tells about itself, read in a job capped at its size plus `job_mb`.
    ///
    /// # Errors
    /// `BadPath` or `Range` when `ask.check` refuses it, `NoFile` or `BadFile` when it is no readable model, `Runner`
    /// when no runner reads it, else as `devices`.
    fn facts(&self, ask: &FactsAsk) -> Result<ModelFacts, SpeechError>;

    /// Runs a card's self-test in a fresh job: the fit, the warm-up, then the clip on the card and on the processor.
    /// `tell` hears the `Loading` stage and `Warming` meanwhile.
    ///
    /// # Errors
    /// `Range` when `ask.check` refuses it, `NoCard`, `CardRoom` or `CardFailed` for the card, else as `facts`.
    fn check(
        &self,
        ask: CheckAsk,
        tell: Box<dyn Fn(SpeechNote) + Send>,
    ) -> Result<CheckDone, SpeechError>;
}

/// The speech service; it needs `Microphone` (ADR-0042 decision 8).
pub struct SpeechKey;

impl ServiceKey for SpeechKey {
    type Api = dyn Speech;
    const ID: ServiceId = ServiceId::new("speech");
    const CAPABILITY: Option<Capability> = Some(Capability::Microphone);
}

#[cfg(test)]
mod tests;
