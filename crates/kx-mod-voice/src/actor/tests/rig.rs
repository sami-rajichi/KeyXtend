//! A rig for the actor: the fake host's bus and settings, every port's fake, a models folder holding English's default
//! model, and what `taps` heard. The test hands each message to the actor as its thread would and jobs run in place, so
//! no test needs a thread or a sleep.

use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use kx_module_api::ports::{
    FieldKind, FieldWatch, PasteText, SessionWatch, SpeechNote, SpeechStage, TextAsk, TextDone,
    TextPasted, VoiceAct, VoiceBar, VoiceClick, VoiceLit, VoicePage,
};
use kx_module_api::{Bus, ModuleCx, Redacted, Subscription, Zeroizing};
use kx_platform_fake::SpeechCall;
use kx_test_support::host::FakeHost;

use super::super::{Actor, Msg, Ports};
pub(super) use super::fakes::{DIR, Fakes, done, english, english_layout};
use super::taps::{Heard, in_place, listen, typist};
use crate::module::read;
use crate::{MANIFEST, VoiceSettings};

/// The actor on a fake host, its fakes, and what it published.
pub(super) struct Rig {
    pub fake: Fakes,
    pub host: FakeHost,
    pub actor: Actor,
    inbox: Receiver<Msg>,
    asks: Arc<Mutex<Vec<TextAsk>>>,
    heard: Arc<Mutex<Vec<Heard>>>,
    _ears: Vec<Subscription>,
}

impl Rig {
    /// A begun rig on the defaults with the models folder set, a text box focused and English in front.
    pub fn new() -> Self {
        Self::build(|_, _| {})
    }

    /// A begun rig whose fakes and settings `setup` prepares first, then a text box focused and English in front.
    pub fn build(setup: impl FnOnce(&Fakes, &mut FakeHost)) -> Self {
        Self::begun(Self::unbegun(setup))
    }

    /// A begun rig of a run whose saves would be hidden (ADR-0037).
    pub fn hidden() -> Self {
        Self::hidden_with(|_, _| {})
    }

    /// A begun rig of a hidden run whose fakes and settings `setup` prepares first.
    pub fn hidden_with(setup: impl FnOnce(&Fakes, &mut FakeHost)) -> Self {
        Self::begun(Self::unbegun_as(true, setup))
    }

    /// `rig` begun, then a text box focused and English in front.
    fn begun(mut rig: Self) -> Self {
        rig.actor.begin();
        rig.pump();
        rig.fake.field.focus(FieldKind::Text);
        rig.pump();
        rig.send(Msg::Layout(english_layout()));
        rig
    }

    /// The rig before `begin`, its settings and fakes prepared by `setup`.
    pub fn unbegun(setup: impl FnOnce(&Fakes, &mut FakeHost)) -> Self {
        Self::unbegun_as(false, setup)
    }

    /// A begun rig on `section`, read and repaired as the module's own start does, with `fake` behind its ports.
    pub fn restarted(fake: Fakes, section: toml::Table) -> Self {
        let setup = |_: &Fakes, host: &mut FakeHost| {
            for (key, value) in section {
                host.put(&key, value);
            }
        };
        let rig = Self::unbegun_from(false, fake, setup, |cx| read(cx).unwrap());
        Self::begun(rig)
    }

    /// The rig before `begin`, of a hidden run when `hidden`, its settings and fakes prepared by `setup`.
    fn unbegun_as(hidden: bool, setup: impl FnOnce(&Fakes, &mut FakeHost)) -> Self {
        Self::unbegun_from(hidden, Fakes::new(), setup, |cx| cx.settings().unwrap())
    }

