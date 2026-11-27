//! `Recorder` over the worker's `record` job (ADR-0056 decision 3): each recording and each re-timed copy runs in a job
//! of its own on a stream (ADR-0042), capped at `job_mb`, into a new video of the Snips folder's unsaved part. A run's
//! notes reach its callback in the job's order; a lost job ends the run with one fault and its broken video goes.

mod copy;
mod name;
mod route;
mod run;

use super::unsaved_part;
use crate::lock;
use crate::worker::{Launch, NEVER_IDLE_S, RecordFrames, Spawner, Split, Times, Unsent, ms};
use kx_ipc::{RECORD_JOB, RecordFault};
use kx_module_api::Clock;
use kx_module_api::ports::{
    FolderPath, MB, MicSeal, RecordAsk, RecordError, RecordFacts, RecordNote, RecordRun,
    RecordWaits, Recorder, SaveError, SnipFile, SnipFolder, SpeedAsk,
};
pub(crate) use name::fresh;
use std::borrow::Cow;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

/// A run's or a copy's job is never started again: a lost one ends its run, and the next ask starts a new job.
const NO_RESTART_MS: u32 = 0;

/// The stream times of one run's or copy's job: goodbye waits `end_ms`, and it never idles out or starts again.
const fn times(waits: &RecordWaits) -> Times {
    Times {
        deadline_ms: waits.end_ms,
        idle_s: NEVER_IDLE_S,
        restart_ms: NO_RESTART_MS,
        restart_max_ms: NO_RESTART_MS,
    }
}

impl Launch {
    /// The record job capped at `job_mb` (`record_mb`).
    #[must_use]
    pub fn record(job_mb: u32) -> Self {
        Self {
            args: vec![RECORD_JOB.to_owned()],
            memory_mb: job_mb,
        }
    }
}

/// What the system can record with now, read without loading any codec; the platform provides it.
pub trait Kit: Send + Sync {
    /// The encoders and sound devices there now (edges 11 and 14); it reads the devices, so call it off the UI thread.
    fn facts(&self) -> RecordFacts;
}

/// Recordings by the worker `S` starts, into `folder`'s unsaved part, holding the app's microphone seal.
pub struct WorkerRecorder<S> {
    spawner: Arc<S>,
    clock: Arc<dyn Clock>,
    seal: MicSeal,
    folder: Arc<dyn SnipFolder>,
    kit: Arc<dyn Kit>,
    /// The last count named, so two videos named in the same second differ before either file is made.
    last: Mutex<u64>,
}

impl<S: Spawner<RecordFrames, Worker: Split<RecordFrames>>> WorkerRecorder<S> {
    /// Recordings over `spawner` on `clock`'s time, opening only `seal`'s passes, into `folder`, with `kit`'s facts.
    #[must_use]
    pub fn new(
        spawner: S,
        clock: Arc<dyn Clock>,
        seal: MicSeal,
        folder: Arc<dyn SnipFolder>,
        kit: Arc<dyn Kit>,
    ) -> Self {
        Self {
            spawner: Arc::new(spawner),
            clock,
            seal,
            folder,
            kit,
            last: Mutex::new(0),
        }
    }

    /// The unsaved part; `NoFolder` in a hidden run.
    fn unsaved(&self) -> Result<FolderPath, RecordError> {
        unsaved_part(&*self.folder).ok_or(RecordError::NoFolder)
    }

    /// Refuses a start or a copy while the drive holds less than `min_free_mb` (edge 8).
    fn room(&self, min_free_mb: u32) -> Result<(), RecordError> {
        let free = self.folder.free().map_err(folder_error)?;
        let least = u64::from(min_free_mb).saturating_mul(MB);
        (free >= least).then_some(()).ok_or(RecordError::Full)
    }

    /// A new video in `unsaved`, counted from the wall clock's seconds past the last one named and every one held.
    fn fresh(&self, unsaved: &FolderPath) -> Result<SnipFile, RecordError> {
        let listed = self.folder.unsaved().unwrap_or_default();
        let held: Vec<SnipFile> = listed.into_iter().map(|u| u.file).collect();
        let now = SystemTime::now().duration_since(UNIX_EPOCH);
        let mut last = lock(&self.last);
        let from = now.map_or(0, |d| d.as_secs()).max(last.saturating_add(1));
        let (count, file) = fresh(unsaved, from, &held).ok_or(RecordError::Failed)?;
        *last = count;
        Ok(file)
    }
}

impl<S: Spawner<RecordFrames, Worker: Split<RecordFrames>>> Recorder for WorkerRecorder<S> {
    fn facts(&self) -> RecordFacts {
        self.kit.facts()
    }

    fn start(
        &self,
        ask: RecordAsk,
        tell: Box<dyn Fn(RecordNote) + Send>,
    ) -> Result<Box<dyn RecordRun>, RecordError> {
        ask.check()?;
        if ask.mic.as_ref().is_some_and(|pass| !self.seal.opens(pass)) {
            return Err(RecordError::BadPass);
        }
        let unsaved = self.unsaved()?;
        self.room(ask.min_free_mb)?;
        let file = self.fresh(&unsaved)?;
        let run = run::WorkerRun::start(self, &ask, file, tell)?;
        Ok(Box::new(run))
    }

    fn speed(&self, ask: &SpeedAsk) -> Result<SnipFile, RecordError> {
        copy::copy(self, ask)
    }
}

/// Deletes `file` from the unsaved part; false when the folder refused, as while its job still holds it.
fn scrap(folder: &dyn SnipFolder, file: &SnipFile) -> bool {
    let removed = folder.remove(file);
    removed
        .inspect_err(|e| tracing::debug!(error = %e, "a broken video is still held"))
        .is_ok()
}

/// Deletes `file` again once its job ended; one still refused is left for the sweep.
fn retry(folder: &dyn SnipFolder, file: &SnipFile) {
    if !scrap(folder, file) {
        tracing::warn!("a broken video was left for the sweep");
    }
}

/// `file`'s path as the frames carry it; `Failed` for one they cannot, which a folder `FolderPath` took never is.
fn plain(file: &SnipFile) -> Result<Cow<'static, str>, RecordError> {
    let path = file.expose().to_str().ok_or(RecordError::Failed)?;
    Ok(Cow::Owned(path.to_owned()))
}

/// `NoFolder` for a hidden run's folder error, else `Failed`.
fn folder_error(error: SaveError) -> RecordError {
    if error == SaveError::NoFolder {
        RecordError::NoFolder
    } else {
        RecordError::Failed
    }
}

/// Why an ask was not sent: `check` already holds the frames' rules, so a refusal is a failure; else the job is gone.
const fn unsent(e: Unsent) -> RecordError {
    match e {
        Unsent::Refused(_) => RecordError::Failed,
        Unsent::Lost(_) => RecordError::Gone,
    }
}

/// The port's error for the job's fault.
const fn error(fault: RecordFault) -> RecordError {
    match fault {
        RecordFault::NoMic => RecordError::NoMic,
        RecordFault::MicRefused => RecordError::MicRefused,
        RecordFault::NoSound => RecordError::NoSound,
        RecordFault::NoVideo => RecordError::NoVideo,
        RecordFault::Full => RecordError::Full,
        RecordFault::Failed => RecordError::Failed,
    }
}

#[cfg(test)]
mod tests;
