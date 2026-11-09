//! The thread's side of the watch: the change handlers' way in, the jobs queued for the thread, and the wait between
//! them, which tells each settled change and blocks with no timeout while idle.

use std::collections::VecDeque;
use std::mem;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::{Arc, Condvar, Mutex, MutexGuard, PoisonError};
use std::time::Duration;

use kx_module_api::Clock;
use kx_module_api::ports::{SelConfig, SelError};

use super::pace::Pace;
use super::run::{Job, Tell};
use super::{Ours, lock};

/// The thread's desk: the pacing, the jobs queued, and whether the watch ended.
struct Desk {
    pace: Pace,
    jobs: VecDeque<Job>,
    /// The run dropped, the watch failed to start, or its thread ended: nothing is queued or told any more.
    closed: bool,
    /// How many timed waits the thread took, so a test proves an idle thread takes none.
    #[cfg(test)]
    timed: usize,
}

/// What the change handlers, the run and the watch's thread share.
pub struct Shared {
    desk: Mutex<Desk>,
    /// Wakes the thread for a change, a job or the end.
    wake: Condvar,
    /// The module's callback, until the run drops.
    tell: Mutex<Option<Tell>>,
    /// The time the thread paces by.
    clock: Arc<dyn Clock>,
}

impl Shared {
    /// The shared part of a watch paced as `cfg` says by `clock`, telling `tell`.
    pub(super) fn new(cfg: SelConfig, clock: Arc<dyn Clock>, tell: Tell) -> Self {
        let desk = Desk {
            pace: Pace::new(cfg),
            jobs: VecDeque::new(),
            closed: false,
            #[cfg(test)]
            timed: 0,
        };
        Self {
            desk: Mutex::new(desk),
            wake: Condvar::new(),
            tell: Mutex::new(Some(tell)),
            clock,
        }
    }

    /// A focus or selection change in `el`, told once it settles; one in our own process keeps the last box.
    pub fn moved<O: Ours>(&self, who: &O, el: &O::El) {
        if who.ours(el) {
            return;
        }
        let mut desk = lock(&self.desk);
        if !desk.closed {
            desk.pace.change(self.clock.now());
            self.wake.notify_one();
        }
    }

    /// Ends the watch after its start-up step `why` failed: it logs once, and every job answers `Unavailable`.
    pub fn fail(&self, why: &'static str) {
        let desk = lock(&self.desk);
        if !desk.closed {
            tracing::warn!(cause = why, "no selection watch: no box shows chips");
            self.close(desk);
        }
    }

    /// Queues `job` for the thread.
    ///
    /// # Errors
    /// `Unavailable` once the watch ended.
    pub(super) fn push(&self, job: Job) -> Result<(), SelError> {
        let mut desk = lock(&self.desk);
        if desk.closed {
            return Err(SelError::Unavailable);
        }
        desk.jobs.push_back(job);
        self.wake.notify_one();
        Ok(())
    }

    /// Lets go of the callback and ends the thread's wait, as the run drops.
    pub(super) fn stop(&self) {
        let gone = lock(&self.tell).take();
        drop(gone);
        self.close(lock(&self.desk));
    }

    /// Ends the watch: the queued jobs drop, so their askers hear `Unavailable`, and the thread's wait ends.
    fn close(&self, mut desk: MutexGuard<'_, Desk>) {
        desk.closed = true;
        let jobs = mem::take(&mut desk.jobs);
        drop(desk);
        drop(jobs);
        self.wake.notify_one();
    }

    /// Tells the module, unless the run dropped; a panic in its callback never ends the watch.
    fn say(&self) {
        let tell = lock(&self.tell);
        let told = tell
            .as_ref()
            .map(|tell| catch_unwind(AssertUnwindSafe(tell)));
        if told.is_some_and(|t| t.is_err()) {
            tracing::warn!("the selection watch's callback panicked");
        }
    }

    /// The next job, telling each settled change first; `None` once the watch ended.
    fn next(&self) -> Option<Job> {
        let mut desk = lock(&self.desk);
        loop {
            if desk.closed {
                return None;
            }
            let now = self.clock.now();
            if desk.pace.tell(now) {
                drop(desk);
                self.say();
                desk = lock(&self.desk);
                continue;
            }
            if let Some(job) = desk.jobs.pop_front() {
                return Some(job);
            }
            desk = match desk.pace.due() {
                None => self.wake.wait(desk).unwrap_or_else(PoisonError::into_inner),
                Some(due) => self.wait_for(desk, due.saturating_since(now)),
            };
        }
    }

    /// Waits for a change, a job or the end, at most `gap`.
    fn wait_for<'a>(&self, desk: MutexGuard<'a, Desk>, gap: Duration) -> MutexGuard<'a, Desk> {
        #[cfg(test)]
        let desk = {
            let mut desk = desk;
            desk.timed += 1;
            desk
        };
        let waited = self.wake.wait_timeout(desk, gap);
        waited.map_or_else(|e| e.into_inner().0, |(desk, _)| desk)
    }

    /// How many timed waits the thread took.
    #[cfg(test)]
    pub(super) fn timed(&self) -> usize {
        lock(&self.desk).timed
    }
}

/// The watch thread's jobs, in order: waiting for the next one tells each settled change, and they end when the run
/// drops. The watch ends when the inbox drops, so a body that returns early answers `Unavailable` from then on.
pub struct Inbox(Arc<Shared>);

impl Inbox {
    /// The inbox of the watch that shares `shared`.
    pub(super) fn new(shared: Arc<Shared>) -> Self {
        Self(shared)
    }
}

impl Iterator for Inbox {
    type Item = Job;

    fn next(&mut self) -> Option<Job> {
        self.0.next()
    }
}

impl Drop for Inbox {
    fn drop(&mut self) {
        self.0.close(lock(&self.0.desk));
    }
}

#[cfg(test)]
mod props;
