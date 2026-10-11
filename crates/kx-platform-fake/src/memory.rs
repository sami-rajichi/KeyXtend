//! A fake memory port: it counts each give-back, and refuses while the test sets an error.

use crate::lock;
use kx_platform::ui::{Memory, UiError};
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

/// A memory port that counts its calls, refused ones too.
#[derive(Debug, Default)]
pub struct FakeMemory {
    calls: AtomicUsize,
    error: Mutex<Option<UiError>>,
}

impl FakeMemory {
    /// A port with no call yet.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// How many give-backs were asked so far.
    #[must_use]
    pub fn calls(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }

    /// Refuses later give-backs with `error`, or takes them again with `None`.
    pub fn set_error(&self, error: Option<UiError>) {
        *lock(&self.error) = error;
    }
}

impl Memory for FakeMemory {
    fn give_back(&self) -> Result<(), UiError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        lock(&self.error).map_or(Ok(()), Err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_counts_each_give_back_and_refuses_while_told() {
        let memory = FakeMemory::new();
        assert_eq!(memory.calls(), 0);
        assert_eq!(memory.give_back(), Ok(()));
        memory.set_error(Some(UiError::Refused(5)));
        assert_eq!(memory.give_back(), Err(UiError::Refused(5)));
        memory.set_error(None);
        assert_eq!(memory.give_back(), Ok(()));
        assert_eq!(memory.calls(), 3, "refused calls count too");
    }
}
