//! Test helpers both stores' tests share.

use crate::guard::{GuardError, KeyGuard};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering::SeqCst};
use zeroize::Zeroizing;

/// A test guard that wraps by reversing the bytes, refuses on cue and counts its calls.
#[derive(Clone, Default)]
pub(crate) struct Mirror {
    pub(crate) refuse: Arc<AtomicBool>,
    calls: Arc<AtomicUsize>,
}

impl Mirror {
    pub(crate) fn refusing() -> Self {
        let guard = Self::default();
        guard.refuse.store(true, SeqCst);
        guard
    }

    pub(crate) fn calls(&self) -> usize {
        self.calls.load(SeqCst)
    }

    fn answer(&self, error: GuardError, bytes: &[u8]) -> Result<Vec<u8>, GuardError> {
        self.calls.fetch_add(1, SeqCst);
        if self.refuse.load(SeqCst) {
            return Err(error);
        }
        Ok(bytes.iter().rev().copied().collect())
    }
}

impl KeyGuard for Mirror {
    fn wrap(&self, key: &[u8]) -> Result<Vec<u8>, GuardError> {
        self.answer(GuardError::Unavailable, key)
    }

    fn unwrap(&self, wrapped: &[u8]) -> Result<Zeroizing<Vec<u8>>, GuardError> {
        self.answer(GuardError::Refused, wrapped)
            .map(Zeroizing::new)
    }
}

/// The file names in `dir`, sorted.
pub(crate) fn listing(dir: &std::path::Path) -> Vec<String> {
    let mut names: Vec<_> = std::fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}
