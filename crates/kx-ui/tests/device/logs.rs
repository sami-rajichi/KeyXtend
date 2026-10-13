//! A log capture for the whole test binary: it keeps each info and warning message, in order.

use std::fmt;
use std::sync::{Arc, Mutex, PoisonError};
use tracing::field::{Field, Visit};
use tracing::span::{Attributes, Id, Record};
use tracing::{Event, Level, Metadata, Subscriber};

/// The id the capture hands every span; the code under test opens none.
const SPAN_ID: u64 = 1;
/// The field that holds a log line's message.
const MESSAGE: &str = "message";

/// The messages logged so far, with their levels.
#[derive(Clone, Default)]
pub struct Lines(Arc<Mutex<Vec<(Level, String)>>>);

impl Lines {
    /// Captures every log line of the process from now on; false when another capture was set first.
    pub fn install() -> Option<Self> {
        let lines = Self::default();
        tracing::subscriber::set_global_default(lines.clone()).ok()?;
        Some(lines)
    }

    /// How many lines at `level` so far hold `text`.
    pub fn count(&self, level: Level, text: &str) -> usize {
        let lines = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        lines
            .iter()
            .filter(|(l, m)| *l == level && m.contains(text))
            .count()
    }
}

/// Reads a line's message.
#[derive(Default)]
struct Message(String);

impl Visit for Message {
    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        if field.name() == MESSAGE {
            self.0 = format!("{value:?}");
        }
    }
}

impl Subscriber for Lines {
    fn enabled(&self, meta: &Metadata<'_>) -> bool {
        *meta.level() <= Level::INFO
    }

    fn new_span(&self, _: &Attributes<'_>) -> Id {
        Id::from_u64(SPAN_ID)
    }

    fn record(&self, _: &Id, _: &Record<'_>) {}

    fn record_follows_from(&self, _: &Id, _: &Id) {}

    fn event(&self, event: &Event<'_>) {
        let mut message = Message::default();
        event.record(&mut message);
        let level = *event.metadata().level();
        let mut lines = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        lines.push((level, message.0));
    }

    fn enter(&self, _: &Id) {}

    fn exit(&self, _: &Id) {}
}
