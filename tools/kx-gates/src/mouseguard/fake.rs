//! A fake hook for tests: it applies its filter on a fake clock, as a hook thread with a timely check would, and
//! installs nothing. Tests that use this process's slot take `slot_lock` first.

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Duration;

use kx_module_api::{Clock, Mono};
use kx_platform_fake::FakeClock;

use super::run::{self, Hook, Live, Scope, Session};
use super::{Counts, End, Filter, Kind, LEFT, MouseGuard};

/// Serialises the tests that put a session in this process's slot.
static SLOT_TESTS: Mutex<()> = Mutex::new(());

/// What the fake hooks saw: starts, stops, the last filter, and whether the next start fails.
#[derive(Default)]
pub struct Seen {
    /// Hooks started.
    pub starts: u32,
    /// Hooks stopped.
    pub stops: u32,
    /// The last started hook's filter.
    pub filter: Option<Filter>,
    /// The next start fails.
    pub fail: bool,
}

/// A hook that lets go exactly when its filter says.
#[derive(Clone)]
pub struct Fake {
    /// What its hooks saw, shared by its clones.
    pub seen: Arc<Mutex<Seen>>,
    /// The time its filters read.
    pub clock: FakeClock,
}

impl Fake {
    /// A hook that saw nothing yet, at time zero.
    pub fn new() -> Self {
        Self {
            seen: Arc::default(),
            clock: FakeClock::new(Mono::from_us(0)),
        }
    }

    /// What its hooks saw; one lock per call.
    pub fn seen(&self) -> MutexGuard<'_, Seen> {
        self.seen.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Hooks started and stopped so far, read under one lock.
    pub fn counts(&self) -> (u32, u32) {
        let seen = self.seen();
        (seen.starts, seen.stops)
    }

    /// A physical press of the left button now, as the hook would see it.
    pub fn press(&self) {
        let now = self.clock.now();
        let mut seen = self.seen();
        let filter = seen.filter.as_mut().expect("a started hook");
        filter.event(false, Kind::Down(LEFT), now);
    }

    /// A move kx-gates injected now, as the hook would see it.
    pub fn mine(&self) {
        let now = self.clock.now();
        let mut seen = self.seen();
        let filter = seen.filter.as_mut().expect("a started hook");
        filter.event(true, Kind::Other, now);
    }

    /// Moves the fake clock on by `ms`.
    pub fn wait(&self, ms: u64) {
        self.clock.advance(Duration::from_millis(ms));
    }
}

impl Hook for Fake {
    type Live = Fake;

    fn start(&self, filter: Filter) -> Result<Fake, String> {
        let mut seen = self.seen();
        if seen.fail {
            return Err("no hook today".to_string());
        }
        seen.starts += 1;
        seen.filter = Some(filter);
        Ok(self.clone())
    }
}

impl Live for Fake {
    fn ended(&self) -> Option<End> {
        let now = self.clock.now();
        self.seen().filter.as_mut()?.tick(now)
    }

    fn stop(self) -> (Option<End>, Counts) {
        let own = self.ended();
        let mut seen = self.seen();
        seen.stops += 1;
        (
            own,
            seen.filter.as_ref().map(Filter::counts).unwrap_or_default(),
        )
    }
}

/// The shipped guard settings.
pub fn cfg() -> MouseGuard {
    crate::config::load()
        .expect("kx-gates.toml loads")
        .mouse_guard
}

/// A session over `fake`.
pub fn begun(fake: &Fake) -> Session<Fake, FakeClock> {
    Session::begin(fake.clone(), fake.clock.clone(), &cfg()).expect("the fake hook starts")
}

/// The lock for tests that use this process's slot; a failed test does not block the others.
pub fn slot_lock() -> MutexGuard<'static, ()> {
    SLOT_TESTS.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Holds a session over `fake` in this process's slot until the scope ends; take `slot_lock` first.
pub fn held(fake: &Fake) -> Scope {
    run::hold(Box::new(begun(fake)))
}
