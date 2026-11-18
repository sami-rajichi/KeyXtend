//! A key typed on our keyboard into the key page's line (ADR-0049): the letters go to the voice module, not the app,
//! Enter seals the key, and no view, no text for the keyboard, no log line and no span field holds it.

use super::{dots, groq, key_page, sealed, unseen};
use crate::rig::{Rig, WAIT, hear};
use kx_module_api::ports::{
    FaceAction, KeyId, PanelLine, ServiceSheet, VoiceLine, VoiceSheet, VoiceView,
};
use std::fmt::{self, Write as _};
use std::sync::atomic::{AtomicBool, Ordering::SeqCst};
use std::sync::mpsc::Receiver;
use std::sync::{Mutex, Once, PoisonError};
use tracing::field::{Field, Visit};
use tracing::span::{Attributes, Id, Record};
use tracing::{Event, Metadata, Subscriber};

/// The board's letter keys from Q on, as the English layout types them.
const LETTERS: &str = "qwertyuiop";
/// The board's scan code of the first.
const FIRST: u16 = 0x10;
/// How many letters the test types.
const TYPED: usize = 24;
/// What the capture's self-check logs as span fields: one as the span starts, one recorded later.
const PROBES: [&str; 2] = ["kx-probe-a", "kx-probe-b"];

/// Every line logged on any thread while the capture is on.
static SEEN: Mutex<Vec<String>> = Mutex::new(Vec::new());
/// True while the capture keeps lines.
static ON: AtomicBool = AtomicBool::new(false);
/// Installs the capture once for the test binary.
static INSTALL: Once = Once::new();

/// Keeps every line logged anywhere and every span's fields, each as their debug text.
struct Lines;

/// Joins a line's fields.
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

    fn new_span(&self, span: &Attributes<'_>) -> Id {
        let mut line = Recorder::default();
        span.record(&mut line);
        keep(line);
        Id::from_u64(1)
    }

    fn record(&self, _: &Id, values: &Record<'_>) {
        let mut line = Recorder::default();
        values.record(&mut line);
        keep(line);
    }

    fn record_follows_from(&self, _: &Id, _: &Id) {}

    fn event(&self, event: &Event<'_>) {
        let mut line = Recorder::default();
        event.record(&mut line);
        keep(line);
    }

    fn enter(&self, _: &Id) {}

    fn exit(&self, _: &Id) {}
}

/// Keeps `line` while the capture is on.
fn keep(line: Recorder) {
    if ON.load(SeqCst) {
        SEEN.lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(line.0);
    }
}

/// Turns the capture on afresh, installed once for the test binary, and checks that it hears span fields too.
fn capture() {
    INSTALL.call_once(|| {
        assert!(
            tracing::subscriber::set_global_default(Lines).is_ok(),
            "the capture"
        );
    });
    SEEN.lock().unwrap_or_else(PoisonError::into_inner).clear();
    ON.store(true, SeqCst);
    let span = tracing::info_span!("probe", first = PROBES[0], later = tracing::field::Empty);
    span.record("later", PROBES[1]);
    let seen = logged();
    for probe in PROBES {
        assert!(
            seen.iter().any(|line| line.contains(probe)),
            "the capture hears {probe}"
        );
    }
    ON.store(true, SeqCst);
}

/// The lines kept since the capture was turned on; it turns off.
fn logged() -> Vec<String> {
    ON.store(false, SeqCst);
    std::mem::take(&mut *SEEN.lock().unwrap_or_else(PoisonError::into_inner))
}

/// Taps the board key `id`.
fn tap(boot: &keyxtend::boot::Boot, id: &str) {
    let tap = FaceAction::Tap {
        id: KeyId::new(id),
        back: false,
    };
    let _ = boot.hub.act(tap);
}

/// The key page as the ui last heard it, once its typed key holds `n` dots.
fn typed(views: &Receiver<VoiceView>, n: u32) {
    loop {
        let view = views.recv_timeout(WAIT).unwrap();
        let sheet = view.0.panel.as_ref().map(|p| &p.sheet);
        if matches!(sheet, Some(VoiceSheet::Service(ServiceSheet { typed, .. })) if *typed.expose() == n)
        {
            return;
        }
    }
}

#[test]
fn a_key_typed_on_our_keyboard_into_its_line_is_sealed_and_shows_nowhere() {
    capture();
    let rig = Rig::new();
    let mut boot = rig.load();
    boot.start().unwrap();
    groq(&mut boot);
    let bus = boot.kernel.bus();
    let ((_views, views), (_lines, lines)) = (hear::<VoiceView>(&bus), hear::<VoiceLine>(&bus));
    key_page(&bus, &views);
    while lines.recv_timeout(WAIT).unwrap().0 != Some(PanelLine::Private) {}
    rig.injector.clear();
    let at = |i: usize| u16::try_from(i % LETTERS.len()).unwrap();
    for i in 0..TYPED {
        tap(&boot, &format!("c-0x{:04X}", FIRST + at(i)));
    }
    typed(&views, u32::try_from(TYPED).unwrap());
    assert!(rig.injector.strokes().is_empty(), "the app got none of it");
    tap(&boot, "enter");
    dots(&views, TYPED);
    let key: String = (0..TYPED)
        .map(|i| LETTERS.as_bytes()[usize::from(at(i))] as char)
        .collect();
    assert_eq!(sealed(&rig), Some(key.clone().into_bytes()), "sealed");
    unseen(&rig, &boot, &[], std::slice::from_ref(&key));
    boot.kernel.stop_all();
    assert!(
        logged().iter().all(|line| !line.contains(&key)),
        "no log line holds it"
    );
}
