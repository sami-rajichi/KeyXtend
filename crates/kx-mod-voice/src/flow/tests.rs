//! The flow on a test clock (plan task 17): each file covers some of the plan's edge cases 1 to 14 and 16 to 18.

use super::*;
use kx_module_api::ports::{
    BarButton, BarHint, FieldChange, FieldKind, LayoutChanged, LayoutId, PanelButton, SpeechError,
    SpeechNote, SpeechStage, TextAsk, TextDone, VoiceAct, VoiceBar, VoiceClick, VoiceLang,
    VoiceLit, VoicePage, VoiceText,
};
use kx_module_api::ports::{Fingerprint, SpeechAsk, SpeechDone, SpeechModel};
use kx_module_api::{Redacted, Secret, Zeroizing};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use outs::{ask, calls, kept, note, opened, text};

mod cancel;
mod close;
mod cloud;
mod copied;
mod faults;
mod gear;
mod lend;
mod listen;
mod outs;
mod paste;
mod prep;
mod props;
mod session;
mod start;
mod text;
mod undo;

/// The size Download fetches in the tests, in bytes.
const BYTES: u64 = 739_508_576;
/// The box the tests type into.
const BOX: u64 = 7;
/// The model file's fingerprint in the tests.
const PRINT: Fingerprint = Fingerprint::new([7; Fingerprint::BYTES]);

/// A clock the test moves; it never sleeps.
#[derive(Default)]
struct TestClock(AtomicU64);

impl TestClock {
    fn advance(&self, by: Duration) {
        let us = u64::try_from(by.as_micros()).unwrap();
        self.0.fetch_add(us, Ordering::SeqCst);
    }
}

impl Clock for TestClock {
    fn now(&self) -> Mono {
        Mono::from_us(self.0.load(Ordering::SeqCst))
    }
}

/// What each language gets: its local model, then its cloud pick, if any.
struct Book(BTreeMap<String, Result<Ready, Lack>>, BTreeMap<String, Net>);

impl Supply for Book {
    fn supply(&self, iso: &str) -> Result<Ready, Lack> {
        self.0.get(iso).cloned().unwrap_or(Err(Lack::Unknown))
    }

    fn cloud(&self, iso: &str) -> Option<Net> {
        self.1.get(iso).cloned()
    }
}

/// A model named `name` that writes `code`.
fn ready(name: &str, code: &str) -> Ready {
    let file = PathBuf::from(format!(r"D:\models\{name}.gguf"));
    let model = SpeechModel {
        file: Redacted::new(file),
        lang: code.into(),
        runner: "transcribe".into(),
        cpu_mb: 900,
    };
    Ready {
        model,
        card: None,
        why: None,
        name: Redacted::new(name.into()),
        print: PRINT,
        bytes: BYTES,
    }
}

/// The layout of `iso`, named `name`.
fn layout(iso: &str, name: &str, rtl: bool) -> LayoutChanged {
    LayoutChanged {
        id: LayoutId::new(1),
        iso: iso.into(),
        name: name.into(),
        rtl,
    }
}

/// French as the bar names it.
fn french() -> VoiceLang {
    VoiceLang {
        name: "Français".into(),
        iso: "fr".into(),
        rtl: false,
        arabic: false,
    }
}

fn secret(text: &str) -> Secret {
    Redacted::new(Zeroizing::new(text.to_owned()))
}

/// `text` as the keyboard types it, with the default suffix.
fn suffixed(text: &str) -> String {
    let settings = VoiceSettings::from_defaults().unwrap();
    format!("{text}{}", settings.type_suffix)
}

/// A run's end after `pieces` pieces.
fn done(pieces: u16) -> SpeechDone {
    SpeechDone {
        pieces,
        sound_ms: 4_000,
        wait_ms: 900,
        load_ms: 0,
        memory_mb: 800,
        card: false,
    }
}

/// A flow with French shown, a model for French and English, none downloaded for Arabic, and a text box in front.
struct Rig {
    flow: Flow,
    clock: Arc<TestClock>,
    book: Book,
    take: u32,
    /// The last upload's take and round, which `sent` answers.
    upload: Option<(u32, u32)>,
    page: Option<VoicePage>,
    /// The light told last.
    light: VoiceLit,
    /// The light is red.
    lit: bool,
}