    /// The rig before `begin` on `fake`, prepared by `setup`, its settings as `read` takes them from the host.
    fn unbegun_from(
        hidden: bool,
        fake: Fakes,
        setup: impl FnOnce(&Fakes, &mut FakeHost),
        read: impl FnOnce(&ModuleCx<'_>) -> VoiceSettings,
    ) -> Self {
        let mut host = FakeHost::new(&MANIFEST);
        host.granted = MANIFEST.capabilities.to_vec();
        host.put("models_dir", DIR.into());
        setup(&fake, &mut host);
        let settings = read(&ModuleCx::new(&mut host));
        let (tx, inbox) = mpsc::channel();
        let (asks, heard) = (Arc::default(), Arc::default());
        let ears = listen(&Bus::new(Arc::clone(&host.core)), &heard);
        let ports = ports(&fake, &mut host, &settings, tx, &asks);
        Self {
            actor: Actor::new(ports, settings, hidden),
            fake,
            host,
            inbox,
            asks,
            heard,
            _ears: ears,
        }
    }

    /// Hands each message a port or a job sent to the actor, as its thread does, until none is left.
    pub fn pump(&mut self) {
        while let Ok(msg) = self.inbox.try_recv() {
            self.actor.handle(msg);
            self.actor.tick();
        }
    }

    /// Hands `msg` to the actor and ticks, as the thread does, then what it led to.
    pub fn send(&mut self, msg: Msg) {
        self.actor.handle(msg);
        self.actor.tick();
        self.pump();
    }

    /// Clicks the Dictation key, past the last click's debounce.
    pub fn key(&mut self) {
        let debounce = self.actor.settings().debounce();
        self.fake.clock.advance(debounce);
        self.send(Msg::Key);
    }

    /// Clicks `click` on the view told last.
    pub fn act(&mut self, click: VoiceClick) {
        let round = self.views().last().map_or(0, |v| v.round);
        self.send(Msg::Act(VoiceAct { round, click }));
    }

    /// Starts a dictation whose microphone opens, and speech is heard.
    pub fn listen(&mut self) {
        let open = vec![SpeechNote::Stage(SpeechStage::Open)];
        self.fake.speech.on(SpeechCall::Open, open);
        self.key();
        self.tell(SpeechNote::Heard);
    }

    /// Dictates `text`: it listens, the key stops it, and the run writes the text in one piece.
    pub fn speak(&mut self, text: &str) {
        self.listen();
        self.key();
        let text = Redacted::new(Zeroizing::new(text.to_owned()));
        self.tell(SpeechNote::Piece { index: 0, text });
        self.tell(SpeechNote::Done(done(1)));
    }

    /// The newest run tells `note`, then what it led to.
    pub fn tell(&mut self, note: SpeechNote) {
        assert!(self.fake.speech.tell(note), "a live run heard it");
        self.pump();
    }

    /// The keyboard answers the last text asked, typed in full.
    pub fn typed(&mut self) {
        let ask = self.asks().last().cloned().unwrap();
        let count = u32::try_from(ask.text.expose().chars().count()).unwrap();
        let done = TextDone {
            round: ask.round,
            count,
            ok: true,
        };
        self.send(Msg::Typed(done));
    }

    /// Moves the clock on by `by` and ticks.
    pub fn advance(&mut self, by: Duration) {
        self.fake.clock.advance(by);
        self.actor.tick();
        self.pump();
    }

    /// Everything published so far.
    pub fn heard(&self) -> Vec<Heard> {
        self.heard.lock().unwrap().clone()
    }

    /// What `pick` takes of everything published, in order.
    fn picked<T>(&self, pick: impl Fn(Heard) -> Option<T>) -> Vec<T> {
        self.heard().into_iter().filter_map(pick).collect()
    }

    /// Every view told, in order.
    pub fn views(&self) -> Vec<VoicePage> {
        self.picked(|h| match h {
            Heard::View(page) => Some(page),
            _ => None,
        })
    }

    /// The bar of the last view.
    pub fn bar(&self) -> VoiceBar {
        self.views()
            .last()
            .map_or(VoiceBar::Hidden, |v| v.bar.clone())
    }

    /// Every light told, in order.
    pub fn lits(&self) -> Vec<VoiceLit> {
        self.picked(|h| match h {
            Heard::Lit(on) => Some(on),
            _ => None,
        })
    }

    /// Every wish for presses told, in order.
    pub fn presses(&self) -> Vec<bool> {
        self.picked(|h| match h {
            Heard::Presses(on) => Some(on),
            _ => None,
        })
    }

    /// The key of every notice told, in order.
    pub fn notices(&self) -> Vec<&'static str> {
        self.picked(|h| match h {
            Heard::Notice(key) => Some(key),
            _ => None,
        })
    }

    /// Every text handed to the typer, in order.
    pub fn asks(&self) -> Vec<TextAsk> {
        self.asks.lock().unwrap().clone()
    }

    /// Every text handed to the clipboard history to paste, in order.
    pub fn pastes(&self) -> Vec<PasteText> {
        self.picked(|h| match h {
            Heard::Paste(ask) => Some(ask),
            _ => None,
        })
    }

    /// The clipboard history answers the last paste asked.
    pub fn pasted(&mut self, ok: bool) {
        let round = self.pastes().last().unwrap().round;
        self.send(Msg::Pasted(TextPasted { round, ok }));
    }

    /// The settings as the module's own writes left them.
    pub fn saved(&self) -> VoiceSettings {
        self.host.sink.read_as().unwrap()
    }
}

/// The ports over `fake`, the watches started with their tells into `tx`, and a typer keeping its texts in `asks`.
fn ports(
    fake: &Fakes,
    host: &mut FakeHost,
    settings: &VoiceSettings,
    tx: Sender<Msg>,
    asks: &Arc<Mutex<Vec<TextAsk>>>,
) -> Ports {
    let (to_field, to_session) = (tx.clone(), tx.clone());
    let field = fake.field.watch(
        settings.field_config(),
        Box::new(move |c| drop(to_field.send(Msg::Field(c)))),
    );
    let session = (fake.session).watch(Box::new(move |n| drop(to_session.send(Msg::Session(n)))));
    Ports {
        bus: ModuleCx::new(host).bus(),
        clock: Arc::new(fake.clock.clone()),
        calendar: Arc::new(fake.calendar.clone()),
        speech: Arc::new(fake.speech.clone()),
        files: Arc::new(fake.files.clone()),
        clip: Arc::clone(&fake.clip) as _,
        reveal: Arc::clone(&fake.reveal) as _,
        shelf: Arc::new(fake.shelf.clone()),
        cloud: Some(Arc::new(fake.cloud.clone())),
        link: Some(Arc::clone(&fake.link) as _),
        field: field.ok(),
        session: session.ok(),
        handle: host.handle(),
        back: tx,
        spawn: in_place,
        typist: typist(asks),
    }
}
