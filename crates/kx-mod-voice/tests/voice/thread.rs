//! The module on its own threads: the key opens a dictation and the light follows it, a text goes out on the typer's
//! thread and a held keyboard never holds the stop, a panic hides the bar, closes the microphone and the keyboard's
//! panel line, and with the feature off nothing starts (edge 36).

use std::any::Any;
use std::panic;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex, MutexGuard, Once};
use std::thread;

use kx_mod_voice::{VoiceModule, VoiceSettings};
use kx_module_api::ports::{
    Drive, FetchAsk, FetchNote, FetchRun, FieldKind, Fingerprint, FolderScan, LayoutChanged,
    LayoutId, ModelError, ModelFiles, PanelLine, SpeechDone, SpeechNote, SpeechStage, TextAsk,
    TextDone, VoiceBar, VoiceKey, VoiceLit, VoiceOpen,
};
use kx_module_api::{Bus, Redacted, Zeroizing, keys};
use kx_platform_fake::{FakeModelFiles, SpeechCall};
use kx_test_support::host::FakeHost;

use crate::rig::{Fakes, Heard, Said, WAIT, start, stop};

/// The English layout.
fn english() -> LayoutChanged {
    LayoutChanged {
        id: LayoutId::new(1),
        iso: "en".into(),
        name: "English".into(),
        rtl: false,
    }
}

/// Starts a dictation whose microphone opens, once the module heard English in front, and waits for the light.
fn listen(fakes: &Fakes, bus: &Bus, heard: &Heard) {
    bus.publish(&english()).unwrap();
    let open = vec![SpeechNote::Stage(SpeechStage::Open)];
    fakes.speech.on(SpeechCall::Open, open);
    bus.publish(&VoiceKey).unwrap();
    heard.wait_for(&Said::Lit(VoiceLit::Rec));
}

#[test]
fn the_key_opens_a_dictation_on_the_actors_thread_and_the_light_follows_it() {
    let fakes = Fakes::new();
    let mut host = fakes.host();
    let (heard, bus) = (Heard::listen(&host), Bus::new(Arc::clone(&host.core)));
    let mut module = VoiceModule::new();
    start(&mut module, &mut host).unwrap();
    listen(&fakes, &bus, &heard);
    assert_eq!(fakes.speech.runs().len(), 1);
    stop(&mut module, &mut host);
    assert!(
        fakes.speech.wait(WAIT, |runs| runs[0].closed),
        "the stop closes the microphone"
    );
    assert!(!fakes.field.watching() && !fakes.session.watching());
}

#[test]
fn a_text_goes_out_on_the_typers_thread_and_a_held_keyboard_never_holds_the_stop() {
    let fakes = Fakes::new();
    let mut host = fakes.host();
    let (heard, bus) = (Heard::listen(&host), Bus::new(Arc::clone(&host.core)));
    let (typing, held) = (mpsc::channel(), Arc::new(Mutex::new(())));
    let (told, gate) = (typing.0, Arc::clone(&held));
    let _keyboard = bus
        .subscribe(move |ask: &TextAsk| {
            let name = thread::current().name().map(str::to_owned);
            drop(told.send((ask.text.expose().to_string(), name)));
            drop(gate.lock());
        })
        .unwrap();
    let mut module = VoiceModule::new();
    start(&mut module, &mut host).unwrap();
    fakes.field.focus(FieldKind::Text);
    listen(&fakes, &bus, &heard);
    let shut = held.lock().unwrap();
    assert!(fakes.speech.tell(SpeechNote::Heard));
    fakes
        .clock
        .advance(VoiceSettings::from_defaults().unwrap().debounce());
    bus.publish(&VoiceKey).unwrap();
    assert!(fakes.speech.wait(WAIT, |runs| runs[0].stops == 1));
    let text = Redacted::new(Zeroizing::new("hello".to_owned()));
    assert!(fakes.speech.tell(SpeechNote::Piece { index: 0, text }));
    assert!(fakes.speech.tell(SpeechNote::Done(done())));
    let (text, name) = typing.1.recv_timeout(WAIT).unwrap();
    assert_eq!(
        (text.as_str(), name.as_deref()),
        ("hello ", Some("kx-voice-type"))
    );
    let stopped = stops_while_held(&mut module, &mut host, shut);
    assert!(stopped, "the stop waited on the keyboard");
}

