//! The watch's run, on the asker's side: each job handed to the thread and answered within `read_ms` or given up, a
//! thread stuck in a hung app skipped until it returns, and the last box, which focus in our own process keeps.

use std::sync::mpsc::{self, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex, Weak};
use std::thread;
use std::time::{Duration, Instant};

use kx_module_api::ports::{Screen, SelConfig, SelError, SelKind, SelRun, SelSpot};
use kx_module_api::{Clock, Mono, Redacted};

use super::{Inbox, Shared, lock};
use crate::deadline;

/// The watch thread's name.
const THREAD: &str = "kx-select";

/// The module's callback for each settled change: look now. It runs on the watch's thread, so it must not block or
/// call into the run.
pub type Tell = Box<dyn Fn() + Send>;

/// Where a job answers.
pub(super) type Reply<T> = Sender<T>;

/// One job for the watch's thread, which hands it to `serve`; one past its deadline makes no OS call.
pub struct Job {
    pub(super) ask: Ask,
    /// Lives as long as the job, so the run can tell when one it gave up on has ended.
    pub(super) held: Arc<()>,
}

impl Job {
    /// A job asking `ask`.
    pub(super) fn new(ask: Ask) -> Self {
        Self {
            ask,
            held: Arc::new(()),
        }
    }

    /// A handle that lives only while the job does.
    pub(super) fn alive(&self) -> Weak<()> {
        Arc::downgrade(&self.held)
    }
}

/// What a job asks, each by its deadline `by`.
pub(super) enum Ask {
    /// The focused box's spot; `None` in our own process.
    Look {
        by: Instant,
        reply: Reply<Result<Option<SelSpot>, SelError>>,
    },
    /// Whether box `box_id` still has focus and its selection holds text.
    Still {
        box_id: u64,
        by: Instant,
        reply: Reply<bool>,
    },
    /// At most `max_chars` characters of box `box_id`'s selection.
    Read {
        box_id: u64,
        max_chars: u16,
        by: Instant,
        reply: Reply<Result<Redacted<String>, SelError>>,
    },
    /// Whether the clipboard holds any format.
    Clip { by: Instant, reply: Reply<bool> },
}

/// Starts `body` as the watch's thread with `cfg`, never waiting for it: jobs queue behind its start, each within
/// `read_ms`, and `tell` hears each settled change from that thread.
///
/// # Errors
/// `Unavailable` when the thread does not start.
pub fn spawn<B>(cfg: SelConfig, tell: Tell, body: B) -> Result<Box<dyn SelRun>, SelError>
where
    B: FnOnce(Inbox, &Arc<Shared>, SelConfig) + Send + 'static,
{
    start(cfg, Arc::new(Since(Instant::now())), tell, body)
}

/// Like `spawn`, with the clock the thread paces by.
fn start<B>(
    cfg: SelConfig,
    clock: Arc<dyn Clock>,
    tell: Tell,
    body: B,
) -> Result<Box<dyn SelRun>, SelError>
where
    B: FnOnce(Inbox, &Arc<Shared>, SelConfig) + Send + 'static,
{
    let shared = Arc::new(Shared::new(cfg, clock, tell));
    let inbox = Inbox::new(Arc::clone(&shared));
    let theirs = Arc::clone(&shared);
    let started = thread::Builder::new()
        .name(THREAD.to_owned())
        .spawn(move || body(inbox, &theirs, cfg));
    if let Err(e) = started {
        tracing::warn!(error = %e, "no selection watch: its thread did not start");
        return Err(SelError::Unavailable);
    }
    Ok(Box::new(Run::new(shared, cfg)))
}

/// Monotonic time since the watch started, which its thread paces by.
struct Since(Instant);

impl Clock for Since {
    fn now(&self) -> Mono {
        Mono::default().saturating_add(self.0.elapsed())
    }
}

/// A running watch; dropping it stops its tells, and its thread ends once the running job returns.
struct Run {
    shared: Arc<Shared>,
    /// How long a job may take.
    cap: Duration,
    /// The last box a look found in another app; focus in our own process keeps it.
    last: SelSpot,
    /// The last answered look found a box that may be read: not `Closed`, and not in our own process.
    readable: bool,
    /// The job given up last at its cap, which lives while the thread is stuck in it.
    stuck: Mutex<Option<Weak<()>>>,
}

impl Run {
    /// The run of a watch whose thread shares `shared`, before any look: no box.
    fn new(shared: Arc<Shared>, cfg: SelConfig) -> Self {
        let last = SelSpot {
            box_id: 0,
            kind: SelKind::Closed,
            selected: false,
            first: None,
            last: None,
            caret: None,
            screen: Screen::default(),
        };
        Self {
            shared,
            cap: Duration::from_millis(u64::from(cfg.read_ms)),
            last,
            readable: false,
            stuck: Mutex::new(None),
        }
    }

    /// Whether the thread still runs a job given up at its cap; once it returns, jobs go through again.
    fn busy(&self) -> bool {
        let mut stuck = lock(&self.stuck);
        let busy = stuck.as_ref().is_some_and(|job| job.strong_count() > 0);
        if !busy {
            *stuck = None;
        }
        busy
    }

    /// Hands the job `make` builds to the thread and waits for its answer for the cap at most; a busy thread gets no
    /// job, and the asker hears `TimedOut` at once. A late answer is dropped unread.
    fn ask<T>(&self, make: impl FnOnce(Reply<T>, Instant) -> Ask) -> Result<T, SelError> {
        if self.busy() {
            return Err(SelError::TimedOut);
        }
        let (reply, answer) = mpsc::channel();
        let job = Job::new(make(reply, deadline(self.cap)));
        let alive = job.alive();
        self.shared.push(job)?;
        match answer.recv_timeout(self.cap) {
            Ok(got) => Ok(got),
            Err(RecvTimeoutError::Disconnected) => Err(SelError::Unavailable),
            Err(RecvTimeoutError::Timeout) => {
                *lock(&self.stuck) = Some(alive);
                Err(SelError::TimedOut)
            }
        }
    }
}

impl SelRun for Run {
    fn look(&mut self) -> Result<SelSpot, SelError> {
        let seen = self.ask(|reply, by| Ask::Look { by, reply })??;
        self.readable = seen.is_some_and(|spot| spot.kind != SelKind::Closed);
        if let Some(spot) = seen {
            self.last = spot;
        }
        Ok(self.last)
    }

    fn still(&mut self, box_id: u64) -> bool {
        self.ask(|reply, by| Ask::Still { box_id, by, reply })
            .unwrap_or(false)
    }

    fn read(&mut self, box_id: u64, max_chars: u16) -> Result<Redacted<String>, SelError> {
        if !self.readable {
            return Err(SelError::Closed);
        }
        self.ask(|reply, by| Ask::Read {
            box_id,
            max_chars,
            by,
            reply,
        })?
    }

    fn clip_holds(&self) -> bool {
        self.ask(|reply, by| Ask::Clip { by, reply })
            .unwrap_or(false)
    }
}

impl Drop for Run {
    fn drop(&mut self) {
        self.shared.stop();
    }
}

#[cfg(test)]
mod tests;
