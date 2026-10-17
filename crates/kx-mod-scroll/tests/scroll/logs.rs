//! A log capture for one test thread, each line's level and fields; one capture serves the whole test binary, so
//! `tracing` never caches a line as unheard while another test thread starts its own. It also hushes planned panics.

use std::any::Any;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fmt;
use std::panic;
use std::sync::Once;
use tracing::field::{Field, Visit};
use tracing::span::{Attributes, Id, Record};
use tracing::{Event, Level, Metadata, Subscriber};

/// The id the capture hands every span; the module opens none.
const SPAN_ID: u64 = 1;
/// The field that holds a log line's message.
pub const MESSAGE: &str = "message";

/// A log line's fields by name, the message under `MESSAGE`.
pub type Fields = BTreeMap<&'static str, String>;

thread_local! {
    /// The lines logged on this thread while `lines_of` runs here, else nothing.
    static SEEN: RefCell<Option<Vec<(Level, Fields)>>> = const { RefCell::new(None) };
}

/// Installs the capture and the quiet panic hook for the whole test binary, once.
static INSTALL: Once = Once::new();

/// The payload of a panic a test plans; the panic hook prints nothing for it.
pub struct Planned;

/// The lines `f` logs on this thread, in order.
pub fn lines_of(f: impl FnOnce()) -> Vec<(Level, Fields)> {
    INSTALL.call_once(|| {
        let usual = panic::take_hook();
        panic::set_hook(Box::new(move |info| {
            if !<dyn Any + Send>::is::<Planned>(info.payload()) {
                usual(info);
            }
        }));
        // Only this file sets a capture in this test binary, so the first call always installs it.
        let _ = tracing::subscriber::set_global_default(Lines);
    });
    SEEN.with(|seen| *seen.borrow_mut() = Some(Vec::new()));
    f();
    SEEN.with(|seen| seen.borrow_mut().take().unwrap_or_default())
}

/// Hands each line to the thread that logs it.
struct Lines;

/// Reads a line's fields: text as it is, anything else as its debug text.
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
        let mut fields = Recorder::default();
        event.record(&mut fields);
        let line = (*event.metadata().level(), fields.0);
        SEEN.with(|seen| {
            if let Some(lines) = seen.borrow_mut().as_mut() {
                lines.push(line);
            }
        });
    }

    fn enter(&self, _: &Id) {}

    fn exit(&self, _: &Id) {}
}
