//! The recorder port (ADR-0056 decisions 3 to 5): a part of the screen recorded with its sound switches by the worker's
//! `record` job into a video of the Snips folder's unsaved part, which tells the time, the end or a fault; re-timed
//! copies there; and what the system can record now. Only `ScreenCapture` reaches it, the microphone also needs a
//! `MicPass` of the app's seal, and sizes count `MB` bytes to the MB, as `kx_ipc::MIB` does.

use super::file::SnipFile;
use super::mic::MicPass;
use super::shot::Part;
use super::speed::SpeedAsk;
use super::waits::RecordWaits;
use crate::{Capability, ServiceId, ServiceKey};
use std::ops::RangeInclusive;
use thiserror::Error;

/// Whether a sound device can be recorded now.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Gear {
    /// It is there and allowed.
    Ready,
    /// None was found.
    Missing,
    /// The system's privacy switch refuses it.
    Refused,
}

/// What the system can record now; each lack dims its keys with a tip (edges 11 and 14). The Snips folder and its
/// room are `SnipFolder`'s.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct RecordFacts {
    /// The system's video encoders are there; a Windows N edition without its Media Feature Pack has none.
    pub video: bool,
    /// The default microphone.
    pub mic: Gear,
    /// What the computer plays.
    pub sound: Gear,
}

/// One recording: the part, its frame rate and sounds, and when the job stops by itself.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecordAsk {
    /// The part filmed.
    pub part: Part,
    /// Frames a second, one of `FPS` (`fps`).
    pub fps: u8,
    /// The microphone records from the start, with the app's pass; `None` leaves it off.
    pub mic: Option<MicPass>,
    /// The computer's sound records from the start.
    pub sound: bool,
    /// The job stops by itself after this many minutes, keeping the video (`max_min`).
    pub max_min: u16,
    /// The job stops by itself before its drive drops under this many MB of `MB` bytes, keeping the video
    /// (`min_free_mb`).
    pub min_free_mb: u32,
    /// The job's memory cap, in MB of `MB` bytes, as every job's cap (`record_mb`).
    pub job_mb: u32,
    /// How long the job may take at each step.
    pub waits: RecordWaits,
}

impl RecordAsk {
    /// `fps`' choices, the frames' own.
    pub const FPS: [u8; 2] = kx_ipc::SNIP_FPS;
    /// `max_min`'s range, the frames' own.
    pub const MAX_MIN: RangeInclusive<u16> = kx_ipc::SNIP_MAX_MIN;
    /// `min_free_mb`'s range, the frames' own.
    pub const MIN_FREE_MB: RangeInclusive<u32> = kx_ipc::SNIP_MIN_FREE_MB;
    /// `job_mb`'s range, the frames' own.
    pub const JOB_MB: RangeInclusive<u32> = kx_ipc::SNIP_RECORD_MB;

    /// Refuses an ask with a value or a wait outside its range.
    ///
    /// # Errors
    /// `RecordError::Range` naming the first such value.
    pub fn check(&self) -> Result<(), RecordError> {
        let bad = first_bad!(self,
            fps: Self::FPS,
            max_min: Self::MAX_MIN,
            min_free_mb: Self::MIN_FREE_MB,
            job_mb: Self::JOB_MB,
        );
        refuse(bad.or_else(|| self.waits.bad()))
    }
}

/// Why a recording ended with its video kept.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum RecordEnd {
    /// Stop, or the Snip key.
    Stopped,
    /// `max_min` was reached (edge 9).
    Longest,
    /// Its drive was about to drop under `min_free_mb` (edge 8).
    Full,
}

/// The end of a recording, whose video is whole and kept.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct RecordDone {
    /// The video's length, in ms.
    pub length_ms: u32,
    /// Why it ended.
    pub end: RecordEnd,
}

/// A sound a recording hears.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Source {
    /// The microphone.
    Mic,
    /// What the computer plays.
    Sound,
}

/// One note from a recording, told in order on any thread.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum RecordNote {
    /// How long it has recorded, told about once a second; the first, at 0, says it started.
    Time {
        /// The time, in ms.
        ms: u32,
    },
    /// It ended, and its video is kept.
    Done(RecordDone),
    /// A fault of the run: it stopped, and its broken video is deleted, at the latest when the run is dropped, else by
    /// the next sweep (edge 13). A switch's fault answers `mic` or `sound` instead, and the run goes on; a switch still
    /// waiting when the run ends is settled by this last note.
    Fault(RecordError),
    /// A sound stopped by itself mid-run, such as a device unplugged; the run goes on with it off, recording silence.
    Lost(Source),
}

impl RecordNote {
    /// True for a run's last note: its `Done` or its `Fault`.
    #[must_use]
    pub const fn ends(self) -> bool {
        matches!(self, Self::Done(_) | Self::Fault(_))
    }
}

