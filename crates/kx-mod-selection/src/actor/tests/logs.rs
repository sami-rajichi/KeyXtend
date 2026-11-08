//! What the actor logs when Translate fails: fixed texts and error kinds, never the selection, its link or a box.

use std::cell::RefCell;
use std::fmt::{self, Write as _};
use std::sync::Once;

use kx_module_api::WriteError;
use kx_module_api::ports::{LINK_SCHEME, LinkError, SelChip, SelError, SelKind};
use toml::Value;
use tracing::field::{Field, Visit};
use tracing::span::{Attributes, Id, Record};
use tracing::{Event, Metadata, Subscriber};

use super::rig::Rig;
use super::spots::sel;
use crate::settings::TRANSLATE_OK_KEY;

/// A selection that appears nowhere else.
const SECRET: &str = "zebra-quartz";
/// The line box's row, which no line may show.
const ROW: i32 = 4_321;

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

/// Clicks Translate on a selection of `SECRET`.
fn translate(rig: &mut Rig) {
    rig.watch.set_text(SECRET);
    rig.focus(sel(SelKind::ReadOnly, ROW));
    rig.act(SelChip::Translate);
}

#[test]
fn a_failed_read_open_or_save_logs_no_text_link_or_box() {
    let lines = lines_of(|| {
        let mut rig = Rig::build(|h| h.put(TRANSLATE_OK_KEY, Value::Boolean(true)));
        rig.watch.fail_reads(SelError::Refused);
        translate(&mut rig);
        rig.watch.heal();
        rig.web.set_error(Some(LinkError::Unavailable));
        translate(&mut rig);
        let mut rig = Rig::new();
        rig.host.sink.refuse(Some(WriteError::Unsaved));
        translate(&mut rig);
        rig.reply(true);
    });
    assert_eq!(lines.len(), 3, "{lines:?}");
    let row = ROW.to_string();
    for line in &lines {
        let bad = [SECRET, LINK_SCHEME, row.as_str()];
        assert!(bad.iter().all(|b| !line.contains(b)), "{line}");
    }
}
