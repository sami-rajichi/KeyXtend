//! A fake recorder: it checks each ask in the port's order, holding the app's seal, writes each recording into the
//! Snips folder's table, and logs what each one heard over time by the folder's clock. The test tells its notes, kills
//! its worker and fails a call once, with no sleep; a `tell` must not call back into the fake.

mod gear;
mod log;
mod run;

use super::folder::{FakeSnipFolder, Stored};
use crate::{lock, wait_for};
use gear::{able, folder_error};
use kx_module_api::ports::{
    Gear, MB, MicSeal, RecordAsk, RecordDone, RecordEnd, RecordError, RecordFacts, RecordNote,
    RecordRun, Recorder, SnipFile, SnipFolder, SpeedAsk,
};
use log::{Log, Run, Shared, Tell, ms};
use run::FakeRecordRun;
use std::fmt;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

/// One recorder call, to fail it once.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum RecordCall {
    /// `Recorder::start`.
    Start,
    /// `RecordRun::mic`.
    Mic,
    /// `RecordRun::sound`.
    Sound,
    /// `RecordRun::stop`.
    Stop,
    /// `Recorder::speed`.
    Speed,
}

/// What a recording hears from `ms` on, so a test sees each switch in step with the picture (edge 12).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Heard {
    /// Since its start, in ms by the clock.
    pub ms: u32,
    /// The microphone is on.
    pub mic: bool,
    /// The computer's sound is on.
    pub sound: bool,
}

/// What one recording was asked, what it heard and where it stands.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecLog {
    /// The ask it started with.
    pub ask: RecordAsk,
    /// Its video in the unsaved part.
    pub file: SnipFile,
    /// What it heard from its start, then after each switch taken.
    pub heard: Vec<Heard>,
    /// The notes it was told, in order.
    pub told: Vec<RecordNote>,
    /// `stop` calls taken.
    pub stops: usize,
    /// `cancel` calls; one after the end leaves the kept video alone.
    pub cancels: usize,
    /// It still records: until a `Done`, a `Fault`, a cancel or a drop.
    pub live: bool,
    /// The module dropped it.
    pub dropped: bool,
}

/// A recorder shared by its clones, with video, microphone and sound ready until set. A new video takes the next
/// count-named file (`1.mp4`, …) not already held, so it never replaces one.
#[derive(Clone)]
pub struct FakeRecorder {
    shared: Arc<Shared>,
    /// The app's seal, which alone opens a microphone pass.
    seal: MicSeal,
    folder: FakeSnipFolder,
}

impl FakeRecorder {
    /// A recorder holding the app's `seal`, recording into `folder`'s unsaved part by its clock.
    #[must_use]
    pub fn new(seal: MicSeal, folder: &FakeSnipFolder) -> Self {
        let (mic, sound) = (Gear::Ready, Gear::Ready);
        let facts = RecordFacts {
            video: true,
            mic,
            sound,
        };
        let (log, changed) = (Mutex::new(Log::new(facts)), Condvar::new());
        let shared = Arc::new(Shared { log, changed });
        let folder = folder.clone();
        Self {
            shared,
            seal,
            folder,
        }
    }

    /// Sets what the system can record now, for `facts` and every later check (edges 11 and 14).
    pub fn set_facts(&self, facts: RecordFacts) {
        lock(&self.shared.log).facts = facts;
    }

    /// Fails the next `call` with `error`, once, changing nothing else; `die` is the worker's death.
    pub fn fail(&self, call: RecordCall, error: RecordError) {
        lock(&self.shared.log).fails.insert(call, error);
    }

    /// Tells `note` to the newest live recording at once on this thread; false when none hears it. A `Done` keeps its
    /// video at that length; a `Fault` deletes it.
    #[must_use]
    pub fn tell(&self, note: RecordNote) -> bool {
        self.to_newest(|_| note)
    }

    /// Tells the newest live recording its time so far by the clock; false when none hears it.
    #[must_use]
    pub fn time(&self) -> bool {
        self.to_newest(|ms| RecordNote::Time { ms })
    }

    /// Ends the newest live recording for `end` with its length so far by the clock, keeping its video (edges 8, 9).
    #[must_use]
    pub fn end(&self, end: RecordEnd) -> bool {
        self.to_newest(|length_ms| RecordNote::Done(RecordDone { length_ms, end }))
    }

    /// Kills the worker under the newest live recording: it tells `Fault(Gone)` and deletes the video (edge 13).
    #[must_use]
    pub fn die(&self) -> bool {
        self.tell(RecordNote::Fault(RecordError::Gone))
    }

    /// Every start asked, refused ones too, oldest first.
    #[must_use]
    pub fn starts(&self) -> Vec<RecordAsk> {
        lock(&self.shared.log).starts.clone()
    }

    /// Each recording started, oldest first.
    #[must_use]
    pub fn runs(&self) -> Vec<RecLog> {
        lock(&self.shared.log).logs()
    }

    /// Every speed copy asked, refused ones too, oldest first.
    #[must_use]
    pub fn speeds(&self) -> Vec<SpeedAsk> {
        lock(&self.shared.log).speeds.clone()
    }