/// Why a recording or a re-timed copy failed; no variant names a path.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Error)]
pub enum RecordError {
    /// The ask's value of this name is outside its range.
    #[error("the ask's {0} is outside its range")]
    Range(&'static str),
    /// A file is no video of the Snips folder's unsaved part.
    #[error("the file is not one of the recorder's videos")]
    BadPath,
    /// A hidden run made no Snips folder.
    #[error("there is no Snips folder")]
    NoFolder,
    /// The system has no video encoders.
    #[error("the system has no video encoder")]
    NoVideo,
    /// No microphone was found.
    #[error("no microphone was found")]
    NoMic,
    /// The system's privacy switch refused the microphone.
    #[error("the system refused the microphone")]
    MicRefused,
    /// The microphone's pass is not of the app's seal, so the module never reached `MicPassKey`.
    #[error("the microphone pass is not the app's")]
    BadPass,
    /// No sound device was found for the computer's sound.
    #[error("no sound device was found")]
    NoSound,
    /// The drive holds less than `min_free_mb` before a start or a copy, or filled up while a video was written, which
    /// was then deleted.
    #[error("the drive has too little room")]
    Full,
    /// The job reached its memory cap; a host that cannot tell a cap from a crash, as the worker host, says `Gone`.
    #[error("the record job ran out of memory")]
    Memory,
    /// The job or the system failed inside.
    #[error("the record job failed")]
    Failed,
    /// The worker ended, answered badly or gave no answer in time.
    #[error("the record worker ended")]
    Gone,
}

/// One recording on the platform's side. Dropping it stops one still running and keeps its video for the next start's
/// sweep.
pub trait RecordRun: Send {
    /// The video it records into, in the Snips folder's unsaved part.
    fn file(&self) -> &SnipFile;

    /// Switches the microphone on with its pass, or off with none, at once; while off, it records silence in step with
    /// the picture (edge 12). It waits seconds at most for the job's ack or fault, so call it off the UI thread; the
    /// run goes on either way.
    ///
    /// # Errors
    /// `BadPass` for a pass its seal did not make, `NoMic` or `MicRefused` for a microphone that cannot record, `Gone`
    /// only when the job never answers, as when the run ended first; its last note then tells why.
    fn mic(&mut self, on: Option<MicPass>) -> Result<(), RecordError>;

    /// Switches the computer's sound on or off at once, waiting for the job's answer as `mic` does, off the UI thread.
    ///
    /// # Errors
    /// `NoSound` without a sound device, `Gone` only when the job never answers.
    fn sound(&mut self, on: bool) -> Result<(), RecordError>;

    /// Stops; `Done` follows with the length, and the video stays. It may wait seconds while a failed job is ended, so
    /// call it off the UI thread.
    ///
    /// # Errors
    /// `Gone` when the job does not answer, or the run already ended.
    fn stop(&mut self) -> Result<(), RecordError>;

    /// Stops at once and deletes the video, telling nothing more (edge 10), even after `stop`; once the run told its
    /// last note, its kept video stays. It never waits for the job, so the UI thread may call it.
    fn cancel(&mut self);
}

/// Records parts of the screen in the worker's `record` job; the platform provides it, holding the app's `MicSeal`.
pub trait Recorder: Send + Sync {
    /// What the system can record now; it reads the devices, so call it off the UI thread.
    fn facts(&self) -> RecordFacts;

    /// Starts recording `ask` into a new video of the unsaved part; `tell` hears its notes in order, on any thread. It
    /// waits seconds at most for the job's first note, so call it off the UI thread.
    ///
    /// # Errors
    /// `Range` when `ask.check` refuses it, `BadPass` for a pass its seal did not make, `NoFolder` in a hidden run,
    /// `NoVideo` without encoders, a sound's error for a device switched on that cannot record, `Full` under
    /// `min_free_mb`, and `Gone` when the job cannot start.
    fn start(
        &self,
        ask: RecordAsk,
        tell: Box<dyn Fn(RecordNote) + Send>,
    ) -> Result<Box<dyn RecordRun>, RecordError>;

    /// Writes `ask`'s re-timed copy into a new video of the unsaved part and names it; it may take seconds, so call it
    /// off the module's thread.
    ///
    /// # Errors
    /// `BadPath` for a `from` outside the unsaved part, `Range` when `ask.check` refuses it, `Full` under its
    /// `min_free_mb` or when the drive fills, else as `start`.
    fn speed(&self, ask: &SpeedAsk) -> Result<SnipFile, RecordError>;
}

/// The recorder service; it needs `ScreenCapture`, and Mic a pass of the app's seal, which only `Microphone` reaches
/// (ADR-0056 decision 10).
pub struct RecorderKey;

impl ServiceKey for RecorderKey {
    type Api = dyn Recorder;
    const ID: ServiceId = ServiceId::new("recorder");
    const CAPABILITY: Option<Capability> = Some(Capability::ScreenCapture);
}

/// `Range` naming `bad`, or fine when there is none.
pub(super) fn refuse(bad: Option<&'static str>) -> Result<(), RecordError> {
    bad.map_or(Ok(()), |name| Err(RecordError::Range(name)))
}
