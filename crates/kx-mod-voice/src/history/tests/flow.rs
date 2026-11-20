//! History fed by the real dictation flow, carried out as the actor will: a password box, a hidden run and history
//! off keep nothing, and Try again replaces the text by its take while its one recording stays.

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use kx_module_api::ports::{
    FieldChange, FieldKind, Fingerprint, LayoutChanged, LayoutId, MAX_BLOB_BYTES, PanelButton,
    SpeechDone, SpeechModel, SpeechNote, SpeechStage, VoiceAct, VoiceClick,
};
use kx_module_api::{Clock, Mono, SecretBytes, US_PER_S};

use super::*;
use crate::flow::{Flow, In, Lack, Out, Ready, RunCall};
use crate::store::Store;
use crate::store::mem::Mem;

/// The box the tests type into.
const BOX: u64 = 7;
/// The row the tests dictate with.
const MODEL: &str = "parakeet-v3-q8";
/// The recording's bytes.
const SOUND: usize = 4_096;

/// A clock the test moves; it never sleeps.
#[derive(Default)]
struct TestClock(AtomicU64);

impl Clock for TestClock {
    fn now(&self) -> Mono {
        Mono::from_us(self.0.load(Ordering::SeqCst))
    }
}

/// Parakeet serves every language.
#[allow(
    clippy::unnecessary_wraps,
    reason = "it is the flow's `Supply`, which may answer a lack"
)]
fn supply(iso: &str) -> Result<Ready, Lack> {
    let model = SpeechModel {
        file: Redacted::new(PathBuf::from(r"D:\models\parakeet.gguf")),
        lang: iso.into(),
        runner: "transcribe".into(),
        cpu_mb: 900,
    };
    let name = Redacted::new(MODEL.into());
    Ok(Ready {
        model,
        card: None,
        why: None,
        name,
        print: Fingerprint::new([1; Fingerprint::BYTES]),
        bytes: 1,
    })
}

/// The recording a run hands over.
fn recording() -> Vec<SecretBytes> {
    vec![Redacted::new(Zeroizing::new(vec![3; SOUND]))]
}

/// Settings with Keep recordings on.
fn keeping() -> VoiceSettings {
    let mut settings = VoiceSettings::from_defaults().unwrap();
    settings.keep_voice = true;
    settings
}

/// The flow with the history and store the actor holds.
struct Rig {
    flow: Flow,
    clock: Arc<TestClock>,
    take: u32,
    round: u32,
    history: History,
    store: Store,
    shelf: Arc<Mem>,
    rules: Rules,
    /// The flow asked the run for its sound.
    asked: bool,
}

impl Rig {
    fn new(settings: VoiceSettings, hidden: bool, kind: FieldKind) -> Self {
        let clock = Arc::new(TestClock::default());
        let shelf = Arc::new(Mem::default());
        let opened = Store::open(shelf.clone(), MAX_BLOB_BYTES).unwrap();
        let rules = Rules::of(&settings, hidden);
        let flow = Flow::new(clock.clone(), settings, hidden);
        let (history, store) = (opened.history, opened.store);
        let mut rig = Self {
            flow,
            clock,
            take: 0,
            round: 0,
            history,
            store,
            shelf,
            rules,
            asked: false,
        };
        let (id, iso, name) = (LayoutId::new(1), "fr".into(), "Français".into());
        rig.on(In::Layout(LayoutChanged {
            id,
            iso,
            name,
            rtl: false,
        }));
        rig.on(In::Field(FieldChange { kind, box_id: BOX }));
        rig
    }