    /// Waits up to `within` until `until` holds for the recordings, so a test never sleeps; true when it does.
    #[must_use]
    pub fn wait(&self, within: Duration, until: impl Fn(&[RecLog]) -> bool) -> bool {
        let shared = &self.shared;
        wait_for(&shared.log, &shared.changed, within, |l| until(&l.logs()))
    }

    /// Waits up to `within` until `count` speed copies were answered, refused ones too; true when they were.
    #[must_use]
    pub fn wait_speeds(&self, count: usize, within: Duration) -> bool {
        let shared = &self.shared;
        wait_for(&shared.log, &shared.changed, within, |l| {
            l.speeds.len() >= count
        })
    }

    /// Tells the newest live recording the note `make` builds from its time so far; false when none hears it.
    fn to_newest(&self, make: impl FnOnce(u32) -> RecordNote) -> bool {
        let now = self.folder.now();
        let newest = {
            let log = lock(&self.shared.log);
            log.newest().map(|id| (id, ms(log.runs[id].at, now)))
        };
        newest.is_some_and(|(id, so_far)| self.shared.tell(id, make(so_far), &self.folder))
    }

    /// Refuses a start or a copy whose drive holds less than `min_free_mb` (edge 8).
    fn room(&self, min_free_mb: u32) -> Result<(), RecordError> {
        let free = self.folder.free();
        let free = free.map_err(|e| folder_error(e, RecordError::Failed))?;
        let least = u64::from(min_free_mb) * MB;
        (free >= least).then_some(()).ok_or(RecordError::Full)
    }

    /// Checks `ask` in the port's order, then writes its new video and logs it as recording `id`.
    fn begin(&self, ask: RecordAsk, tell: Tell) -> Result<(usize, SnipFile), RecordError> {
        ask.check()?;
        if ask.mic.as_ref().is_some_and(|pass| !self.seal.opens(pass)) {
            return Err(RecordError::BadPass);
        }
        let unsaved = self.folder.unsaved_dir().ok_or(RecordError::NoFolder)?;
        self.room(ask.min_free_mb)?;
        let mut log = lock(&self.shared.log);
        able(log.facts, &ask)?;
        log.failed(RecordCall::Start)?;
        let file = log.name(&unsaved, |f| self.folder.stored(f).is_some())?;
        let empty = Stored::Video { length_ms: 0 };
        self.folder.put(file.clone(), empty);
        let id = log.runs.len();
        let run = Run::new(ask, file.clone(), self.folder.now(), tell);
        log.runs.push(run);
        Ok((id, file))
    }

    /// Writes `ask`'s copy, checked in the port's order, into a new video of the unsaved part.
    fn copy(&self, log: &mut Log, ask: &SpeedAsk) -> Result<SnipFile, RecordError> {
        ask.check()?;
        let unsaved = self.folder.unsaved_dir().ok_or(RecordError::NoFolder)?;
        let from = self.folder.unsaved_video(&ask.from);
        let length = from.map_err(|e| folder_error(e, RecordError::BadPath))?;
        self.room(ask.min_free_mb)?;
        if !log.facts.video {
            return Err(RecordError::NoVideo);
        }
        log.failed(RecordCall::Speed)?;
        let file = log.name(&unsaved, |f| self.folder.stored(f).is_some())?;
        let length_ms = length.div_ceil(u32::from(ask.speed.times()));
        self.folder.put(file.clone(), Stored::Video { length_ms });
        Ok(file)
    }
}

/// Shows the counts, never a path.
impl fmt::Debug for FakeRecorder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let log = lock(&self.shared.log);
        let live = log.runs.iter().filter(|r| r.log.live).count();
        f.debug_struct("FakeRecorder")
            .field("starts", &log.starts.len())
            .field("runs", &log.runs.len())
            .field("live", &live)
            .field("speeds", &log.speeds.len())
            .finish_non_exhaustive()
    }
}

impl Recorder for FakeRecorder {
    fn facts(&self) -> RecordFacts {
        lock(&self.shared.log).facts
    }

    fn start(&self, ask: RecordAsk, tell: Tell) -> Result<Box<dyn RecordRun>, RecordError> {
        lock(&self.shared.log).starts.push(ask.clone());
        let begun = self.begin(ask, tell);
        self.shared.changed.notify_all();
        let (id, file) = begun?;
        let (shared, folder) = (Arc::clone(&self.shared), self.folder.clone());
        let seal = self.seal.clone();
        let run = FakeRecordRun {
            shared,
            folder,
            seal,
            id,
            file,
        };
        Ok(Box::new(run))
    }

    /// The copy's length is the source's divided by the speed, rounded up, so a very short one still gives one.
    fn speed(&self, ask: &SpeedAsk) -> Result<SnipFile, RecordError> {
        let mut log = lock(&self.shared.log);
        log.speeds.push(ask.clone());
        let made = self.copy(&mut log, ask);
        drop(log);
        self.shared.changed.notify_all();
        made
    }
}

#[cfg(test)]
mod tests;
