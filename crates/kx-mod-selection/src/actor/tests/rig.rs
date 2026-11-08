//! A rig for the actor: the fake host's bus and settings, the fake watch, link and clock, and what `taps` heard. The
//! test hands each message to the actor as its thread would and opens run in place, so no test needs a thread or a
//! sleep.

use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use kx_module_api::ports::{
    ChipAct, ChipsView, EditAsk, EditDone, Piece, SelChip, SelSpot, SelectionWatch,
};
use kx_module_api::{Bus, ModuleCx, Mono, Notice, Redacted, Subscription, keys};
use kx_platform_fake::{FakeClock, FakeSelectionWatch, FakeWebLink};
use kx_test_support::host::FakeHost;

use super::super::{Actor, Msg, Ports};
use super::taps::{Counted, Heard, held, in_place, listen, release};
use crate::MANIFEST;
use crate::settings::SelSettings;

/// The system's display language the rig's module was built with.
pub(super) const OS: &str = "fr-FR";
/// When the rig's clock starts.
const START: Mono = Mono::from_us(1_000);

/// The actor on a fake host, its fakes, and what it published.
pub(super) struct Rig {
    pub watch: FakeSelectionWatch,
    pub web: Arc<FakeWebLink>,
    pub clock: FakeClock,
    pub host: FakeHost,
    actor: Actor,
    inbox: Receiver<Msg>,
    asked: Arc<Mutex<Vec<(u64, u16)>>>,
    heard: Arc<Mutex<Vec<Heard>>>,
    _ears: Vec<Subscription>,
}

impl Rig {
    /// A begun rig on the default settings.
    pub fn new() -> Self {
        Self::build(|_| {})
    }

    /// A begun rig whose settings `setup` prepares first.
    pub fn build(setup: impl FnOnce(&mut FakeHost)) -> Self {
        let mut host = FakeHost::new(&MANIFEST);
        host.granted = MANIFEST.capabilities.to_vec();
        setup(&mut host);
        let settings: SelSettings = ModuleCx::new(&mut host).settings().unwrap();
        let (watch, clock) = (FakeSelectionWatch::new(), FakeClock::new(START));
        let web = Arc::new(FakeWebLink::new());
        let (tx, inbox) = mpsc::channel();
        let wake = tx.clone();
        let tell = Box::new(move || drop(wake.send(Msg::Wake)));
        let asked = Arc::default();
        let run = Counted {
            run: watch.watch(settings.sel_config(), tell).unwrap(),
            asked: Arc::clone(&asked),
        };
        let heard = Arc::default();
        let ears = listen(&Bus::new(Arc::clone(&host.core)), &heard);
        let ports = Ports {
            bus: ModuleCx::new(&mut host).bus(),
            clock: Arc::new(clock.clone()),
            run: Some(Box::new(run)),
            web: Arc::clone(&web) as _,
            handle: host.handle(),
            back: tx,
            spawn: in_place,
        };
        let mut actor = Actor::new(ports, &settings, OS);
        actor.begin();
        let mut rig = Self {
            watch,
            web,
            clock,
            host,
            actor,
            inbox,
            asked,
            heard,
            _ears: ears,
        };
        rig.pump();
        rig
    }

    /// Hands each message the watch or an open sent to the actor, as its thread does, until none is left.
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

    /// Focus moves to a new box that looks as `spot` says; returns its id.
    pub fn focus(&mut self, spot: SelSpot) -> u64 {
        self.watch.focus(spot);
        self.pump();
        self.watch.spot().box_id
    }

    /// The focused box now looks as `spot` says, and the watch tells it.
    pub fn change(&mut self, spot: SelSpot) {
        self.watch.change(spot);
        self.pump();
    }

    /// A press and release in another app, as a click there.
    pub fn click(&mut self) {
        self.send(Msg::Pressed(false));
        self.send(Msg::Released(false));
    }

    /// The keyboard typed a break, as our Esc or an arrow.
    pub fn broke(&mut self) {
        self.send(Msg::typed(&Piece::Break));
    }

    /// The keyboard typed a letter.
    pub fn letter(&mut self) {
        let text = Piece::Text(Redacted::new("a".to_owned()));
        self.send(Msg::typed(&text));
    }

    /// A click on `chip` of the strip shown last.
    pub fn act(&mut self, chip: SelChip) {
        let round = self.round();
        self.send(Msg::Act(ChipAct { round, chip }));
    }

