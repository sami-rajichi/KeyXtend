//! The watch's run: the box told last, the module's callback until the run drops, and reads handed to the watch's
//! thread, each answered within `read_ms` or given up.

use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::thread;
use std::time::{Duration, Instant};

use kx_module_api::Redacted;
use kx_module_api::ports::{
    FieldChange, FieldConfig, FieldEnds, FieldError, FieldKind, FieldRead, FieldRun,
};

use super::Probe;
use crate::deadline;

/// The watch thread's name.
const THREAD: &str = "kx-field";

/// The module's callback for each focus change.
pub type Tell = Box<dyn Fn(FieldChange) + Send>;

/// Where a read answers: what it read, `None` when focus is in our own process or the box does not tell.
pub(super) type Reply<T> = Sender<Result<Option<T>, FieldError>>;

/// One read for the watch's thread, which hands it to `serve`; one past its deadline is answered `TimedOut` unstarted.
pub struct Job(pub(super) Ask);

/// What a job asks.
pub(super) enum Ask {
    /// At most `max_words` words before the caret.
    Before {
        max_words: u8,
        by: Instant,
        reply: Reply<FieldRead>,
    },
    /// Whether the text before the caret ends with `text`.
    Ends {
        text: Redacted<String>,
        by: Instant,
        reply: Reply<FieldEnds>,
    },
}

/// The box told last, how many focus changes were told, and whether the watch failed to start.
struct State {
    last: FieldChange,
    count: u64,
    dead: bool,
}

/// What the focus handler and the run share: the state, and the callback until the run drops.
pub struct Shared {
    state: Mutex<State>,
    tell: Mutex<Option<Tell>>,
}

/// Locks `m`, taking the value even when a panicking thread poisoned it.
fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(PoisonError::into_inner)
}

impl Shared {
    /// Keeps and tells box `el`, which took the focus; focus in our own process keeps the last box.
    pub fn moved<P: Probe>(&self, probe: &P, el: &P::El) {
        if !probe.ours(el) {
            self.told(probe.change(el));
        }
    }

    /// Like `moved`, unless a focus change was told already, so the start-up look never overwrites a newer event.
    pub fn first<P: Probe>(&self, probe: &P, el: &P::El) {
        if !probe.ours(el) {
            self.tell_if(probe.change(el), |state| state.count == 0);
        }
    }

    /// Marks the watch dead after the step `why` failed at start-up: it logs and tells `Unknown` once, and no read
    /// runs from then on.
    pub fn fail(&self, why: &'static str) {
        let tell = lock(&self.tell);
        let mut state = lock(&self.state);
        if state.dead {
            return;
        }
        tracing::warn!(cause = why, "no field watch: every box reads as unknown");
        state.dead = true;
        state.last.kind = FieldKind::Unknown;
        let change = state.last;
        drop(state);
        if let Some(tell) = tell.as_ref() {
            tell(change);
        }
    }

    /// The kind of the box told last; `Unknown` once the watch is dead.
    pub fn kind(&self) -> FieldKind {
        lock(&self.state).last.kind
    }

    /// Keeps `change` as the box with focus and tells it; a dead watch or a dropped run does neither.
    fn told(&self, change: FieldChange) {
        self.tell_if(change, |_| true);
    }

    /// Keeps and tells `change` when the watch lives, the run holds the callback, and `when` agrees.
    fn tell_if(&self, change: FieldChange, when: impl FnOnce(&State) -> bool) {
        let tell = lock(&self.tell);
        let Some(tell) = tell.as_ref() else {
            return;
        };
        let mut state = lock(&self.state);
        if state.dead || !when(&state) {
            return;
        }
        state.last = change;
        state.count = state.count.saturating_add(1);
        drop(state);
        tell(change);
    }

    /// Whether the watch failed to start.
    fn dead(&self) -> bool {
        lock(&self.state).dead
    }

