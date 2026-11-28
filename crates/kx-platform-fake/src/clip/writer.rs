//! A fake clipboard writer: it records every write with its parts and its kind in order, and answers each with the
//! result the test queued, or success.

use crate::lock;
use kx_module_api::ports::{ClipError, ClipPart, ClipWait, ClipboardWriter};
use std::collections::VecDeque;
use std::fmt;
use std::sync::{Arc, Condvar, Mutex, PoisonError};
use std::time::Duration;

/// Which of the writer's calls set the clipboard, so a test tells their markers apart.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum WriteKind {
    /// `write`: with our "leave out of history" marker.
    Marked,
    /// `write_lent`: with that marker and the private ones.
    Lent,
    /// `write_kept`: with none of our markers, so both histories keep it.
    Kept,
}

/// What the writes did, and what the test set for them.
#[derive(Default)]
struct Log {
    /// The results still to come, oldest first.
    queued: VecDeque<Result<(), ClipError>>,
    /// The parts of each write that set the clipboard, oldest first.
    writes: Vec<Vec<ClipPart>>,
    /// The kind of each of those writes.
    kinds: Vec<WriteKind>,
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

    /// Whether each write that set the clipboard was lent, oldest first.
    #[must_use]
    pub fn lent(&self) -> Vec<bool> {
        let kinds = lock(&self.log).kinds.clone();
        kinds.into_iter().map(|k| k == WriteKind::Lent).collect()
    }

    /// The kind of each write that set the clipboard, oldest first.
    #[must_use]
    pub fn kinds(&self) -> Vec<WriteKind> {
        lock(&self.log).kinds.clone()
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

impl FakeClipboardWriter {
    /// Takes the next queued result for a write of `parts` of `kind`, and records it when it succeeds.
    fn record(&self, parts: &[ClipPart], wait: ClipWait, kind: WriteKind) -> Result<(), ClipError> {
        let mut log = lock(&self.log);
        log.tries += 1;
        log.last_wait = Some(wait);
        self.tried.notify_all();
        let result = log.queued.pop_front().unwrap_or(Ok(()));
        if result.is_ok() {
            log.writes.push(parts.to_vec());
            log.kinds.push(kind);
        }
        result
    }
}

impl ClipboardWriter for FakeClipboardWriter {
    fn write(&self, parts: &[ClipPart], wait: ClipWait) -> Result<(), ClipError> {
        self.record(parts, wait, WriteKind::Marked)
    }

    fn write_lent(&self, parts: &[ClipPart], wait: ClipWait) -> Result<(), ClipError> {
        self.record(parts, wait, WriteKind::Lent)
    }

    fn write_kept(&self, parts: &[ClipPart], wait: ClipWait) -> Result<(), ClipError> {
        self.record(parts, wait, WriteKind::Kept)
    }
}

#[cfg(test)]
mod tests;