    /// The keyboard answers the last edit asked.
    pub fn answer(&mut self, ok: bool) {
        let round = self.edits().last().unwrap().round;
        self.send(Msg::Done(EditDone { round, ok }));
    }

    /// The user answers the last Translate question.
    pub fn reply(&mut self, yes: bool) {
        let notice = self.question().unwrap();
        self.send(Msg::Answered { notice, yes });
    }

    /// Moves the clock on by `by` and ticks.
    pub fn advance(&mut self, by: Duration) {
        self.clock.advance(by);
        self.actor.tick();
        self.pump();
    }

    /// No open's thread starts from now on.
    pub fn no_threads(&mut self) {
        self.actor.ports.spawn = |_| false;
    }

    /// Every open waits from now on, until `release_opens`.
    pub fn hold_opens(&mut self) {
        self.actor.ports.spawn = held;
    }

    /// Ends every open held so far, then what they led to.
    pub fn release_opens(&mut self) {
        release();
        self.pump();
    }

    /// How long the thread would wait now.
    pub fn wait(&self) -> Option<Duration> {
        self.actor.wait()
    }

    /// The cap of every read the actor asked, in order, refused ones too.
    pub fn reads(&self) -> Vec<u16> {
        self.asked().map(|(_, cap)| cap).collect()
    }

    /// The box of every read the actor asked, in order.
    pub fn read_boxes(&self) -> Vec<u64> {
        self.asked().map(|(id, _)| id).collect()
    }

    /// The box and cap of every read the actor asked, in order.
    fn asked(&self) -> std::vec::IntoIter<(u64, u16)> {
        self.asked.lock().unwrap().clone().into_iter()
    }

    /// The settings as the module's own writes left them.
    pub fn saved(&self) -> SelSettings {
        self.host.sink.read_as().unwrap()
    }

    /// Everything published so far.
    pub fn heard(&self) -> Vec<Heard> {
        self.heard.lock().unwrap().clone()
    }

    /// What `pick` takes from everything published, in order.
    fn picked<T>(&self, pick: impl FnMut(Heard) -> Option<T>) -> Vec<T> {
        self.heard().into_iter().filter_map(pick).collect()
    }

    /// Every view published.
    pub fn views(&self) -> Vec<ChipsView> {
        self.picked(|h| {
            if let Heard::View(v) = h {
                Some(v)
            } else {
                None
            }
        })
    }

    /// The last view published, `Hidden` before any.
    pub fn view(&self) -> ChipsView {
        self.views().pop().unwrap_or(ChipsView::Hidden)
    }

    /// The round and keycaps of the strip shown last; it fails the test when none shows or the last one is a tick.
    pub fn shown(&self) -> (u32, Vec<SelChip>) {
        match self.view() {
            ChipsView::Shown {
                round,
                chips,
                acted: None,
                ..
            } => (round, chips),
            _ => panic!("no live strip shows: {:?}", self.heard()),
        }
    }

    /// The round of the last strip shown, hidden since or not.
    pub fn round(&self) -> u32 {
        let mut views = self.views().into_iter().rev();
        views
            .find_map(|v| match v {
                ChipsView::Shown { round, .. } => Some(round),
                ChipsView::Hidden => None,
            })
            .unwrap()
    }

    /// Every edit asked of the keyboard.
    pub fn edits(&self) -> Vec<EditAsk> {
        self.picked(|h| {
            if let Heard::Edit(e) = h {
                Some(e)
            } else {
                None
            }
        })
    }

    /// Every notice raised.
    pub fn notices(&self) -> Vec<Notice> {
        self.picked(|h| {
            if let Heard::Notice(n) = h {
                Some(n)
            } else {
                None
            }
        })
    }

    /// The keys of every notice raised.
    pub fn keys(&self) -> Vec<&'static str> {
        self.notices().iter().map(|n| n.key).collect()
    }

    /// The last Translate question raised.
    pub fn question(&self) -> Option<Notice> {
        let mut notices = self.notices().into_iter().rev();
        notices.find(|n| n.key == keys::TRANSLATE_ASK)
    }

    /// Every presses wish told, in order.
    pub fn presses(&self) -> Vec<bool> {
        self.picked(|h| {
            if let Heard::Presses(on) = h {
                Some(on)
            } else {
                None
            }
        })
    }

    /// How many events were heard so far.
    pub fn count(&self) -> usize {
        self.heard.lock().unwrap().len()
    }
}
