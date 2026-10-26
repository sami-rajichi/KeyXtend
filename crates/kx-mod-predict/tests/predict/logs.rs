//! The actor's logs carry counts and error kinds only: never a typed word, a learned word or an email.

use crate::rig::{Rig, plant};
use kx_mod_predict::learn::{LearnFormatError, Learner};
use kx_mod_predict::{OTHER_LANGUAGE, PredictSettings, STOPPED_LINE};
use kx_module_api::ports::{FieldKind, SealedError, WordsAsk};
use std::cell::RefCell;
use std::fmt::{self, Write as _};
use std::sync::Once;
use tracing::field::{Field, Visit};
use tracing::span::{Attributes, Id, Record};
use tracing::{Event, Metadata, Subscriber};

/// The texts the test types, none of which may reach a log.
const SECRETS: [&str; 3] = ["hello", "sami@x.org", "مرحبا"];

thread_local! {
    /// Every field of every line logged on this thread while `lines_of` runs here.
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

#[test]
fn no_log_line_shows_a_typed_word_a_learned_word_or_an_email() {
    let lines = lines_of(|| {
        let mut rig = Rig::build(|f, _| plant(f, "en", &["hello", "مرحبا"]));
        rig.lang("en");
        rig.focus(FieldKind::Text);
        rig.typed("hel");
        rig.ask(WordsAsk::Forget);
        rig.pick(0);
        rig.typed("lo مرحبا sami@x.org ");
        rig.fakes.store.fail_save(SealedError::Io);
        rig.lang("ar");
        rig.stop();
    });
    assert!(lines.iter().any(|l| l.contains(STOPPED_LINE)), "{lines:?}");
    for line in &lines {
        for secret in SECRETS {
            assert!(!line.contains(secret), "{line}");
        }
    }
}

#[test]
fn a_learner_that_does_not_read_logs_why() {
    let name = kx_mod_predict::learned_blob("en").unwrap();
    let damaged = lines_of(|| {
        let mut rig = Rig::build(|f, _| f.store.plant(&name, b"junk".to_vec()));
        rig.lang("en");
    });
    let why = LearnFormatError::BadMagic.to_string();
    assert!(damaged.iter().any(|l| l.contains(&why)), "{damaged:?}");
    let cfg = PredictSettings::from_defaults().unwrap().learn_config();
    let other = lines_of(|| {
        let mut rig = Rig::build(|f, _| {
            let fr = Learner::new("fr", cfg).unwrap();
            f.store.plant(&name, fr.to_bytes());
        });
        rig.lang("en");
    });
    assert!(
        other.iter().any(|l| l.contains(OTHER_LANGUAGE)),
        "{other:?}"
    );
}
