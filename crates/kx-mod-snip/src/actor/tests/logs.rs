//! What the actor logs for the debug times: the Snip key's wait for the freeze, in whole ms and nothing else.

use std::cell::RefCell;
use std::fmt;
use std::sync::Once;
use std::time::Duration;

use tracing::field::{Field, Visit};
use tracing::span::{Attributes, Id, Record};
use tracing::{Event, Metadata, Subscriber};

use super::super::Msg;
use super::rig::Rig;

/// A logged line: each field's name with its debug text, in order.
type Line = Vec<(String, String)>;

thread_local! {
    /// Every line logged on this thread while `lines_of` runs here.
    static SEEN: RefCell<Option<Vec<Line>>> = const { RefCell::new(None) };
}

/// Installs the capture for the whole test binary, once. A scoped one (`with_default`) would miss a line whose
/// callsite another test's thread reached first, as tracing then keeps that callsite off for every thread.
static INSTALL: Once = Once::new();

/// Each line `f` logs on this thread.
fn lines_of(f: impl FnOnce()) -> Vec<Line> {
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

/// Keeps a line's fields.
#[derive(Default)]
struct Recorder(Line);

impl Visit for Recorder {
    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        self.0.push((field.name().to_owned(), format!("{value:?}")));
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
        SEEN.with(|seen| seen.borrow_mut().as_mut().map(|l| l.push(line.0)));
    }

    fn enter(&self, _: &Id) {}

    fn exit(&self, _: &Id) {}
}

#[test]
fn the_snip_key_logs_its_ms_to_the_freeze_and_no_other_field() {
    let mut rig = Rig::new();
    let aside = Duration::from_millis(u64::from(rig.actor.settings.aside_ms));
    let lines = lines_of(|| {
        rig.send(Msg::Key);
        rig.advance(aside);
    });
    assert_eq!(
        rig.fake.freeze.caps().len(),
        1,
        "the freeze ran after aside_ms"
    );
    let timed: Vec<&Line> = lines
        .iter()
        .filter(|l| l.iter().any(|(name, _)| name == "ms"))
        .collect();
    assert_eq!(timed.len(), 1, "one timed line: {lines:?}");
    let names: Vec<&str> = timed[0].iter().map(|(name, _)| name.as_str()).collect();
    assert_eq!(names, ["message", "ms"], "{lines:?}");
    assert_eq!(
        timed[0][1].1,
        aside.as_millis().to_string(),
        "the wait on the clock"
    );
}