    /// Feeds `input` and carries out the History effects as the actor will.
    fn on(&mut self, input: In) {
        for out in self.flow.on(input, &supply) {
            match out {
                Out::Open { take, .. } => self.take = take,
                Out::View(view) => self.round = view.0.round,
                Out::Keep(kept) => {
                    let edit = self.history.keep(kept, Stamp::default(), &self.rules);
                    self.store.apply(&self.history, edit).unwrap();
                }
                Out::Run {
                    take,
                    call: RunCall::Keep,
                } => {
                    self.asked = true;
                    let (h, r) = (&mut self.history, &self.rules);
                    self.store.keep_sound(h, take, &recording(), r).unwrap();
                }
                _ => {}
            }
        }
    }

    fn tell(&mut self, note: SpeechNote) {
        let take = self.take;
        self.on(In::Note { take, note });
    }

    /// The run writes `text`, then ends.
    fn write(&mut self, text: &str) {
        let text = secret(text);
        self.tell(SpeechNote::Piece { index: 0, text });
        let done = SpeechDone {
            pieces: 1,
            sound_ms: 2_000,
            wait_ms: 500,
            load_ms: 0,
            memory_mb: 800,
            card: false,
        };
        self.tell(SpeechNote::Done(done));
    }

    /// Starts, speaks a second, stops and writes `text`.
    fn dictate(&mut self, text: &str) {
        self.on(In::Key);
        self.tell(SpeechNote::Stage(SpeechStage::Open));
        self.tell(SpeechNote::Heard);
        self.clock
            .0
            .fetch_add(u64::from(US_PER_S), Ordering::SeqCst);
        self.on(In::Key);
        self.tell(SpeechNote::Stage(SpeechStage::Shut));
        self.write(text);
    }

    /// Try again, which writes `text`.
    fn again(&mut self, text: &str) {
        let (round, click) = (self.round, VoiceClick::Button(PanelButton::Again));
        self.on(In::Act(VoiceAct { round, click }));
        self.write(text);
    }
}

#[test]
fn a_dictation_is_kept_with_its_recording() {
    let mut rig = Rig::new(keeping(), false, FieldKind::Text);
    rig.dictate("bonjour");
    assert_eq!(texts(&rig.history), ["bonjour"]);
    let entry = &rig.history.entries()[0];
    assert_eq!(
        (entry.lang.as_str(), entry.model.expose().as_str()),
        ("fr", MODEL)
    );
    let blobs = rig.store.sound(entry).unwrap().unwrap();
    let bytes: Vec<u8> = blobs.iter().flat_map(|b| b.expose().to_vec()).collect();
    assert_eq!(bytes, recording()[0].expose().to_vec());
}

#[test]
fn try_again_replaces_the_text_and_keeps_its_one_recording() {
    let mut rig = Rig::new(keeping(), false, FieldKind::Text);
    rig.dictate("bonjour");
    let first = rig.history.entries()[0].clone();
    assert!(first.sound.is_some());
    rig.again("bonsoir");
    assert_eq!(texts(&rig.history), ["bonsoir"]);
    let now = &rig.history.entries()[0];
    assert_eq!((now.id, now.sound), (first.id, first.sound));
    let back = Store::open(rig.shelf.clone(), MAX_BLOB_BYTES).unwrap();
    assert_eq!(back.history.entries(), rig.history.entries());
}

#[test]
fn a_password_box_keeps_nothing() {
    let mut rig = Rig::new(keeping(), false, FieldKind::Password);
    rig.dictate("hunter2");
    assert!(rig.history.is_empty() && rig.shelf.is_empty() && !rig.asked);
}

#[test]
fn a_hidden_run_keeps_nothing() {
    let mut rig = Rig::new(keeping(), true, FieldKind::Text);
    rig.dictate("bonjour");
    assert!(rig.history.is_empty() && rig.shelf.is_empty() && !rig.asked);
}

#[test]
fn history_off_keeps_nothing_even_with_keep_recordings_on() {
    let mut settings = keeping();
    settings.history = false;
    let mut rig = Rig::new(settings, false, FieldKind::Text);
    rig.dictate("bonjour");
    assert!(rig.history.is_empty() && rig.shelf.is_empty());
    assert!(!rig.asked, "the sound stays in the worker");
}
