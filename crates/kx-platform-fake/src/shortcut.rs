//! A fake shortcuts service: the test sets the ids it knows, and the fake records each send with its held keys.

use crate::lock;
use kx_module_api::ports::{Scan, ShortcutError, ShortcutId, ShortcutInfo, Shortcuts};
use std::fmt;
use std::sync::{Arc, Mutex};

/// What the sends did, and what the test set for them.
#[derive(Default)]
struct Log {
    known: Vec<ShortcutInfo>,
    sent: Vec<(ShortcutId, Vec<Scan>)>,
    error: Option<ShortcutError>,
}

/// Shortcuts the test scripts, shared by its clones: an id not in `known` is `Unknown` and records nothing, and while
/// an error is set every other send fails with it and records nothing.
#[derive(Clone, Default)]
pub struct FakeShortcuts {
    log: Arc<Mutex<Log>>,
}

impl FakeShortcuts {
    /// A fake that knows no id and records nothing yet.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the ids `known` lists, with their steps.
    pub fn set_known(&self, known: Vec<ShortcutInfo>) {
        lock(&self.log).known = known;
    }

    /// Every send taken so far, with its held keys, oldest first.
    #[must_use]
    pub fn sent(&self) -> Vec<(ShortcutId, Vec<Scan>)> {
        lock(&self.log).sent.clone()
    }

    /// Forgets the sends so far.
    pub fn clear(&self) {
        lock(&self.log).sent.clear();
    }

    /// Refuses every later send of a known id with `error`, or takes them again with `None`.
    pub fn set_error(&self, error: Option<ShortcutError>) {
        lock(&self.log).error = error;
    }
}

/// Shows only how many ids are known and how many sends were taken, as ids and keys stay out of logs.
impl fmt::Debug for FakeShortcuts {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let log = lock(&self.log);
        f.debug_struct("FakeShortcuts")
            .field("known", &log.known.len())
            .field("sent", &log.sent.len())
            .finish_non_exhaustive()
    }
}

impl Shortcuts for FakeShortcuts {
    fn known(&self) -> Vec<ShortcutInfo> {
        lock(&self.log).known.clone()
    }

    fn send(&self, id: &ShortcutId, held: &[Scan]) -> Result<(), ShortcutError> {
        let mut log = lock(&self.log);
        if !log.known.iter().any(|k| k.id == *id) {
            return Err(ShortcutError::Unknown);
        }
        if let Some(error) = log.error {
            return Err(error);
        }
        log.sent.push((id.clone(), held.to_vec()));
        Ok(())
    }
}

#[cfg(test)]
mod tests;
