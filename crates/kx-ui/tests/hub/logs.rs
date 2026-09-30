//! A log capture for one test thread: it keeps each warning's message.

use std::fmt;
use std::sync::{Arc, Mutex, PoisonError};
use tracing::field::{Field, Visit};
use tracing::span::{Attributes, Id, Record};
use tracing::{Event, Level, Metadata, Subscriber};

/// The id the capture hands every span; the hub opens none.
const SPAN_ID: u64 = 1;
/// The field that holds a log line's message.
const MESSAGE: &str = "message";

/// Keeps the messages of the warnings logged on this thread.
#[derive(Clone, Default)]
struct Warnings(Arc<Mutex<Vec<String>>>);

/// The warnings `f` logs on this thread, in order.
pub fn warnings_of(f: impl FnOnce()) -> Vec<String> {
    let seen = Warnings::default();
    tracing::subscriber::with_default(seen.clone(), f);
    seen.0
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .clone()
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

impl Subscriber for Warnings {
    fn enabled(&self, _: &Metadata<'_>) -> bool {
        true
    }

    fn new_span(&self, _: &Attributes<'_>) -> Id {
        Id::from_u64(SPAN_ID)
    }

    fn record(&self, _: &Id, _: &Record<'_>) {}

    fn record_follows_from(&self, _: &Id, _: &Id) {}

    fn event(&self, event: &Event<'_>) {
        if *event.metadata().level() == Level::WARN {
            let mut message = Message::default();
            event.record(&mut message);
            self.0
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .push(message.0);
        }
    }

    fn enter(&self, _: &Id) {}

    fn exit(&self, _: &Id) {}
}
