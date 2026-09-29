//! A global log capture for the panic tests: it keeps every line and hushes only the planned panics.

use std::any::Any;
use std::collections::BTreeMap;
use std::fmt;
use std::panic;
use std::sync::{Mutex, Once, PoisonError};
use tracing::field::{Field, Visit};
use tracing::span::{Attributes, Id, Record};
use tracing::{Level, Metadata, Subscriber};

/// The id the capture hands every span; the code under test opens none.
const SPAN_ID: u64 = 1;

/// A log line's fields by name.
pub type Fields = BTreeMap<&'static str, String>;
/// One captured log line: its level and its fields.
pub type Line = (Level, Fields);

/// Tells whether a panic payload is one the test planned.
pub type Planned = fn(&(dyn Any + Send)) -> bool;

static SETUP: Once = Once::new();
static LINES: Mutex<Vec<Line>> = Mutex::new(Vec::new());

/// Installs the quiet hook and the global capture once, before any test logs, so no callsite misses it.
pub fn install(planned: Planned) {
    SETUP.call_once(|| {
        let usual = panic::take_hook();
        panic::set_hook(Box::new(move |info| {
            if !planned(info.payload()) {
                usual(info);
            }
        }));
        let installed = tracing::subscriber::set_global_default(Capture);
        assert!(installed.is_ok(), "only this setup sets the subscriber");
    });
}

/// Every line captured so far.
pub fn lines() -> Vec<Line> {
    LINES.lock().unwrap_or_else(PoisonError::into_inner).clone()
}

/// The global subscriber: it keeps every log line in `LINES`.
struct Capture;

/// Collects one event's fields.
#[derive(Default)]
struct Recorder(Fields);

impl Visit for Recorder {
    fn record_str(&mut self, field: &Field, value: &str) {
        self.0.insert(field.name(), value.to_owned());
    }

    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        self.0.insert(field.name(), format!("{value:?}"));
    }
}

impl Subscriber for Capture {
    fn enabled(&self, _: &Metadata<'_>) -> bool {
        true
    }

    fn new_span(&self, _: &Attributes<'_>) -> Id {
        Id::from_u64(SPAN_ID)
    }

    fn record(&self, _: &Id, _: &Record<'_>) {}

    fn record_follows_from(&self, _: &Id, _: &Id) {}

    fn event(&self, event: &tracing::Event<'_>) {
        let mut fields = Recorder::default();
        event.record(&mut fields);
        let line = (*event.metadata().level(), fields.0);
        LINES
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(line);
    }

    fn enter(&self, _: &Id) {}

    fn exit(&self, _: &Id) {}
}
