//! A rig for the actor: the shared fake host's bus and settings handle, the fakes of the box, the packs, the sealed
//! store, the clock and the day, and every event the actor publishes. The test hands each message to the actor, as its
//! thread would, so no test needs a thread or a sleep.

use kx_mod_predict::{Actor, Days, Flow, MANIFEST, Msg, Pack, Ports, PredictSettings};
use kx_module_api::ports::{
    ChipKind, ChipPicked, FieldKind, Lane, PackView, Piece, Suggested, WordsAsk,
};
use kx_module_api::{Bus, ModuleCx, Mono, Redacted, Subscription};
use kx_platform_fake::{FakeClock, FakeFieldWatch, FakeSealedStore, FakeWordPacks};
use kx_test_support::host::FakeHost;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// When each test starts.
pub const T0: Mono = Mono::from_us(1_000_000);
/// Each test's first day number.
pub const DAY: u32 = 20_000;

mod heard;
mod stored;
pub use heard::Heard;
use heard::listen;
pub use stored::{blob, learner, plant, saved};

/// A day the test moves.
struct Today(Arc<AtomicU32>);

impl Days for Today {
    fn today(&self) -> u32 {
        self.0.load(Ordering::SeqCst)
    }
}

/// The fakes behind the actor's ports.
#[derive(Clone)]
pub struct Fakes {
    pub field: FakeFieldWatch,
    pub packs: FakeWordPacks,
    pub store: FakeSealedStore,
    pub clock: FakeClock,
    pub day: Arc<AtomicU32>,
}

/// The actor on a fake host, its fakes, and what it published.
pub struct Rig {
    pub fakes: Fakes,
    pub host: FakeHost,
    pub settings: PredictSettings,
    actor: Actor,
    tells: Receiver<Msg>,
    view: Option<Box<dyn PackView>>,
    heard: Arc<Mutex<Vec<Heard>>>,
    _ears: Vec<Subscription>,
}

impl Rig {
    /// A begun rig on the default settings, focus on no box.
    pub fn new() -> Self {
        Self::build(|_, _| {})
    }

    /// A begun rig whose fakes and settings `setup` prepares first; a text box confirms every pick unless it says
    /// otherwise.
    pub fn build(setup: impl FnOnce(&Fakes, &mut FakeHost)) -> Self {
        let fakes = Fakes {
            field: FakeFieldWatch::new(),
            packs: FakeWordPacks::new(),
            store: FakeSealedStore::new(),
            clock: FakeClock::new(T0),
            day: Arc::new(AtomicU32::new(DAY)),
        };
        fakes.field.set_ends(Some(true));
        let mut host = FakeHost::new(&MANIFEST);
        host.granted = MANIFEST.capabilities.to_vec();
        setup(&fakes, &mut host);
        let settings: PredictSettings = ModuleCx::new(&mut host).settings().unwrap();
        let (tx, tells) = mpsc::channel();
        let field = kx_mod_predict::watch(&fakes.field, settings.field_config(), &tx);
        let ports = Ports {
            bus: ModuleCx::new(&mut host).bus(),
            clock: Arc::new(fakes.clock.clone()),
            days: Arc::new(Today(Arc::clone(&fakes.day))),
            packs: Arc::new(fakes.packs.clone()),
            store: Arc::new(fakes.store.clone()),
            handle: host.handle(),
            field,
        };
        let heard = Arc::default();
        let ears = listen(&Bus::new(Arc::clone(&host.core)), &heard);
        let actor = Actor::new(ports, &settings);
        let mut rig = Self {
            fakes,
            host,
            settings,
            actor,
            tells,
            view: None,
            heard,
            _ears: ears,
        };
        rig.actor.begin();
        rig.arrive();
        rig
    }

    /// Opens the language in front's pack, as the thread does after a switch: the old one closes first.
    fn arrive(&mut self) {
        self.view = None;
        self.view = self.actor.open();
        let pack = self.actor.parse(self.view.as_deref());
        self.actor.arrive(pack.as_ref());
    }

    /// Hands `msg` to the actor, then each focus change the box told, ticking after each as the thread does.
    pub fn send(&mut self, msg: Msg) -> Flow {
        let flow = self.step(msg);
        self.told();
        flow
    }

    /// One message and its tick; a switch opens the new language.
    fn step(&mut self, msg: Msg) -> Flow {
        let pack = parse(self.view.as_deref());
        let flow = self.actor.handle(pack.as_ref(), msg);
        drop(pack);
        match flow {
            Flow::Switch => self.arrive(),
            Flow::Go => self.tick(),
            Flow::Quit => {}
        }
        flow
    }

