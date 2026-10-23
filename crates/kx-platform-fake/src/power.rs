//! A fake power dialog: it counts the opens, or refuses them with the error the test sets, and wakes a test waiting
//! for an open from another thread.

use crate::lock;
use kx_module_api::ports::{PowerDialog, PowerError};
use std::sync::{Arc, Condvar, Mutex, PoisonError};
use std::time::Duration;

/// What the opens did, and what the test set for them.
#[derive(Debug, Default)]
struct Log {
    opened: usize,
    error: Option<PowerError>,
}

/// The log, and the signal each counted open gives.
#[derive(Debug, Default)]
struct Shared {
    log: Mutex<Log>,
    opened: Condvar,
}

/// A power dialog shared by its clones: each open counts, unless an error is set, which refuses it uncounted.
#[derive(Clone, Debug, Default)]
pub struct FakePowerDialog {
    shared: Arc<Shared>,
}

impl FakePowerDialog {
    /// A dialog never opened, with no error set.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// How many opens showed the dialog.
    #[must_use]
    pub fn opened(&self) -> usize {
        lock(&self.shared.log).opened
    }

    /// Waits up to `within` until `count` opens showed the dialog, so a test never sleeps; true when they did.
    #[must_use]
    pub fn wait_opened(&self, count: usize, within: Duration) -> bool {
        let log = lock(&self.shared.log);
        let waited = self
            .shared
            .opened
            .wait_timeout_while(log, within, |l| l.opened < count);
        let (log, _) = waited.unwrap_or_else(PoisonError::into_inner);
        log.opened >= count
    }

    /// Refuses every later open with `error`, or takes them again with `None`.
    pub fn set_error(&self, error: Option<PowerError>) {
        lock(&self.shared.log).error = error;
    }
}

impl PowerDialog for FakePowerDialog {
    fn open(&self) -> Result<(), PowerError> {
        let mut log = lock(&self.shared.log);
        if let Some(error) = log.error {
            return Err(error);
        }
        log.opened += 1;
        self.shared.opened.notify_all();
        Ok(())
    }
}

#[cfg(test)]
mod tests;
