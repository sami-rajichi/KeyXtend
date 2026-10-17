//! UI Automation on a COM thread of its own (ADR-0029 decision 3): from a point or a window, it finds the first element
//! up that scrolls along a way's axis, reads its box and percents, and takes `ScrollPattern` line steps. A read is
//! waited on for its cap at most, a step never, and at most one job waits behind the running one, so nothing piles up
//! behind a stuck app.

mod down;
mod work;

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};

use kx_module_api::ports::Dir;

use super::{Read, Seek};

/// The UI Automation thread's name.
const THREAD: &str = "kx-scroll-uia";
/// How many jobs a read may wait behind: the running one.
const BEHIND: usize = 1;
/// The walk down to a point gets the time left divided by this, and the walk up keeps the rest. The time is checked
/// only between UI Automation calls, so one call can run past it.
const DOWN_SHARE: u32 = 2;

/// The thread's body: it serves the jobs in its inbox, counting each one done.
type Body = fn(&Receiver<Job>, &AtomicUsize, u32);

/// One job for the thread; each is dropped unstarted after `by`.
enum Job {
    /// Read the box and percents of what scrolls along `way`'s axis, and answer on `reply`: none when UI Automation
    /// could not look there.
    Read {
        seek: Seek,
        way: Dir,
        by: Instant,
        reply: Sender<Option<Read>>,
    },
    /// Take `count` line steps towards `dir`.
    Steps {
        seek: Seek,
        dir: Dir,
        count: u16,
        by: Instant,
    },
}

/// The UI Automation thread; dropping it ends the thread once its running call returns.
pub(crate) struct Uia {
    jobs: Sender<Job>,
    /// Jobs given and not yet done.
    pending: Arc<AtomicUsize>,
    /// How long a step may wait to start.
    cap: Duration,
}

impl Uia {
    /// Starts the thread; UI Automation's own timeouts become `read_ms`, so a stuck call ends soon after a read gives
    /// up on it.
    pub(crate) fn start(read_ms: u32) -> Self {
        Self::spawn(read_ms, work::run)
    }

    /// Starts `body` as the thread, never waiting for it: jobs queue behind its client's start, each within its cap.
    fn spawn(read_ms: u32, body: Body) -> Self {
        let cap = Duration::from_millis(u64::from(read_ms));
        let (jobs, inbox) = mpsc::channel();
        let pending = Arc::new(AtomicUsize::new(0));
        let theirs = Arc::clone(&pending);
        let run = move || body(&inbox, &theirs, read_ms);
        if let Err(e) = thread::Builder::new().name(THREAD.to_owned()).spawn(run) {
            tracing::warn!(error = %e, "no UI Automation: its thread did not start");
        }
        Self { jobs, pending, cap }
    }

    /// The read at `seek` along `way`'s axis, waited on for `cap` at most; none when it took longer, UI Automation
    /// could not look there, or a job is stuck with another waiting behind it.
    pub(crate) fn read(&self, seek: Seek, way: Dir, cap: Duration) -> Option<Read> {
        if self.pending.load(Ordering::SeqCst) > BEHIND {
            return None;
        }
        let (reply, answer) = mpsc::channel();
        let by = deadline(cap);
        self.give(Job::Read {
            seek,
            way,
            by,
            reply,
        })?;
        answer.recv_timeout(cap).ok().flatten()
    }

    /// True while no job is given and not done, so steps can go.
    pub(crate) fn idle(&self) -> bool {
        self.pending.load(Ordering::SeqCst) == 0
    }

    /// Gives `count` line steps towards `dir` at `seek`, never waited on; false when a job is still running.
    pub(crate) fn steps(&self, seek: Seek, dir: Dir, count: u16) -> bool {
        let by = deadline(self.cap);
        self.idle()
            && self
                .give(Job::Steps {
                    seek,
                    dir,
                    count,
                    by,
                })
                .is_some()
    }

    /// Hands `job` to the thread; none when the thread has ended.
    fn give(&self, job: Job) -> Option<()> {
        self.pending.fetch_add(1, Ordering::SeqCst);
        let given = self.jobs.send(job).ok();
        if given.is_none() {
            self.pending.fetch_sub(1, Ordering::SeqCst);
        }
        given
    }
}

/// `cap` from now, or now when the clock cannot go that far.
fn deadline(cap: Duration) -> Instant {
    let now = Instant::now();
    now.checked_add(cap).unwrap_or(now)
}

#[cfg(test)]
mod tests;
