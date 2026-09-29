//! Fakes of the UI ports: screens, the window guard and backdrops.

use crate::lock;
use kx_module_api::ports::{Point, Rect};
use kx_platform::ui::{
    Backdrop, BackdropKind, FocusToken, ScreenArea, Screens, UiError, WindowGuard,
};
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

/// Screens with work areas and a pointer the test sets.
#[derive(Debug, Default)]
pub struct FakeScreens {
    areas: Mutex<Vec<ScreenArea>>,
    cursor: Mutex<Point>,
}

impl FakeScreens {
    /// Screens with `areas` and the pointer at `cursor`.
    #[must_use]
    pub fn new(areas: Vec<ScreenArea>, cursor: Point) -> Self {
        Self {
            areas: Mutex::new(areas),
            cursor: Mutex::new(cursor),
        }
    }

    /// Replaces the screens, as when one is plugged in or out.
    pub fn set_areas(&self, areas: Vec<ScreenArea>) {
        *lock(&self.areas) = areas;
    }

    /// Moves the pointer.
    pub fn set_cursor(&self, at: Point) {
        *lock(&self.cursor) = at;
    }
}

impl Screens for FakeScreens {
    fn work_areas(&self) -> Vec<ScreenArea> {
        lock(&self.areas).clone()
    }

    fn cursor(&self) -> Result<Point, UiError> {
        Ok(*lock(&self.cursor))
    }
}

/// One call the fake window guard took.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GuardCall {
    /// `guard_own(spare)`.
    GuardOwn(Option<String>),
    /// `shape(title, rects)`.
    Shape(String, Vec<Rect>),
    /// `remember`, with the token it gave.
    Remember(FocusToken),
    /// `give_back(token)`.
    GiveBack(FocusToken),
}

/// A window guard that records its calls; `guard_own` reports the count the test chose.
#[derive(Debug, Default)]
pub struct FakeWindowGuard {
    count: usize,
    calls: Mutex<Vec<GuardCall>>,
    last: AtomicU64,
}

impl FakeWindowGuard {
    /// A guard whose `guard_own` says it guards `count` windows.
    #[must_use]
    pub fn new(count: usize) -> Self {
        Self {
            count,
            ..Self::default()
        }
    }

    /// Every call so far, oldest first.
    #[must_use]
    pub fn calls(&self) -> Vec<GuardCall> {
        lock(&self.calls).clone()
    }

    fn record(&self, call: GuardCall) {
        lock(&self.calls).push(call);
    }
}

impl WindowGuard for FakeWindowGuard {
    fn guard_own(&self, spare: Option<&str>) -> Result<usize, UiError> {
        self.record(GuardCall::GuardOwn(spare.map(str::to_owned)));
        Ok(self.count)
    }

    fn shape(&self, title: &str, rects: &[Rect]) -> Result<(), UiError> {
        self.record(GuardCall::Shape(title.to_owned(), rects.to_vec()));
        Ok(())
    }

    fn remember(&self) -> FocusToken {
        let token = FocusToken::new(self.last.fetch_add(1, Ordering::SeqCst) + 1);
        self.record(GuardCall::Remember(token));
        token
    }

    fn give_back(&self, token: FocusToken) {
        self.record(GuardCall::GiveBack(token));
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
mod tests {
    use super::*;

    const TITLE: &str = "KeyXtend";

    fn area() -> ScreenArea {
        let rect = Rect {
            x: 0,
            y: 0,
            w: 1920,
            h: 1040,
        };
        ScreenArea {
            rect,
            dpi: 144,
            primary: true,
        }
    }

    #[test]
    fn screens_report_what_the_test_set() {
        let fake = FakeScreens::new(vec![area()], Point { x: 5, y: 6 });
        assert_eq!(fake.work_areas(), [area()]);
        assert_eq!(fake.cursor(), Ok(Point { x: 5, y: 6 }));
        fake.set_cursor(Point { x: 7, y: 8 });
        fake.set_areas(Vec::new());
        assert_eq!(fake.cursor(), Ok(Point { x: 7, y: 8 }));
        assert!(fake.work_areas().is_empty());
    }

    #[test]
    fn the_guard_records_every_call_in_order() {
        let fake = FakeWindowGuard::new(3);
        let rects = [area().rect];
        assert_eq!(fake.guard_own(Some(TITLE)), Ok(3));
        assert_eq!(fake.shape(TITLE, &rects), Ok(()));
        let first = fake.remember();
        let second = fake.remember();
        assert_ne!(first, second);
        fake.give_back(first);
        let want = [
            GuardCall::GuardOwn(Some(TITLE.into())),
            GuardCall::Shape(TITLE.into(), rects.to_vec()),
            GuardCall::Remember(first),
            GuardCall::Remember(second),
            GuardCall::GiveBack(first),
        ];
        assert_eq!(fake.calls(), want);
    }

    #[test]
    fn a_backdrop_is_recorded_or_refused() {
        let fake = FakeBackdrop::new();
        assert_eq!(fake.apply(TITLE, BackdropKind::Mica, true), Ok(()));
        fake.set_error(Some(UiError::Refused(1)));
        let refused = fake.apply(TITLE, BackdropKind::Acrylic, false);
        assert_eq!(refused, Err(UiError::Refused(1)));
        assert_eq!(fake.applied(), [(TITLE.into(), BackdropKind::Mica, true)]);
    }

    #[test]
    fn a_focus_token_is_opaque() {
        let fake = FakeWindowGuard::new(0);
        assert_eq!(fake.remember(), FocusToken::new(1));
    }
}
