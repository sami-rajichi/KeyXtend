//! Fakes of the UI ports: screens, the window guard and backdrops.

use crate::lock;
use kx_module_api::ports::{Point, Rect, Screen};
use kx_platform::ui::{Backdrop, BackdropKind, FocusToken, Screens, UiError, WindowGuard};
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

/// Screens with work areas and a pointer the test sets.
#[derive(Debug, Default)]
pub struct FakeScreens {
    areas: Mutex<Vec<Screen>>,
    cursor: Mutex<Point>,
    cursor_error: Mutex<Option<UiError>>,
}

impl FakeScreens {
    /// Screens with `areas` and the pointer at `cursor`.
    #[must_use]
    pub fn new(areas: Vec<Screen>, cursor: Point) -> Self {
        Self {
            areas: Mutex::new(areas),
            cursor: Mutex::new(cursor),
            cursor_error: Mutex::default(),
        }
    }

    /// Replaces the screens, as when one is plugged in or out.
    pub fn set_areas(&self, areas: Vec<Screen>) {
        *lock(&self.areas) = areas;
    }

    /// Moves the pointer.
    pub fn set_cursor(&self, at: Point) {
        *lock(&self.cursor) = at;
    }

    /// Makes `cursor` fail with `error`, or answer again with `None`.
    pub fn set_cursor_error(&self, error: Option<UiError>) {
        *lock(&self.cursor_error) = error;
    }
}

impl Screens for FakeScreens {
    fn work_areas(&self) -> Vec<Screen> {
        lock(&self.areas).clone()
    }

    fn cursor(&self) -> Result<Point, UiError> {
        match *lock(&self.cursor_error) {
            Some(error) => Err(error),
            None => Ok(*lock(&self.cursor)),
        }
    }
}

/// One call the fake window guard took.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GuardCall {
    /// `guard_own(spare, top)`.
    GuardOwn(Option<String>, Option<String>),
    /// `shape(title, rects)`.
    Shape(String, Vec<Rect>),
    /// `lift_alone(title)`.
    LiftAlone(String),
    /// `watch_entries(title)`.
    WatchEntries(String),
    /// `unwatch_entries(title)`.
    UnwatchEntries(String),
    /// `lay_under(title, under)`.
    LayUnder(String, String),
    /// `remember`, with the token it gave.
    Remember(FocusToken),
    /// `give_back(token)`.
    GiveBack(FocusToken),
}

/// A window guard that records every call, refused or not; `guard_own` reports the count the test chose.
#[derive(Debug, Default)]
pub struct FakeWindowGuard {
    count: AtomicUsize,
    error: Mutex<Option<UiError>>,
    calls: Mutex<Vec<GuardCall>>,
    last: AtomicU64,
}

impl FakeWindowGuard {
    /// A guard whose `guard_own` says it guards `count` windows.
    #[must_use]
    pub fn new(count: usize) -> Self {
        Self {
            count: AtomicUsize::new(count),
            ..Self::default()
        }
    }

    /// Sets how many windows later `guard_own` calls report.
    pub fn set_count(&self, count: usize) {
        self.count.store(count, Ordering::SeqCst);
    }

    /// Refuses later `guard_own`, `shape`, `lift_alone`, `watch_entries` and `lay_under` calls with `error`, or takes them
    /// again with `None`.
    pub fn set_error(&self, error: Option<UiError>) {
        *lock(&self.error) = error;
    }

    /// Every call so far, oldest first.
    #[must_use]
    pub fn calls(&self) -> Vec<GuardCall> {
        lock(&self.calls).clone()
    }

    /// Records `call`, then answers with the error the test set, if any.
    fn record(&self, call: GuardCall) -> Result<(), UiError> {
        lock(&self.calls).push(call);
        lock(&self.error).map_or(Ok(()), Err)
    }
}

impl WindowGuard for FakeWindowGuard {
    fn guard_own(&self, spare: Option<&str>, top: Option<&str>) -> Result<usize, UiError> {
        let call = GuardCall::GuardOwn(spare.map(str::to_owned), top.map(str::to_owned));
        self.record(call)?;
        Ok(self.count.load(Ordering::SeqCst))
    }

    fn shape(&self, title: &str, rects: &[Rect]) -> Result<(), UiError> {
        self.record(GuardCall::Shape(title.to_owned(), rects.to_vec()))
    }

    fn lift_alone(&self, title: &str) -> Result<(), UiError> {
        self.record(GuardCall::LiftAlone(title.to_owned()))
    }

    fn watch_entries(&self, title: &str) -> Result<(), UiError> {
        self.record(GuardCall::WatchEntries(title.to_owned()))
    }

    fn unwatch_entries(&self, title: &str) {
        lock(&self.calls).push(GuardCall::UnwatchEntries(title.to_owned()));
    }

    fn lay_under(&self, title: &str, under: &str) -> Result<(), UiError> {
        self.record(GuardCall::LayUnder(title.to_owned(), under.to_owned()))
    }

    fn remember(&self) -> FocusToken {
        let token = FocusToken::new(self.last.fetch_add(1, Ordering::SeqCst) + 1);
        lock(&self.calls).push(GuardCall::Remember(token));
        token
    }

    fn give_back(&self, token: FocusToken) {
        lock(&self.calls).push(GuardCall::GiveBack(token));
    }
}

/// A backdrop that records each applied material, or refuses while an error is set.
#[derive(Debug, Default)]
pub struct FakeBackdrop {
    applied: Mutex<Vec<(String, BackdropKind, bool)>>,
    error: Mutex<Option<UiError>>,
}

impl FakeBackdrop {
    /// A backdrop with nothing applied.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Every applied title, material and dark flag, oldest first.
    #[must_use]
    pub fn applied(&self) -> Vec<(String, BackdropKind, bool)> {
        lock(&self.applied).clone()
    }

    /// Refuses every later call with `error`, or takes them again with `None`.
    pub fn set_error(&self, error: Option<UiError>) {
        *lock(&self.error) = error;
    }
}

impl Backdrop for FakeBackdrop {
    fn apply(&self, title: &str, kind: BackdropKind, dark: bool) -> Result<(), UiError> {
        if let Some(error) = *lock(&self.error) {
            return Err(error);
        }
        lock(&self.applied).push((title.to_owned(), kind, dark));
        Ok(())
    }
}

#[cfg(test)]
mod tests;