    /// Runs the actor's timers now, as a wake of its thread would.
    pub fn tick(&mut self) {
        self.actor.tick();
    }

    /// How long the thread would wait now before its next tick.
    pub fn wait(&self) -> Option<Duration> {
        self.actor.wait()
    }

    /// The keyboard now shows language `iso`, named by its code.
    pub fn lang(&mut self, iso: &str) -> Flow {
        self.named(iso, iso)
    }

    /// The keyboard now shows language `iso`, which the OS names `name`.
    pub fn named(&mut self, iso: &str, name: &str) -> Flow {
        let rtl = false;
        self.send(Msg::Lang {
            iso: iso.to_owned(),
            name: name.to_owned(),
            rtl,
        })
    }

    /// Focus moves to a new box of `kind`.
    pub fn focus(&mut self, kind: FieldKind) {
        self.fakes.field.focus(kind);
        self.told();
    }

    /// Hands the actor each focus change the box told and it has not had yet.
    pub fn told(&mut self) {
        while let Ok(told) = self.tells.try_recv() {
            self.step(told);
        }
    }

    /// Hands `msg` to the actor ahead of the focus changes the box told, as when their event lands late.
    pub fn late(&mut self, msg: Msg) {
        self.step(msg);
    }

    /// The keyboard typed `text`.
    pub fn typed(&mut self, text: &str) {
        self.send(Msg::Typed(Piece::Text(Redacted::new(text.to_owned()))));
    }

    /// A tap on the menu, the list or a control chip.
    pub fn ask(&mut self, ask: WordsAsk) {
        self.send(Msg::Ask(ask));
    }

    /// A tap on the chip at `index` of the last lane.
    pub fn pick(&mut self, index: u8) {
        let round = self.lane().round;
        self.send(Msg::Picked(ChipPicked { round, index }));
    }

    /// The module stops.
    pub fn stop(&mut self) {
        self.send(Msg::Quit);
    }

    /// Moves the clock on by `by`.
    pub fn advance(&self, by: Duration) {
        self.fakes.clock.advance(by);
    }

    /// Makes today `day`.
    pub fn set_day(&self, day: u32) {
        self.fakes.day.store(day, Ordering::SeqCst);
    }

    /// Everything published since the last take.
    pub fn take(&self) -> Vec<Heard> {
        std::mem::take(&mut *self.heard.lock().unwrap())
    }

    /// Everything published so far, kept.
    pub fn all(&self) -> Vec<Heard> {
        self.heard.lock().unwrap().clone()
    }

    /// The last lane published.
    pub fn lane(&self) -> Suggested {
        let lanes = self.all().into_iter().rev().find_map(|h| match h {
            Heard::Lane(s) => Some(s),
            _ => None,
        });
        lanes.unwrap()
    }

    /// The last lane's chips: each text and kind.
    pub fn chips(&self) -> Vec<(String, ChipKind)> {
        match self.lane().lane {
            Lane::Chips(chips) => chips
                .into_iter()
                .map(|c| (c.text.into_inner(), c.kind))
                .collect(),
            _ => Vec::new(),
        }
    }

    /// The last lane's word chips' texts.
    pub fn words(&self) -> Vec<String> {
        let words = self.chips().into_iter();
        words
            .filter(|c| c.1 == ChipKind::Word)
            .map(|c| c.0)
            .collect()
    }

    /// Every word the actor asked the keyboard to type: its erase and text.
    pub fn typed_words(&self) -> Vec<(u16, String)> {
        let words = self.all().into_iter().filter_map(|h| match h {
            Heard::Word(w) => Some((w.erase, w.text.into_inner())),
            _ => None,
        });
        words.collect()
    }

    /// Every notice key published.
    pub fn notices(&self) -> Vec<&'static str> {
        let keys = self.all().into_iter().filter_map(|h| match h {
            Heard::Notice(n) => Some(n.key),
            _ => None,
        });
        keys.collect()
    }

    /// Every `PressesWanted` published.
    pub fn presses(&self) -> Vec<bool> {
        let told = self.all().into_iter().filter_map(|h| match h {
            Heard::Presses(on) => Some(on),
            _ => None,
        });
        told.collect()
    }
}

/// The open pack, read again for each message; the thread reads it once per switch.
fn parse(view: Option<&dyn PackView>) -> Option<Pack<'_>> {
    view.and_then(|v| Pack::parse(v.bytes()).ok())
}