impl Rig {
    fn new() -> Self {
        Self::with(VoiceSettings::from_defaults().unwrap(), false)
    }

    fn with(settings: VoiceSettings, hidden: bool) -> Self {
        let clock = Arc::new(TestClock::default());
        let flow = Flow::new(clock.clone(), settings, hidden);
        let mut book = BTreeMap::new();
        book.insert("fr".into(), Ok(ready("parakeet", "fr")));
        book.insert("en".into(), Ok(ready("parakeet", "en")));
        book.insert("ar".into(), Err(Lack::Missing { bytes: BYTES }));
        let mut rig = Self {
            flow,
            clock,
            book: Book(book, BTreeMap::new()),
            take: 0,
            upload: None,
            page: None,
            light: VoiceLit::Off,
            lit: false,
        };
        rig.on(In::Layout(layout("fr", "Français", false)));
        rig.field(FieldKind::Text, BOX);
        rig
    }

    /// Feeds `input`, noting the take opened, the view and the light told.
    fn on(&mut self, input: In) -> Vec<Out> {
        let out = self.flow.on(input, &self.book);
        for effect in &out {
            match effect {
                Out::Open { take, .. } => self.take = *take,
                Out::Send(up) => self.upload = Some((up.take, up.round)),
                Out::View(view) => self.page = Some((*view.0).clone()),
                Out::Lit(light) => (self.light, self.lit) = (*light, *light == VoiceLit::Rec),
                _ => {}
            }
        }
        out
    }

    fn key(&mut self) -> Vec<Out> {
        self.on(In::Key)
    }

    fn wait(&self, ms: u64) {
        self.clock.advance(Duration::from_millis(ms));
    }

    /// Waits `ms`, then ticks.
    fn tick_after(&mut self, ms: u64) -> Vec<Out> {
        self.wait(ms);
        self.on(In::Tick)
    }

    /// Tells the open take's run `note`.
    fn tell(&mut self, note: SpeechNote) -> Vec<Out> {
        let take = self.take;
        self.on(In::Note { take, note })
    }

    /// The microphone opens and speech is heard.
    fn speak(&mut self) {
        self.tell(SpeechNote::Stage(SpeechStage::Open));
        self.tell(SpeechNote::Heard);
    }

    /// Writes `texts` as pieces, then ends the run.
    fn write(&mut self, texts: &[&str]) -> Vec<Out> {
        let mut index = 0;
        for text in texts {
            let text = secret(text);
            self.tell(SpeechNote::Piece { index, text });
            index += 1;
        }
        self.tell(SpeechNote::Done(done(index)))
    }

    /// Starts, speaks, stops and writes `text`.
    fn dictate(&mut self, text: &str) -> Vec<Out> {
        self.key();
        self.speak();
        self.wait(1_000);
        self.key();
        self.tell(SpeechNote::Stage(SpeechStage::Shut));
        self.write(&[text])
    }

    /// Clicks `button` on the bar shown.
    fn bar(&mut self, button: BarButton) -> Vec<Out> {
        self.click(VoiceClick::Bar(button))
    }

    fn click(&mut self, click: VoiceClick) -> Vec<Out> {
        let round = self.round();
        self.on(In::Act(VoiceAct { round, click }))
    }

    fn again(&mut self) -> Vec<Out> {
        self.click(VoiceClick::Button(PanelButton::Again))
    }

    /// The gear or the tray: the panel's open reaches the flow, then the panel shows.
    fn gear(&mut self) -> Vec<Out> {
        let mut out = self.on(In::Opening);
        out.extend(self.on(In::Panel(Some(Box::new(close::panel())))));
        out
    }

    fn field(&mut self, kind: FieldKind, box_id: u64) -> Vec<Out> {
        self.on(In::Field(FieldChange { kind, box_id }))
    }

    /// The keyboard typed `count` characters of the last ask.
    fn typed(&mut self, ask: &TextAsk, count: u32) -> Vec<Out> {
        let round = ask.round;
        self.on(In::Typed(TextDone {
            round,
            count,
            ok: true,
        }))
    }

    fn round(&self) -> u32 {
        self.page.as_ref().map_or(0, |p| p.round)
    }

    fn shown(&self) -> VoiceBar {
        self.page
            .as_ref()
            .map_or(VoiceBar::Hidden, |p| p.bar.clone())
    }
}