    /// Lets go of the callback, so nothing is told or kept from now on.
    fn stop(&self) {
        let gone = lock(&self.tell).take();
        drop(gone);
    }
}

/// Starts `body` as the watch's thread with `cfg`, never waiting for it: reads queue behind its start, each within
/// `read_ms`.
///
/// # Errors
/// `Unavailable` when the thread does not start.
pub fn spawn<B>(cfg: FieldConfig, tell: Tell, body: B) -> Result<Box<dyn FieldRun>, FieldError>
where
    B: FnOnce(&Receiver<Job>, &Arc<Shared>, FieldConfig) + Send + 'static,
{
    let none = FieldChange {
        kind: FieldKind::None,
        box_id: 0,
    };
    let state = State {
        last: none,
        count: 0,
        dead: false,
    };
    let shared = Arc::new(Shared {
        state: Mutex::new(state),
        tell: Mutex::new(Some(tell)),
    });
    let (jobs, inbox) = mpsc::channel();
    let theirs = Arc::clone(&shared);
    let started = thread::Builder::new()
        .name(THREAD.to_owned())
        .spawn(move || body(&inbox, &theirs, cfg));
    if let Err(e) = started {
        tracing::warn!(error = %e, "no field watch: its thread did not start");
        return Err(FieldError::Unavailable);
    }
    let cap = Duration::from_millis(u64::from(cfg.read_ms));
    Ok(Box::new(Run {
        shared,
        jobs,
        cap,
        cfg,
    }))
}

/// A running watch; dropping it stops its tells, and its thread ends once the running read returns.
struct Run {
    shared: Arc<Shared>,
    jobs: Sender<Job>,
    /// How long a read may take.
    cap: Duration,
    /// The watch's config, whose `read_chars` bounds an end `ends_with` checks.
    cfg: FieldConfig,
}

impl Run {
    /// Hands the read `make` builds to the thread and waits for its answer for the cap at most.
    fn ask<T>(&self, make: impl FnOnce(Reply<T>, Instant) -> Ask) -> Result<Option<T>, FieldError> {
        let (reply, answer) = mpsc::channel();
        let job = Job(make(reply, deadline(self.cap)));
        self.jobs.send(job).map_err(|_| FieldError::Unavailable)?;
        answer.recv_timeout(self.cap).unwrap_or_else(|e| {
            Err(match e {
                RecvTimeoutError::Timeout => FieldError::TimedOut,
                RecvTimeoutError::Disconnected => FieldError::Unavailable,
            })
        })
    }

    /// Whether the box told last may be read: only a text box is.
    fn readable(&self) -> bool {
        self.shared.kind().reads()
    }
}

impl FieldRun for Run {
    fn kind(&self) -> FieldKind {
        self.shared.kind()
    }

    fn before_cursor(&mut self, max_words: u8) -> Result<FieldRead, FieldError> {
        if self.shared.dead() {
            return Err(FieldError::Unavailable);
        }
        let told = FieldRead {
            kind: self.shared.kind(),
            text: None,
        };
        if !self.readable() {
            return Ok(told);
        }
        let read = self.ask(|reply, by| Ask::Before {
            max_words,
            by,
            reply,
        })?;
        Ok(read.unwrap_or(told))
    }

    fn ends_with(&mut self, text: &Redacted<String>) -> FieldEnds {
        let kind = self.shared.kind();
        if !kind.reads() {
            return FieldEnds {
                kind,
                ends: Some(false),
            };
        }
        let unsure = FieldEnds { kind, ends: None };
        if !self.cfg.fits(text.expose()) {
            return unsure;
        }
        let text = text.clone();
        let ended = self.ask(|reply, by| Ask::Ends { text, by, reply });
        ended.ok().flatten().unwrap_or(unsure)
    }
}

impl Drop for Run {
    fn drop(&mut self) {
        self.shared.stop();
    }
}

#[cfg(test)]
mod tests;
