//! A fake reveal: it records each folder it was asked to show, refuses while the test sets an error, and lets a test
//! wait for a call made on another thread without sleeping.

use crate::lock;
use kx_platform::ui::{Reveal, RevealError};
use std::path::{Path, PathBuf};
use std::sync::{Condvar, Mutex, PoisonError};
use std::time::Duration;

/// A reveal that keeps every folder asked, refused ones too.
#[derive(Debug, Default)]
pub struct FakeReveal {
    shown: Mutex<Vec<PathBuf>>,
    asked: Condvar,
    error: Mutex<Option<RevealError>>,
}

impl FakeReveal {
    /// A reveal with no call yet.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Every folder asked so far, oldest first.
    #[must_use]
    pub fn shown(&self) -> Vec<PathBuf> {
        lock(&self.shown).clone()
    }

    /// Waits up to `within` until `n` folders were asked, so a test never sleeps; true when they were.
    #[must_use]
    pub fn wait_shown(&self, n: usize, within: Duration) -> bool {
        let shown = lock(&self.shown);
        let waited = self
            .asked
            .wait_timeout_while(shown, within, |s| s.len() < n);
        let (shown, _) = waited.unwrap_or_else(PoisonError::into_inner);
        shown.len() >= n
    }

    /// Refuses later calls with `error`, or takes them again with `None`.
    pub fn set_error(&self, error: Option<RevealError>) {
        *lock(&self.error) = error;
    }
}

impl Reveal for FakeReveal {
    fn show(&self, folder: &Path) -> Result<(), RevealError> {
        let refused = *lock(&self.error);
        lock(&self.shown).push(folder.to_path_buf());
        self.asked.notify_all();
        refused.map_or(Ok(()), Err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    /// How long a test waits for another thread's call.
    const WAIT: Duration = Duration::from_secs(5);

    #[test]
    fn it_records_each_folder_and_refuses_while_told() {
        let reveal = FakeReveal::new();
        assert_eq!(reveal.show(Path::new("a")), Ok(()));
        reveal.set_error(Some(RevealError::Missing));
        assert_eq!(reveal.show(Path::new("b")), Err(RevealError::Missing));
        reveal.set_error(None);
        assert_eq!(reveal.show(Path::new("c")), Ok(()));
        let want: Vec<PathBuf> = ["a", "b", "c"].map(PathBuf::from).into();
        assert_eq!(reveal.shown(), want, "refused calls count too");
    }

    #[test]
    fn a_test_waits_for_a_call_from_another_thread() {
        let reveal = Arc::new(FakeReveal::new());
        assert!(!reveal.wait_shown(1, Duration::ZERO), "nothing asked yet");
        let there = Arc::clone(&reveal);
        let call = std::thread::spawn(move || there.show(Path::new("d")));
        assert!(reveal.wait_shown(1, WAIT));
        assert_eq!(call.join().ok(), Some(Ok(())));
    }
}
