//! A log capture for one test thread: it keeps each warning's message. One capture serves the whole test binary, so
//! `tracing` never caches a log line as unheard while another test thread starts its own.

use std::cell::RefCell;
use std::fmt;
use std::sync::Once;
use tracing::field::{Field, Visit};
use tracing::span::{Attributes, Id, Record};
use tracing::{Event, Level, Metadata, Subscriber};

/// The id the capture hands every span; the hub opens none.
const SPAN_ID: u64 = 1;
/// The field that holds a log line's message.
const MESSAGE: &str = "message";

thread_local! {
    /// The warnings logged on this thread while `warnings_of` runs here, else nothing.
    static SEEN: RefCell<Option<Vec<String>>> = const { RefCell::new(None) };
}

/// Installs the capture for the whole test binary, once.
static INSTALL: Once = Once::new();

/// The warnings `f` logs on this thread, in order.
pub fn warnings_of(f: impl FnOnce()) -> Vec<String> {
    INSTALL.call_once(|| {
        // Only this file sets a capture in these test binaries, so the first call always installs it.
        let _ = tracing::subscriber::set_global_default(Warnings);
    });
    SEEN.with(|seen| *seen.borrow_mut() = Some(Vec::new()));
    f();
    SEEN.with(|seen| seen.borrow_mut().take().unwrap_or_default())
}

/// Hands each warning to the thread that logs it.
struct Warnings;

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
        if *event.metadata().level() != Level::WARN {
            return;
        }
        let mut message = Message::default();
        event.record(&mut message);
        SEEN.with(|seen| {
            if let Some(lines) = seen.borrow_mut().as_mut() {
                lines.push(message.0);
            }
        });
    }

    fn enter(&self, _: &Id) {}

    fn exit(&self, _: &Id) {}
}
