//! A log capture for one test thread: each line's level and message. One capture serves the whole test binary, so
//! `tracing` never caches a log line as unheard while another test thread starts its own.

use std::cell::RefCell;
use std::fmt;
use std::sync::Once;
use tracing::field::{Field, Visit};
use tracing::span::{Attributes, Id, Record};
use tracing::{Event, Level, Metadata, Subscriber};

/// The id the capture hands every span; the module opens none.
const SPAN_ID: u64 = 1;
/// The field that holds a log line's message.
const MESSAGE: &str = "message";

thread_local! {
    /// The lines logged on this thread while `lines_of` runs here, else nothing.
    static SEEN: RefCell<Option<Vec<(Level, String)>>> = const { RefCell::new(None) };
}

/// Installs the capture for the whole test binary, once.
static INSTALL: Once = Once::new();

/// The level and message of each line `f` logs on this thread, in order.
pub fn lines_of(f: impl FnOnce()) -> Vec<(Level, String)> {
    INSTALL.call_once(|| {
        // Only this file sets a capture in this test binary, so the first call always installs it.
        let _ = tracing::subscriber::set_global_default(Lines);
    });
    SEEN.with(|seen| *seen.borrow_mut() = Some(Vec::new()));
    f();
    SEEN.with(|seen| seen.borrow_mut().take().unwrap_or_default())
}

/// Hands each line to the thread that logs it.
struct Lines;

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
    fn enabled(&self, _: &Metadata<'_>) -> bool {
        true
    }

    fn new_span(&self, _: &Attributes<'_>) -> Id {
        Id::from_u64(SPAN_ID)
    }

    fn record(&self, _: &Id, _: &Record<'_>) {}

    fn record_follows_from(&self, _: &Id, _: &Id) {}

    fn event(&self, event: &Event<'_>) {
        let mut message = Message::default();
        event.record(&mut message);
        let line = (*event.metadata().level(), message.0);
        SEEN.with(|seen| seen.borrow_mut().as_mut().map(|lines| lines.push(line)));
    }

    fn enter(&self, _: &Id) {}

    fn exit(&self, _: &Id) {}
}
