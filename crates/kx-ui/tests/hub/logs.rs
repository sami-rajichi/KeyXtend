//! A log capture for one test thread: it keeps each warning's message, or every line with all its fields. One capture
//! serves the whole test binary, so `tracing` never caches a log line as unheard while another test thread starts its own.

use std::cell::RefCell;
use std::fmt::{self, Write};
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
    /// Every line logged on this thread while `lines_of` runs here, else nothing.
    static ALL: RefCell<Option<Vec<String>>> = const { RefCell::new(None) };
}

/// Installs the capture for the whole test binary, once.
static INSTALL: Once = Once::new();

/// Installs the capture unless it is in already.
fn install() {
    INSTALL.call_once(|| {
        // Only this file sets a capture in these test binaries, so the first call always installs it.
        let _ = tracing::subscriber::set_global_default(Warnings);
    });
}

/// The warnings `f` logs on this thread, in order.
pub fn warnings_of(f: impl FnOnce()) -> Vec<String> {
    install();
    SEEN.with(|seen| *seen.borrow_mut() = Some(Vec::new()));
    f();
    SEEN.with(|seen| seen.borrow_mut().take().unwrap_or_default())
}

/// Every line `f` logs on this thread at any level, each as its fields `name=value`, in order.
#[allow(
    dead_code,
    reason = "only the hub tests read every line; other test binaries share this file"
)]
pub fn lines_of(f: impl FnOnce()) -> Vec<String> {
    install();
    ALL.with(|all| *all.borrow_mut() = Some(Vec::new()));
    f();
    ALL.with(|all| all.borrow_mut().take().unwrap_or_default())
}

/// Hands each line to the thread that logs it.
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

/// Reads every field of a line.
#[derive(Default)]
struct Fields(String);

impl Visit for Fields {
    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        // Writing to a String cannot fail.
        let _ = write!(self.0, " {}={value:?}", field.name());
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
        ALL.with(|all| {
            if let Some(lines) = all.borrow_mut().as_mut() {
                let mut fields = Fields::default();
                event.record(&mut fields);
                lines.push(fields.0);
            }
        });
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
