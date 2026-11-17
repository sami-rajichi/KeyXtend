//! The fake's log of runs and what the test set, and the one path every note takes: a run's callback is held while
//! its answers are taken and told, so a drop waits out a note being told and none comes after it.

use super::{CloudAnswer, CloudCalls, CloudLog, CloudState, Tell};
use crate::lock;
use kx_module_api::Secret;
use kx_module_api::ports::{CloudAsk, CloudFault, CloudNote, CloudRun};
use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex, MutexGuard, PoisonError};

/// A run's callback, shared so it is called with the log unlocked.
type Hearer = Arc<Mutex<Tell>>;

/// A run's answers not yet told, and its callback until it is dropped.
struct Ear {
    script: VecDeque<CloudAnswer>,
    hearer: Option<Hearer>,
}

/// What the runs and tests did, and what the test set for them.
#[derive(Default)]
pub(super) struct Log {
    /// The runs and tests, as the test reads them.
    pub(super) calls: CloudCalls,
    /// Each run's ear, by its place in `calls.runs`.
    ears: Vec<Ear>,
    /// The scripts the next runs take, in turn.
    pub(super) scripts: VecDeque<VecDeque<CloudAnswer>>,
    /// The answers the next tests give, in turn.
    pub(super) test_answers: VecDeque<Result<(), CloudFault>>,
    /// The fault the next `send` returns.
    pub(super) fail: Option<CloudFault>,
    /// Tests wait inside `test` while this holds.
    pub(super) held: bool,
    /// The key the service takes; `None` takes any.
    pub(super) key: Option<Secret>,
}

impl Log {
    /// Records a run of `ask` that tells `tell` and gives its place; it takes the next script, led by `Fault(Key)`
    /// when the service refuses its key.
    pub(super) fn open(&mut self, ask: CloudAsk, tell: Tell) -> usize {
        let mut script = self.scripts.pop_front().unwrap_or_default();
        if !self.key_ok(&ask.key) {
            script.push_front(CloudAnswer::Fault(CloudFault::Key));
        }
        self.calls.runs.push(CloudLog::new(ask));
        let hearer = Some(Arc::new(Mutex::new(tell)));
        self.ears.push(Ear { script, hearer });
        self.calls.runs.len() - 1
    }

    /// True when the service takes `key`.
    pub(super) fn key_ok(&self, key: &Secret) -> bool {
        let same = |own: &Secret| own.expose().as_str() == key.expose().as_str();
        self.key.as_ref().is_none_or(same)
    }

    /// Adds `more` to run `id`'s answers and takes the notes it tells now: texts in order, up to its end or its first
    /// piece with no answer; none once it ended or was dropped.
    pub(super) fn take(&mut self, id: usize, more: Vec<CloudAnswer>) -> Vec<CloudNote> {
        let (Some(run), Some(ear)) = (self.calls.runs.get_mut(id), self.ears.get_mut(id)) else {
            return Vec::new();
        };
        ear.script.extend(more);
        let mut notes = Vec::new();
        while run.state == CloudState::Sending {
            if run.told == run.sizes.len() {
                run.state = CloudState::Done;
                notes.push(CloudNote::Done);
            } else if let Some(answer) = ear.script.pop_front() {
                notes.push(run.note(answer));
            } else {
                break;
            }
        }
        if run.state != CloudState::Sending {
            ear.script.clear();
        }
        notes
    }

    /// Ends run `id` on `fault` while it sends, as a cut connection does, and gives the note; none otherwise.
    pub(super) fn cut(&mut self, id: usize, fault: CloudFault) -> Vec<CloudNote> {
        let (Some(run), Some(ear)) = (self.calls.runs.get_mut(id), self.ears.get_mut(id)) else {
            return Vec::new();
        };
        if run.state != CloudState::Sending {
            return Vec::new();
        }
        ear.script.clear();
        vec![run.end(fault)]
    }

    /// Marks run `id` dropped, cancelling its unanswered pieces while it sends, and gives back its callback.
    fn drop_run(&mut self, id: usize) -> Option<Hearer> {
        let run = &mut self.calls.runs[id];
        if run.state == CloudState::Sending {
            run.state = CloudState::Dropped;
        }
        let ear = &mut self.ears[id];
        ear.script.clear();
        ear.hearer.take()
    }
}

/// The log, and the signal each change gives, for a test waiting on another thread.
#[derive(Default)]
pub(super) struct Shared {
    pub(super) log: Mutex<Log>,
    pub(super) changed: Condvar,
}

impl Shared {
    /// Tells run `id` the notes `take` gives, in order, on this thread with the log unlocked; false when none. Its
    /// callback stays held from the taking to the last note, so notes keep their order across threads.
    pub(super) fn tell(&self, id: usize, take: impl FnOnce(&mut Log) -> Vec<CloudNote>) -> bool {
        let hearer = lock(&self.log).ears.get(id).and_then(|e| e.hearer.clone());
        let Some(hearer) = hearer else {
            return false;
        };
        let tell = lock(&hearer);
        let notes = take(&mut lock(&self.log));
        self.changed.notify_all();
        let told = !notes.is_empty();
        for note in notes {
            tell(note);
        }
        told
    }

    /// Waits while key tests are held.
    pub(super) fn released<'a>(&self, log: MutexGuard<'a, Log>) -> MutexGuard<'a, Log> {
        let released = self.changed.wait_while(log, |l| l.held);
        released.unwrap_or_else(PoisonError::into_inner)
    }
}

/// A live fake run; dropping it cancels its unanswered pieces.
pub(super) struct FakeCloudRun {
    pub(super) shared: Arc<Shared>,
    /// Its place in the log's runs.
    pub(super) id: usize,
}

impl CloudRun for FakeCloudRun {}

/// Waits out a note being told on another thread, so none comes after the drop returns.
impl Drop for FakeCloudRun {
    fn drop(&mut self) {
        let gone = lock(&self.shared.log).drop_run(self.id);
        self.shared.changed.notify_all();
        if let Some(hearer) = gone {
            drop(lock(&hearer));
        }
    }
}
