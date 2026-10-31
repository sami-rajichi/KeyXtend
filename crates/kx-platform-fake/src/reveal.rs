//! A fake reveal: it records each folder it was asked to show, refuses while the test sets an error, holds its calls
//! while told, and lets a test wait for a call made on another thread without sleeping.

use crate::lock;
use kx_platform::ui::{Reveal, RevealError};
use std::path::{Path, PathBuf};
use std::sync::{Condvar, Mutex, MutexGuard, PoisonError};
use std::time::Duration;

/// What the fake was asked and how it answers.
#[derive(Debug, Default)]
struct State {
    /// Every folder asked, oldest first.
    shown: Vec<PathBuf>,
    /// The calls that returned.
    left: usize,
    /// The error later calls give.
    error: Option<RevealError>,
    /// Calls wait inside `show` while this holds.
    held: bool,
}

/// A reveal that keeps every folder asked, refused ones too.
#[derive(Debug, Default)]
pub struct FakeReveal {
    state: Mutex<State>,
    changed: Condvar,
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
        lock(&self.state).shown.clone()
    }

    /// Waits up to `within` until `n` folders were asked, so a test never sleeps; true when they were.
    #[must_use]
    pub fn wait_shown(&self, n: usize, within: Duration) -> bool {
        self.wait(within, |s| s.shown.len() >= n)
    }

    /// Waits up to `within` until `n` calls returned; true when they did.
    #[must_use]
    pub fn wait_left(&self, n: usize, within: Duration) -> bool {
        self.wait(within, |s| s.left >= n)
    }

    /// Refuses later calls with `error`, or takes them again with `None`.
    pub fn set_error(&self, error: Option<RevealError>) {
        lock(&self.state).error = error;
    }

    /// Holds calls inside `show` after they are recorded, as a slow file manager would, until called with false.
    pub fn hold(&self, on: bool) {
        lock(&self.state).held = on;
        self.changed.notify_all();
    }

    /// Waits up to `within` until `done` holds for the state.
    fn wait(&self, within: Duration, done: impl Fn(&State) -> bool) -> bool {
        let waited = self
            .changed
            .wait_timeout_while(lock(&self.state), within, |s| !done(s));
        let (state, _) = waited.unwrap_or_else(PoisonError::into_inner);
        done(&state)
    }

    /// Waits while calls are held.
    fn wait_released<'a>(&self, state: MutexGuard<'a, State>) -> MutexGuard<'a, State> {
        let released = self.changed.wait_while(state, |s| s.held);
        released.unwrap_or_else(PoisonError::into_inner)
    }
}

impl Reveal for FakeReveal {
    fn show(&self, folder: &Path) -> Result<(), RevealError> {
        let mut state = lock(&self.state);
        state.shown.push(folder.to_path_buf());
        self.changed.notify_all();
        let mut state = self.wait_released(state);
        state.left += 1;
        self.changed.notify_all();
        state.error.map_or(Ok(()), Err)
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

    #[test]
    fn a_held_call_is_recorded_and_returns_only_when_released() {
        let reveal = Arc::new(FakeReveal::new());
        reveal.hold(true);
        let there = Arc::clone(&reveal);
        let call = std::thread::spawn(move || there.show(Path::new("e")));
        assert!(reveal.wait_shown(1, WAIT));
        assert!(!reveal.wait_left(1, Duration::ZERO), "still held");
        reveal.hold(false);
        assert!(reveal.wait_left(1, WAIT));
        assert_eq!(call.join().ok(), Some(Ok(())));
    }
}
