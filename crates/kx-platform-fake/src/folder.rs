//! A fake folder reveal: it records each folder it was asked to show, refuses while the test sets an error, and lets a
//! test wait for a call made on another thread without sleeping. Its `Debug` prints counts, never a folder.

use crate::lock;
use kx_module_api::ports::{FolderError, FolderPath, FolderReveal};
use std::fmt;
use std::sync::{Condvar, Mutex, PoisonError};
use std::time::Duration;

/// What the fake was asked and how it answers.
#[derive(Default)]
struct State {
    /// Every folder asked, oldest first.
    shown: Vec<FolderPath>,
    /// The error later calls give.
    error: Option<FolderError>,
}

/// A folder reveal that keeps every folder asked, refused ones too.
#[derive(Default)]
pub struct FakeFolderReveal {
    state: Mutex<State>,
    changed: Condvar,
}

impl FakeFolderReveal {
    /// A reveal with no call yet.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Every folder asked so far, oldest first.
    #[must_use]
    pub fn shown(&self) -> Vec<FolderPath> {
        lock(&self.state).shown.clone()
    }

    /// Waits up to `within` until `n` folders were asked, so a test never sleeps; true when they were.
    #[must_use]
    pub fn wait_shown(&self, n: usize, within: Duration) -> bool {
        let waited =
            (self.changed).wait_timeout_while(lock(&self.state), within, |s| s.shown.len() < n);
        let (state, _) = waited.unwrap_or_else(PoisonError::into_inner);
        state.shown.len() >= n
    }

    /// Refuses later calls with `error`, or takes them again with `None`.
    pub fn set_error(&self, error: Option<FolderError>) {
        lock(&self.state).error = error;
    }
}

/// Shows the counts, never a folder.
impl fmt::Debug for FakeFolderReveal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let state = lock(&self.state);
        f.debug_struct("FakeFolderReveal")
            .field("shown", &state.shown.len())
            .field("error", &state.error)
            .finish_non_exhaustive()
    }
}

impl FolderReveal for FakeFolderReveal {
    fn show(&self, folder: &FolderPath) -> Result<(), FolderError> {
        let mut state = lock(&self.state);
        state.shown.push(folder.clone());
        self.changed.notify_all();
        state.error.map_or(Ok(()), Err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kx_test_support::paths::native_path;
    use std::sync::Arc;

    /// How long a test waits for another thread's call.
    const WAIT: Duration = Duration::from_secs(5);

    /// A checked folder under the test models folder.
    fn folder(name: &str) -> FolderPath {
        FolderPath::new(native_path(&format!(r"D:\KeyXtend\{name}"))).unwrap()
    }

    #[test]
    fn it_records_each_folder_and_refuses_while_told() {
        let reveal = FakeFolderReveal::new();
        assert_eq!(reveal.show(&folder("a")), Ok(()));
        reveal.set_error(Some(FolderError::Linked));
        assert_eq!(reveal.show(&folder("b")), Err(FolderError::Linked));
        reveal.set_error(None);
        assert_eq!(reveal.show(&folder("c")), Ok(()));
        let want = ["a", "b", "c"].map(folder);
        assert_eq!(reveal.shown(), want, "refused calls count too");
    }

    #[test]
    fn a_test_waits_for_a_call_from_another_thread() {
        let reveal = Arc::new(FakeFolderReveal::new());
        assert!(!reveal.wait_shown(1, Duration::ZERO), "nothing asked yet");
        let there = Arc::clone(&reveal);
        let call = std::thread::spawn(move || there.show(&folder("d")));
        assert!(reveal.wait_shown(1, WAIT));
        assert_eq!(call.join().ok(), Some(Ok(())));
    }

    #[test]
    fn debug_never_prints_a_folder() {
        let reveal = FakeFolderReveal::new();
        assert_eq!(reveal.show(&folder("models")), Ok(()));
        let text = format!("{reveal:?}");
        assert!(text.contains("shown: 1"), "{text}");
        assert!(!text.contains("KeyXtend"), "{text}");
    }
}
