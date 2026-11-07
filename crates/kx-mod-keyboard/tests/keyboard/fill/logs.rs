//! A fill's values never show: no line logged while it types, waits, fails or meets a second fill, and no `Debug` of
//! the module or the request, captured on the test's thread, where the bus runs the handlers.

use super::{GAP, Quick, gap_ms, line};
use kx_module_api::ports::{FillPart, FillType, InjectError};
use std::cell::RefCell;
use std::fmt::{self, Write as _};
use std::sync::Once;
use tracing::field::{Field, Visit};
use tracing::span::{Attributes, Id, Record};
use tracing::{Event, Metadata, Subscriber};

/// A value that appears nowhere else in the output.
const SECRET: &str = "zebra-quartz-7";

thread_local! {
    /// Every line logged on this thread while `lines_of` runs here.
    static SEEN: RefCell<Option<Vec<String>>> = const { RefCell::new(None) };
}

/// Installs the capture for the whole test binary, once.
static INSTALL: Once = Once::new();

/// Each line `f` logs on this thread, its fields joined.
fn lines_of(f: impl FnOnce()) -> Vec<String> {
    INSTALL.call_once(|| {
        // Only this file sets a capture in this test binary, so the first call installs it.
        let _ = tracing::subscriber::set_global_default(Lines);
    });
    SEEN.with(|seen| *seen.borrow_mut() = Some(Vec::new()));
    f();
    SEEN.with(|seen| seen.borrow_mut().take().unwrap_or_default())
}

/// Hands each line to the thread that logs it.
struct Lines;

/// Joins a line's fields, each as its debug text.
#[derive(Default)]
struct Recorder(String);

impl Visit for Recorder {
    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        let _ = write!(self.0, "{}={value:?} ", field.name());
    }
}

impl Subscriber for Lines {
    fn enabled(&self, _: &Metadata<'_>) -> bool {
        true
    }

    fn new_span(&self, _: &Attributes<'_>) -> Id {
        Id::from_u64(1)
    }

    fn record(&self, _: &Id, _: &Record<'_>) {}

    fn record_follows_from(&self, _: &Id, _: &Id) {}

    fn event(&self, event: &Event<'_>) {
        let mut line = Recorder::default();
        event.record(&mut line);
        SEEN.with(|seen| seen.borrow_mut().as_mut().map(|l| l.push(line.0.clone())));
    }

    fn enter(&self, _: &Id) {}

    fn exit(&self, _: &Id) {}
}

/// A fill of the secret, a Tab and the secret again.
fn both() -> Vec<FillPart> {
    vec![line(SECRET), FillPart::Tab, line(SECRET)]
}

#[test]
fn no_line_logged_while_a_fill_types_waits_or_fails_shows_its_value() {
    let q = Quick::opened(None);
    let lines = lines_of(|| {
        q.fill(1, both());
        q.fill(2, vec![line(SECRET)]);
        q.b.rig.tap("c-0x0010").unwrap();
        q.b.rig
            .fakes
            .injector
            .set_error(Some(InjectError::Unmappable));
        q.relabel_in(gap_ms());
        q.fill(3, both());
    });
    for line in lines {
        assert!(!line.contains(SECRET), "{line}");
    }
}

#[test]
fn neither_the_module_nor_the_request_shows_a_value() {
    let q = Quick::quiet(None);
    q.fill(1, both());
    let ask = FillType {
        round: 1,
        parts: both(),
        tab_gap: GAP,
    };
    let shown = format!("{:?} {ask:?}", q.b.rig.module);
    assert!(!shown.contains(SECRET), "{shown}");
}