/// Stops `module` on another thread while `held` keeps the keyboard typing; true when the stop ended within `WAIT`.
fn stops_while_held(
    module: &mut VoiceModule,
    host: &mut FakeHost,
    held: MutexGuard<'_, ()>,
) -> bool {
    thread::scope(|s| {
        let (finished, stopping) = mpsc::channel();
        s.spawn(move || {
            stop(module, host);
            let _ = finished.send(());
        });
        let stopped = stopping.recv_timeout(WAIT).is_ok();
        drop(held);
        stopped
    })
}

/// A run's end after one piece.
fn done() -> SpeechDone {
    SpeechDone {
        pieces: 1,
        sound_ms: 4_000,
        wait_ms: 900,
        load_ms: 0,
        memory_mb: 800,
        card: false,
    }
}

/// The payload of a panic the test plans; the panic hook prints nothing for it.
struct Planned;

/// Hushes planned panics for the whole test binary, once.
fn hush() {
    static HUSH: Once = Once::new();
    HUSH.call_once(|| {
        let usual = panic::take_hook();
        panic::set_hook(Box::new(move |info| {
            if !<dyn Any + Send>::is::<Planned>(info.payload()) {
                usual(info);
            }
        }));
    });
}

/// Model files whose drives panic once armed, as a platform bug on the actor's thread would.
struct Armed {
    real: FakeModelFiles,
    armed: Arc<AtomicBool>,
}

impl ModelFiles for Armed {
    fn drives(&self) -> Vec<Drive> {
        if self.armed.load(Ordering::SeqCst) {
            panic::panic_any(Planned);
        }
        self.real.drives()
    }

    fn make(&self, folder: &Path) -> Result<(), ModelError> {
        self.real.make(folder)
    }

    fn scan(&self, folder: &Path, exts: &[String]) -> Result<FolderScan, ModelError> {
        self.real.scan(folder, exts)
    }

    fn fingerprint(&self, file: &Path) -> Result<Fingerprint, ModelError> {
        self.real.fingerprint(file)
    }

    fn fetch(
        &self,
        ask: FetchAsk,
        tell: Box<dyn Fn(FetchNote) + Send>,
    ) -> Result<Box<dyn FetchRun>, ModelError> {
        self.real.fetch(ask, tell)
    }

    fn remove(&self, file: &Path) -> Result<(), ModelError> {
        self.real.remove(file)
    }
}

#[test]
fn a_panic_on_the_actors_thread_hides_the_bar_closes_the_microphone_and_tells_the_user() {
    hush();
    let (fakes, armed) = (Fakes::new(), Arc::new(AtomicBool::new(false)));
    let files = Armed {
        real: fakes.files.clone(),
        armed: Arc::clone(&armed),
    };
    let mut host = fakes.host_with(Arc::new(files));
    let (heard, bus) = (Heard::listen(&host), Bus::new(Arc::clone(&host.core)));
    let mut module = VoiceModule::new();
    start(&mut module, &mut host).unwrap();
    bus.publish(&VoiceOpen).unwrap();
    heard.wait_for(&Said::Line(Some(PanelLine::None)));
    listen(&fakes, &bus, &heard);
    armed.store(true, Ordering::SeqCst);
    bus.publish(&VoiceOpen).unwrap();
    heard.wait(|s| matches!(s, Said::View(page) if page.bar == VoiceBar::Hidden).then_some(()));
    heard.wait_for(&Said::Lit(VoiceLit::Off));
    heard.wait_for(&Said::Line(None));
    heard.wait_for(&Said::Notice(keys::MODULE_FAILED));
    assert!(
        fakes
            .speech
            .wait(WAIT, |runs| runs[0].closed && !runs[0].held)
    );
    stop(&mut module, &mut host);
}

#[test]
fn with_the_feature_off_no_bar_shows_no_key_lights_and_nothing_starts() {
    let fakes = Fakes::new();
    let mut host = fakes.host();
    host.put("on", false.into());
    let (heard, bus) = (Heard::listen(&host), Bus::new(Arc::clone(&host.core)));
    let mut module = VoiceModule::new();
    start(&mut module, &mut host).unwrap();
    let told = heard.drain();
    assert!(
        matches!(&told[..], [Said::View(page), Said::Lit(VoiceLit::Off), Said::Line(None)] if page.bar == VoiceBar::Hidden && page.panel.is_none())
    );
    assert_eq!(host.lookups.get(), 0, "no service, so no worker");
    bus.publish(&VoiceKey).unwrap();
    bus.publish(&TextDone {
        round: 1,
        count: 0,
        ok: true,
    })
    .unwrap();
    stop(&mut module, &mut host);
    assert!(heard.drain().is_empty() && fakes.speech.runs().is_empty());
    assert!(fakes.speech.device_asks().is_empty() && !fakes.field.watching());
}
