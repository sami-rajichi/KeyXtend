//! The fake's downloads: notes told on cue move the part file and name the checked file, and the live run records
//! each pause, resume and cancel.

use super::{FakeModelFiles, Log, Shared};
use crate::lock;
use kx_module_api::ports::{FetchAsk, FetchNote, FetchRun, ModelError};
use std::sync::{Arc, Mutex};

/// The callback a download tells its notes to.
pub(super) type Tell = Box<dyn Fn(FetchNote) + Send>;
/// A download's callback, shared so it is called with the log unlocked.
pub(super) type Hearer = Arc<Mutex<Tell>>;

/// What one download was asked and where it stands.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FetchLog {
    /// The ask it started with.
    pub ask: FetchAsk,
    /// `pause` calls.
    pub pauses: usize,
    /// `resume` calls.
    pub resumes: usize,
    /// `cancel` calls.
    pub cancels: usize,
    /// Paused by `pause` or a drop, until `resume`.
    pub paused: bool,
    /// A `Done` or a `Fault` was told.
    pub ended: bool,
    /// The run was dropped.
    pub dropped: bool,
}

impl FetchLog {
    /// A download just started with `ask`.
    pub(super) fn new(ask: FetchAsk) -> Self {
        Self {
            ask,
            pauses: 0,
            resumes: 0,
            cancels: 0,
            paused: false,
            ended: false,
            dropped: false,
        }
    }

    /// Whether it still hears notes.
    fn live(&self) -> bool {
        !self.ended && !self.dropped
    }
}

/// True for a fault after which the job deleted the part.
fn deletes(error: ModelError) -> bool {
    matches!(
        error,
        ModelError::Size | ModelError::Print | ModelError::Cancelled
    )
}

impl Log {
    /// Marks `note` on download `id` if it hears it: its part's bytes, its named file or its end. A paused download
    /// hears only a fault, as it reads nothing.
    fn mark(&mut self, id: usize, note: FetchNote) -> bool {
        let life = &mut self.fetches[id];
        let moves = !matches!(note, FetchNote::Fault(_));
        if !life.live() || (life.paused && moves) {
            return false;
        }
        let file = life.ask.file.expose();
        match note {
            FetchNote::Progress { done, .. } => self.tree.set_part(file, Some(done)),
            FetchNote::Done => {
                self.tree.set_part(file, None);
                self.tree.add_file(file, life.ask.bytes, life.ask.print);
            }
            FetchNote::Fault(error) if deletes(error) => self.tree.set_part(file, None),
            FetchNote::Fault(_) => {}
        }
        life.ended = !matches!(note, FetchNote::Progress { .. });
        true
    }
}

impl Shared {
    /// Tells `note` to download `id` on this thread with the log unlocked; false when it does not hear it. Its
    /// callback stays held from the marking to the telling, so a drop waits out a note being told.
    fn tell(&self, id: usize, note: FetchNote) -> bool {
        let hearer = lock(&self.log).hearers[id].clone();
        let Some(hearer) = hearer else {
            return false;
        };
        let tell = lock(&hearer);
        let heard = lock(&self.log).mark(id, note);
        self.changed.notify_all();
        if heard {
            tell(note);
        }
        heard
    }
}

impl FakeModelFiles {
    /// Tells `note` to the newest live download at once on this thread; false when none hears it, as progress while
    /// paused. `Done` names its file, a bad size or fingerprint or a cancel deletes its part, and other faults keep it.
    #[must_use]
    pub fn tell(&self, note: FetchNote) -> bool {
        let live = lock(&self.shared.log)
            .fetches
            .iter()
            .rposition(FetchLog::live);
        live.is_some_and(|id| self.shared.tell(id, note))
    }

    /// Tells the newest live download that `done` of its file's bytes are written, keeping them in its part; false as
    /// `tell`.
    #[must_use]
    pub fn progress(&self, done: u64) -> bool {
        let log = lock(&self.shared.log);
        let total = log.fetches.iter().rev().find(|f| f.live());
        let Some(total) = total.map(|f| f.ask.bytes) else {
            return false;
        };
        drop(log);
        self.tell(FetchNote::Progress { done, total })
    }
}

/// A live fake download; dropping it pauses it.
pub(super) struct FakeFetchRun {
    pub(super) shared: Arc<Shared>,
    /// Its place in the log's downloads.
    pub(super) id: usize,
}

impl FakeFetchRun {
    /// Changes this download's log as `change` says.
    fn note(&self, change: impl FnOnce(&mut FetchLog)) {
        change(&mut lock(&self.shared.log).fetches[self.id]);
        self.shared.changed.notify_all();
    }
}

impl FetchRun for FakeFetchRun {
    fn pause(&mut self) {
        self.note(|f| {
            f.pauses += 1;
            f.paused = true;
        });
    }

    fn resume(&mut self) {
        self.note(|f| {
            f.resumes += 1;
            f.paused = false;
        });
    }

    /// Deletes the part and, while the download runs, tells `Fault(Cancelled)` as the job does.
    fn cancel(&mut self) {
        let mut log = lock(&self.shared.log);
        let life = &mut log.fetches[self.id];
        life.cancels += 1;
        life.paused = false;
        let file = life.ask.file.expose().clone();
        log.tree.set_part(&file, None);
        drop(log);
        self.shared
            .tell(self.id, FetchNote::Fault(ModelError::Cancelled));
    }
}

impl Drop for FakeFetchRun {
    /// Pauses a live download and waits out a note being told on another thread, so none comes after.
    fn drop(&mut self) {
        let mut log = lock(&self.shared.log);
        let life = &mut log.fetches[self.id];
        life.paused |= life.live();
        life.dropped = true;
        let gone = log.hearers[self.id].take();
        drop(log);
        self.shared.changed.notify_all();
        if let Some(hearer) = gone {
            drop(lock(&hearer));
        }
    }
}
