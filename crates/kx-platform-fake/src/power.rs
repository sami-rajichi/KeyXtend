//! A fake power dialog: it counts the opens, or refuses them with the error the test sets.

use crate::lock;
use kx_module_api::ports::{PowerDialog, PowerError};
use std::sync::{Arc, Mutex};

/// What the opens did, and what the test set for them.
#[derive(Debug, Default)]
struct Log {
    opened: usize,
    error: Option<PowerError>,
}

/// A power dialog shared by its clones: each open counts, unless an error is set, which refuses it uncounted.
#[derive(Clone, Debug, Default)]
pub struct FakePowerDialog {
    log: Arc<Mutex<Log>>,
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
        lock(&self.log).opened
    }

    /// Refuses every later open with `error`, or takes them again with `None`.
    pub fn set_error(&self, error: Option<PowerError>) {
        lock(&self.log).error = error;
    }
}

impl PowerDialog for FakePowerDialog {
    fn open(&self) -> Result<(), PowerError> {
        let mut log = lock(&self.log);
        if let Some(error) = log.error {
            return Err(error);
        }
        log.opened += 1;
        Ok(())
    }
}

#[cfg(test)]
mod tests;
