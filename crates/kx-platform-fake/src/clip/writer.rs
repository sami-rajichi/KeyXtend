//! A fake clipboard writer: it records every write with its parts in order, and answers each with the result the test
//! queued, or success.

use crate::lock;
use kx_module_api::ports::{ClipError, ClipPart, ClipWait, ClipboardWriter};
use std::collections::VecDeque;
use std::fmt;
use std::sync::{Arc, Condvar, Mutex, PoisonError};
use std::time::Duration;

/// What the writes did, and what the test set for them.
#[derive(Default)]
struct Log {
    /// The results still to come, oldest first.
    queued: VecDeque<Result<(), ClipError>>,
    /// The parts of each write that set the clipboard, oldest first.
    writes: Vec<Vec<ClipPart>>,
    last_wait: Option<ClipWait>,
    tries: usize,
}

/// A clipboard writer shared by its clones: each write takes the next queued result, success once none is left, and
/// only a write that succeeds sets the clipboard.
#[derive(Clone, Default)]
pub struct FakeClipboardWriter {
    log: Arc<Mutex<Log>>,
    /// The signal each write gives, for a test waiting on another thread.
    tried: Arc<Condvar>,
}

impl FakeClipboardWriter {
    /// A writer with no write and nothing queued.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Queues `result` for a later write, after those already queued, such as `Err(ClipError::Busy)`.
    pub fn queue(&self, result: Result<(), ClipError>) {
        lock(&self.log).queued.push_back(result);
    }

    /// The parts of each write that set the clipboard, oldest first.
    #[must_use]
    pub fn writes(&self) -> Vec<Vec<ClipPart>> {
        lock(&self.log).writes.clone()
    }

    /// How many writes were asked, refused ones too.
    #[must_use]
    pub fn tries(&self) -> usize {
        lock(&self.log).tries
    }

    /// The wait the latest write took, refused or not.
    #[must_use]
    pub fn last_wait(&self) -> Option<ClipWait> {
        lock(&self.log).last_wait
    }

    /// Waits up to `within` until `count` writes were asked, so a test never sleeps; true when they were.
    #[must_use]
    pub fn wait_tries(&self, count: usize, within: Duration) -> bool {
        let waited = self
            .tried
            .wait_timeout_while(lock(&self.log), within, |l| l.tries < count);
        let (log, _) = waited.unwrap_or_else(PoisonError::into_inner);
        log.tries >= count
    }
}

/// Shows the counts, never the parts.
impl fmt::Debug for FakeClipboardWriter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let log = lock(&self.log);
        f.debug_struct("FakeClipboardWriter")
            .field("writes", &log.writes.len())
            .field("tries", &log.tries)
            .field("queued", &log.queued.len())
            .finish_non_exhaustive()
    }
}

impl ClipboardWriter for FakeClipboardWriter {
    fn write(&self, parts: &[ClipPart], wait: ClipWait) -> Result<(), ClipError> {
        let mut log = lock(&self.log);
        log.tries += 1;
        log.last_wait = Some(wait);
        self.tried.notify_all();
        let result = log.queued.pop_front().unwrap_or(Ok(()));
        if result.is_ok() {
            log.writes.push(parts.to_vec());
        }
        result
    }
}

#[cfg(test)]
mod tests;
