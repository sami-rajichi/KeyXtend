//! A fake input injector that records every stroke in order.

use crate::lock;
use kx_module_api::Redacted;
use kx_module_api::ports::{InjectError, InputInjector, Scan, TextRoute};
use std::sync::Mutex;

/// One call the fake injector took.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Stroke {
    /// `tap(key)`.
    Tap(Scan),
    /// `chord(held, key)`.
    Chord(Vec<Scan>, Scan),
    /// `text(text, route)`; the text shows only through `Redacted::expose`.
    Text(Redacted<String>, TextRoute),
}

/// An injector that records each stroke in order, or refuses them all while an error is set.
#[derive(Debug, Default)]
pub struct FakeInjector {
    strokes: Mutex<Vec<Stroke>>,
    error: Mutex<Option<InjectError>>,
    target: Mutex<Option<String>>,
}

impl FakeInjector {
    /// An injector with no strokes, no error and no app in front.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Every stroke so far, oldest first.
    #[must_use]
    pub fn strokes(&self) -> Vec<Stroke> {
        lock(&self.strokes).clone()
    }

    /// Forgets the strokes so far.
    pub fn clear(&self) {
        lock(&self.strokes).clear();
    }

    /// Refuses every later call with `error`, or takes them again with `None`.
    pub fn set_error(&self, error: Option<InjectError>) {
        *lock(&self.error) = error;
    }

    /// Names the app in front that `target_app` reports.
    pub fn set_target(&self, app: Option<&str>) {
        *lock(&self.target) = app.map(str::to_owned);
    }

    fn record(&self, stroke: Stroke) -> Result<(), InjectError> {
        if let Some(error) = *lock(&self.error) {
            return Err(error);
        }
        lock(&self.strokes).push(stroke);
        Ok(())
    }
}

impl InputInjector for FakeInjector {
    fn tap(&self, key: Scan) -> Result<(), InjectError> {
        self.record(Stroke::Tap(key))
    }

    fn chord(&self, held: &[Scan], key: Scan) -> Result<(), InjectError> {
        self.record(Stroke::Chord(held.to_vec(), key))
    }

    fn text(&self, text: &Redacted<String>, route: TextRoute) -> Result<(), InjectError> {
        self.record(Stroke::Text(text.clone(), route))
    }

    fn target_app(&self) -> Option<String> {
        lock(&self.target).clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    const A: Scan = Scan::new(0x1E);
    const SHIFT: Scan = Scan::new(0x2A);
    const CTRL: Scan = Scan::new(0x1D);
    const SECRET: &str = "hunter2";

    fn secret() -> Redacted<String> {
        Redacted::new(SECRET.to_owned())
    }

    #[test]
    fn strokes_are_recorded_in_order() {
        let fake = FakeInjector::new();
        fake.tap(A).unwrap();
        fake.chord(&[SHIFT, CTRL], A).unwrap();
        fake.text(&secret(), TextRoute::Keys).unwrap();
        let want = [
            Stroke::Tap(A),
            Stroke::Chord(vec![SHIFT, CTRL], A),
            Stroke::Text(secret(), TextRoute::Keys),
        ];
        assert_eq!(fake.strokes(), want);
    }

    #[test]
    fn text_shows_only_when_the_test_exposes_it() {
        let fake = FakeInjector::new();
        fake.text(&secret(), TextRoute::Unicode).unwrap();
        let shown = format!("{fake:?} {:?}", fake.strokes());
        assert!(!shown.contains(SECRET), "{shown}");
        let Stroke::Text(text, _) = &fake.strokes()[0] else {
            panic!("not a text stroke");
        };
        assert_eq!(text.expose(), SECRET);
    }

    #[test]
    fn a_set_error_refuses_every_call_and_records_nothing() {
        let fake = FakeInjector::new();
        fake.set_error(Some(InjectError::Unmappable));
        assert_eq!(fake.tap(A), Err(InjectError::Unmappable));
        assert_eq!(fake.chord(&[SHIFT], A), Err(InjectError::Unmappable));
        let text = fake.text(&secret(), TextRoute::Keys);
        assert_eq!(text, Err(InjectError::Unmappable));
        assert!(fake.strokes().is_empty());
        fake.set_error(None);
        assert_eq!(fake.tap(A), Ok(()));
    }

    #[test]
    fn clear_forgets_the_strokes() {
        let fake = FakeInjector::new();
        fake.tap(A).unwrap();
        fake.clear();
        assert!(fake.strokes().is_empty());
    }

    #[test]
    fn the_target_app_is_the_one_set() {
        let fake = FakeInjector::new();
        assert_eq!(fake.target_app(), None);
        fake.set_target(Some("notepad.exe"));
        assert_eq!(fake.target_app().as_deref(), Some("notepad.exe"));
    }

    #[test]
    fn it_serves_as_the_port_from_another_thread() {
        let fake = Arc::new(FakeInjector::new());
        let port: Arc<dyn InputInjector> = fake.clone();
        std::thread::spawn(move || port.tap(A))
            .join()
            .unwrap()
            .unwrap();
        assert_eq!(fake.strokes(), [Stroke::Tap(A)]);
    }
}
