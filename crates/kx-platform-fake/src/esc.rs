//! A fake Esc watch: the test presses Esc, which reaches the watcher whose guard lives, and sees whether one listens.

use crate::lock;
use kx_platform::ui::{EscWatch, OnEsc, UiError, WatchGuard};
use std::fmt;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

/// The watcher while its guard lives, with the number of the watch it came from.
type Slot = Arc<Mutex<Option<(usize, Arc<dyn Fn() + Send + Sync>)>>>;

/// An Esc watch the test drives: one watcher at a time, as the hub keeps one.
#[derive(Default)]
pub struct FakeEscWatch {
    slot: Slot,
    /// Watches started so far.
    starts: AtomicUsize,
    error: Mutex<Option<UiError>>,
}

impl FakeEscWatch {
    /// A watch nobody listens to yet.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// True while a watcher's guard lives.
    #[must_use]
    pub fn watching(&self) -> bool {
        lock(&self.slot).is_some()
    }

    /// How many watches started, refused ones left out.
    #[must_use]
    pub fn starts(&self) -> usize {
        self.starts.load(Ordering::SeqCst)
    }

    /// The hand presses Esc: the live watcher hears it, outside the lock; false when none listens.
    pub fn press(&self) -> bool {
        let on = lock(&self.slot).as_ref().map(|(_, on)| Arc::clone(on));
        on.map(|on| on()).is_some()
    }

    /// Refuses later `watch` calls with `error`, or takes them again with `None`.
    pub fn set_error(&self, error: Option<UiError>) {
        *lock(&self.error) = error;
    }
}

impl fmt::Debug for FakeEscWatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FakeEscWatch")
            .field("watching", &self.watching())
            .field("starts", &self.starts())
            .finish_non_exhaustive()
    }
}

impl EscWatch for FakeEscWatch {
    /// A new watch replaces the last one's watcher; dropping an old guard then changes nothing.
    fn watch(&self, on: OnEsc) -> Result<WatchGuard, UiError> {
        if let Some(error) = *lock(&self.error) {
            return Err(error);
        }
        let n = self.starts.fetch_add(1, Ordering::SeqCst);
        *lock(&self.slot) = Some((n, Arc::from(on)));
        let slot = Arc::clone(&self.slot);
        Ok(WatchGuard::new(move || {
            let mut held = lock(&slot);
            if held.as_ref().is_some_and(|(m, _)| *m == n) {
                *held = None;
            }
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_press_reaches_the_watcher_until_its_guard_drops() {
        let fake = FakeEscWatch::new();
        assert!(!fake.press(), "nobody listens");
        let heard = Arc::new(AtomicUsize::new(0));
        let count = Arc::clone(&heard);
        let guard = fake.watch(Box::new(move || {
            count.fetch_add(1, Ordering::SeqCst);
        }));
        assert!(fake.watching() && fake.press());
        assert_eq!(heard.load(Ordering::SeqCst), 1);
        drop(guard);
        assert!(!fake.watching() && !fake.press());
        assert_eq!((heard.load(Ordering::SeqCst), fake.starts()), (1, 1));
    }

    #[test]
    fn a_refused_watch_starts_nothing() {
        let fake = FakeEscWatch::new();
        fake.set_error(Some(UiError::Refused(5)));
        assert_eq!(fake.watch(Box::new(|| {})).err(), Some(UiError::Refused(5)));
        assert_eq!((fake.watching(), fake.starts()), (false, 0));
    }
}
