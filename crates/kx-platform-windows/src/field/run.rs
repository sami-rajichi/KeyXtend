//! The watch's run: the box told last, the module's callback until the run drops, and reads handed to the watch's
//! thread, each answered within `read_ms` or given up.

use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use kx_module_api::Redacted;
use kx_module_api::ports::{FieldChange, FieldConfig, FieldError, FieldKind, FieldRun};

use crate::{deadline, lock};

/// The watch thread's name.
const THREAD: &str = "kx-field-uia";

/// The module's callback for each focus change.
pub(crate) type Tell = Box<dyn Fn(FieldChange) + Send>;

/// Where a read answers: what it read, `None` when the box does not tell.
pub(super) type Reply<T> = Sender<Result<Option<T>, FieldError>>;

/// One read for the watch's thread; one past `by` is answered `TimedOut` unstarted.
pub(super) enum Job {
    /// At most `max_words` words before the caret.
    Before {
        max_words: u8,
        by: Instant,
        reply: Reply<String>,
    },
    /// Whether the text before the caret ends with `text`.
    Ends {
        text: Redacted<String>,
        by: Instant,
        reply: Reply<bool>,
    },
}

/// What the focus handler and the run share: the box told last, and the callback until the run drops.
pub(super) struct Shared {
    last: Mutex<FieldChange>,
    tell: Mutex<Option<Tell>>,
}

impl Shared {
    /// Keeps `change` as the box with focus and tells it; once the run has dropped it does neither.
    pub(super) fn told(&self, change: FieldChange) {
        let tell = lock(&self.tell);
        if let Some(tell) = tell.as_ref() {
            *lock(&self.last) = change;
            tell(change);
        }
    }

    /// The kind of the box told last.
    pub(super) fn kind(&self) -> FieldKind {
        lock(&self.last).kind
    }

    /// Lets go of the callback, so nothing is told or kept from now on.
    fn stop(&self) {
        let gone = lock(&self.tell).take();
        drop(gone);
    }
}

/// Starts `body` as the watch's thread, never waiting for it: reads queue behind its start, each within its cap.
///
/// # Errors
/// `Unavailable` when the thread does not start.
pub(super) fn spawn<B>(
    cfg: FieldConfig,
    tell: Tell,
    body: B,
) -> Result<Box<dyn FieldRun>, FieldError>
where
    B: FnOnce(&Receiver<Job>, &Arc<Shared>, u32) + Send + 'static,
{
    let none = FieldChange {
        kind: FieldKind::None,
        box_id: 0,
    };
    let shared = Arc::new(Shared {
        last: Mutex::new(none),
        tell: Mutex::new(Some(tell)),
    });
    let (jobs, inbox) = mpsc::channel();
    let theirs = Arc::clone(&shared);
    let read_ms = cfg.read_ms;
    let started = thread::Builder::new()
        .name(THREAD.to_owned())
        .spawn(move || body(&inbox, &theirs, read_ms));
    if let Err(e) = started {
        tracing::warn!(error = %e, "no field watch: its thread did not start");
        return Err(FieldError::Unavailable);
    }
    let cap = Duration::from_millis(u64::from(read_ms));
    Ok(Box::new(Run { shared, jobs, cap }))
}

/// A running watch; dropping it stops its tells, and its thread ends once the running read returns.
struct Run {
    shared: Arc<Shared>,
    jobs: Sender<Job>,
    /// How long a read may take.
    cap: Duration,
}

impl Run {
    /// Hands the job `make` builds to the thread and waits for its answer for the cap at most.
    fn ask<T>(&self, make: impl FnOnce(Reply<T>, Instant) -> Job) -> Result<Option<T>, FieldError> {
        let (reply, answer) = mpsc::channel();
        let job = make(reply, deadline(self.cap));
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
        self.shared.kind() == FieldKind::Text
    }
}

impl FieldRun for Run {
    fn kind(&self) -> FieldKind {
        self.shared.kind()
    }

    fn before_cursor(&mut self, max_words: u8) -> Result<Option<Redacted<String>>, FieldError> {
        if !self.readable() {
            return Ok(None);
        }
        let text = self.ask(|reply, by| Job::Before {
            max_words,
            by,
            reply,
        })?;
        Ok(text.map(Redacted::new))
    }

    fn ends_with(&mut self, text: &Redacted<String>) -> Option<bool> {
        if !self.readable() {
            return None;
        }
        let text = text.clone();
        self.ask(|reply, by| Job::Ends { text, by, reply })
            .ok()
            .flatten()
    }
}

impl Drop for Run {
    fn drop(&mut self) {
        self.shared.stop();
    }
}

#[cfg(test)]
mod tests;
