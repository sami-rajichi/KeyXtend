//! A fake system look: the test sets the look and the display language, and each set calls the watchers.

use crate::lock;
use kx_platform::ui::{SysLook, SystemLook, UiError, WatchGuard};
use std::collections::BTreeMap;
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

/// The display language a new `FakeSystemLook` reports.
pub const DEFAULT_LANGUAGE: &str = "en-US";

/// A watcher of the look.
type Watcher = Box<dyn Fn(SysLook) + Send>;

/// A system look the test sets; each `set` calls every watcher whose guard lives.
pub struct FakeSystemLook {
    look: Mutex<SysLook>,
    language: Mutex<String>,
    watchers: Arc<Mutex<BTreeMap<u64, Watcher>>>,
    next: AtomicU64,
    watch_error: Mutex<Option<UiError>>,
}

impl FakeSystemLook {
    /// The default look, in `DEFAULT_LANGUAGE`, with no watchers.
    #[must_use]
    pub fn new() -> Self {
        Self {
            look: Mutex::default(),
            language: Mutex::new(DEFAULT_LANGUAGE.to_owned()),
            watchers: Arc::default(),
            next: AtomicU64::default(),
            watch_error: Mutex::default(),
        }
    }

    /// Sets the look and calls every watcher with it; a watcher must not call `set` or drop its guard.
    pub fn set(&self, look: SysLook) {
        *lock(&self.look) = look;
        for watcher in lock(&self.watchers).values() {
            watcher(look);
        }
    }

    /// Refuses later `watch` calls with `error`, or takes them again with `None`.
    pub fn set_watch_error(&self, error: Option<UiError>) {
        *lock(&self.watch_error) = error;
    }

    /// Sets the display language tag.
    pub fn set_language(&self, tag: &str) {
        tag.clone_into(&mut lock(&self.language));
    }
}

impl Default for FakeSystemLook {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for FakeSystemLook {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FakeSystemLook")
            .field("look", &*lock(&self.look))
            .field("language", &*lock(&self.language))
            .finish_non_exhaustive()
    }
}

impl SystemLook for FakeSystemLook {
    fn read(&self) -> SysLook {
        *lock(&self.look)
    }

    fn watch(&self, on_change: Box<dyn Fn(SysLook) + Send>) -> Result<WatchGuard, UiError> {
        if let Some(error) = *lock(&self.watch_error) {
            return Err(error);
        }
        let id = self.next.fetch_add(1, Ordering::SeqCst);
        lock(&self.watchers).insert(id, on_change);
        let watchers = Arc::downgrade(&self.watchers);
        Ok(WatchGuard::new(move || {
            if let Some(watchers) = watchers.upgrade() {
                lock(&watchers).remove(&id);
            }
        }))
    }

    fn ui_language(&self) -> String {
        lock(&self.language).clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dark() -> SysLook {
        SysLook {
            dark: true,
            ..SysLook::default()
        }
    }

    #[test]
    fn set_calls_the_watcher_until_its_guard_drops() {
        let fake = FakeSystemLook::new();
        let seen = Arc::new(Mutex::new(Vec::new()));
        let keep = Arc::clone(&seen);
        let guard = fake.watch(Box::new(move |look| keep.lock().unwrap().push(look)));
        let guard = guard.unwrap();
        fake.set(dark());
        assert_eq!(fake.read(), dark());
        assert_eq!(*seen.lock().unwrap(), [dark()]);
        drop(guard);
        fake.set(SysLook::default());
        assert_eq!(seen.lock().unwrap().len(), 1, "no call after the drop");
    }

    #[test]
    fn two_watchers_each_hear_every_set() {
        let fake = FakeSystemLook::new();
        let count = Arc::new(AtomicU64::new(0));
        let hear = |count: &Arc<AtomicU64>| {
            let count = Arc::clone(count);
            Box::new(move |_: SysLook| {
                count.fetch_add(1, Ordering::SeqCst);
            })
        };
        let _first = fake.watch(hear(&count)).unwrap();
        let _second = fake.watch(hear(&count)).unwrap();
        fake.set(dark());
        assert_eq!(count.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn a_refused_watch_hears_nothing() {
        let fake = FakeSystemLook::new();
        fake.set_watch_error(Some(UiError::Refused(7)));
        let count = Arc::new(AtomicU64::new(0));
        let keep = Arc::clone(&count);
        let watch = fake.watch(Box::new(move |_| {
            keep.fetch_add(1, Ordering::SeqCst);
        }));
        assert!(matches!(watch, Err(UiError::Refused(7))));
        fake.set(dark());
        assert_eq!(count.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn a_new_fake_shows_the_usual_pointer_size() {
        assert_eq!(FakeSystemLook::new().read().pointer_px, SysLook::POINTER_PX);
    }

    #[test]
    fn the_ui_language_is_the_one_set() {
        let fake = FakeSystemLook::new();
        assert_eq!(fake.ui_language(), DEFAULT_LANGUAGE);
        fake.set_language("ar-TN");
        assert_eq!(fake.ui_language(), "ar-TN");
    }
}
