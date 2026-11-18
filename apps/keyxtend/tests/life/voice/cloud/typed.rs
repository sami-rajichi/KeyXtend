//! A key typed on our keyboard into the key page's line (ADR-0049): the letters go to the voice module, not the app,
//! Enter seals the key, and no view, no text for the keyboard and no log line holds it.

use super::{dots, groq, key_page, sealed, unseen};
use crate::rig::{Rig, WAIT, hear};
use kx_module_api::ports::{
    FaceAction, KeyId, PanelLine, ServiceSheet, VoiceLine, VoiceSheet, VoiceView,
};
use std::fmt::{self, Write as _};
use std::sync::mpsc::Receiver;
use std::sync::{Mutex, Once};
use tracing::field::{Field, Visit};
use tracing::span::{Attributes, Id, Record};
use tracing::{Event, Metadata, Subscriber};

/// The board's letter keys from Q on, as the English layout types them.
const LETTERS: &str = "qwertyuiop";
/// The board's scan code of the first.
const FIRST: u16 = 0x10;
/// How many letters the test types.
const TYPED: usize = 24;

/// Every line logged on any thread since the capture was installed.
static SEEN: Mutex<Vec<String>> = Mutex::new(Vec::new());
/// Installs the capture once for the test binary.
static INSTALL: Once = Once::new();

/// Keeps every line logged anywhere, each as its fields' debug text.
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

    fn new_span(&self, _: &Attributes<'_>) -> Id {
        Id::from_u64(1)
    }

    fn record(&self, _: &Id, _: &Record<'_>) {}

    fn record_follows_from(&self, _: &Id, _: &Id) {}

    fn event(&self, event: &Event<'_>) {
        let mut line = Recorder::default();
        event.record(&mut line);
        SEEN.lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(line.0);
    }

    fn enter(&self, _: &Id) {}

    fn exit(&self, _: &Id) {}
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
    INSTALL.call_once(|| {
        assert!(
            tracing::subscriber::set_global_default(Lines).is_ok(),
            "the capture"
        );
    });
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
    let seen = SEEN
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    assert!(
        seen.iter().all(|line| !line.contains(&key)),
        "no log line holds it"
    );
}
