//! A fake session watch: the test tells each lock, unlock, sleep and wake on cue, on its own thread, and the fake
//! counts the watches.

use crate::lock;
use kx_module_api::ports::{SessionError, SessionNote, SessionRun, SessionWatch};
use std::fmt;
use std::sync::{Arc, Condvar, Mutex, PoisonError};
use std::time::Duration;

/// The callback a watch tells on each note.
type Tell = Box<dyn Fn(SessionNote) + Send>;

/// What the watches did, and what the test set for them.
#[derive(Debug, Default)]
struct Log {
    /// The live watch's number, which holds `tell`; 0 for none.
    live: usize,
    watches: usize,
    stops: usize,
    fail: Option<SessionError>,
}

/// The log, the live watch's callback, kept apart so a note tells with the log unlocked, and the signal each watch
/// gives. Whoever takes both locks takes `tell` first.
#[derive(Default)]
struct Shared {
    log: Mutex<Log>,
    tell: Mutex<Option<Tell>>,
    watched: Condvar,
}

/// A session watch shared by its clones: a second watch takes the notes from the first, and a `tell` runs under the
/// fake's lock, so it must not call `tell`, start a watch or drop its run.
#[derive(Clone, Default)]
pub struct FakeSessionWatch {
    shared: Arc<Shared>,
}

impl FakeSessionWatch {
    /// A watch never started, with nothing scripted.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Tells the live watch `note` at once on this thread; false when no watch runs.
    #[must_use]
    pub fn tell(&self, note: SessionNote) -> bool {
        let tell = lock(&self.shared.tell);
        tell.as_ref().map(|tell| tell(note)).is_some()
    }

    /// Refuses the next watch with `error`, once.
    pub fn fail_watch(&self, error: SessionError) {
        lock(&self.shared.log).fail = Some(error);
    }

    /// Whether a watch is held, from its start until the module drops it.
    #[must_use]
    pub fn watching(&self) -> bool {
        let log = lock(&self.shared.log);
        log.watches > log.stops
    }

    /// How many watches started.
    #[must_use]
    pub fn watches(&self) -> usize {
        lock(&self.shared.log).watches
    }

    /// Waits up to `within` until a watch is held, so a test never sleeps; true when one is.
    #[must_use]
    pub fn wait_watching(&self, within: Duration) -> bool {
        let log = lock(&self.shared.log);
        let held = |l: &mut Log| l.watches <= l.stops;
        let waited = self.shared.watched.wait_timeout_while(log, within, held);
        let (log, _) = waited.unwrap_or_else(PoisonError::into_inner);
        log.watches > log.stops
    }
}

/// Shows the counts.
impl fmt::Debug for FakeSessionWatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let log = lock(&self.shared.log);
        f.debug_struct("FakeSessionWatch")
            .field("watches", &log.watches)
            .field("stops", &log.stops)
            .finish_non_exhaustive()
    }
}

impl SessionWatch for FakeSessionWatch {
    fn watch(&self, tell: Tell) -> Result<Box<dyn SessionRun>, SessionError> {
        let mut slot = lock(&self.shared.tell);
        let mut log = lock(&self.shared.log);
        if let Some(error) = log.fail.take() {
            return Err(error);
        }
        log.watches += 1;
        let id = log.watches;
        log.live = id;
        *slot = Some(tell);
        self.shared.watched.notify_all();
        let shared = Arc::clone(&self.shared);
        Ok(Box::new(FakeSessionRun { shared, id }))
    }
}

/// A live fake watch; dropping it stops it, and its tells when it is the live one.
struct FakeSessionRun {
    shared: Arc<Shared>,
    id: usize,
}

impl SessionRun for FakeSessionRun {}

impl Drop for FakeSessionRun {
    /// Waits out a note being told on another thread, so none comes after.
    fn drop(&mut self) {
        let mut slot = lock(&self.shared.tell);
        let mut log = lock(&self.shared.log);
        log.stops += 1;
        let live = log.live == self.id;
        if live {
            log.live = 0;
        }
        let gone = live.then(|| slot.take()).flatten();
        drop((log, slot));
        drop(gone);
    }
}

#[cfg(test)]
mod tests;
