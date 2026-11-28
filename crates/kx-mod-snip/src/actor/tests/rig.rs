//! A rig for the actor: the fake host's bus and settings, every port's fake over one Snips folder, and what the actor
//! published. The test hands each message to the actor as its thread would and jobs run in place, or wait until
//! released, so no test needs a thread or a sleep.

use std::cell::RefCell;
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use kx_module_api::ports::{
    BarKey, Keep, PanelKey, PanelLine, RecKey, SnipAct, SnipAside, SnipClick, SnipLine, SnipLit,
    SnipPage, SnipStage, SnipView, StripKey,
};
use kx_module_api::{Bus, Event, ModuleCx, Subscription};
use kx_test_support::host::FakeHost;

use super::super::{Actor, Job, Msg};
pub(super) use super::fakes::{FAR, Fakes, NEAR};
use crate::{MANIFEST, SnipSettings};

thread_local! {
    /// The jobs `held` keeps on this test's thread until `release`.
    static HELD: RefCell<Vec<Job>> = const { RefCell::new(Vec::new()) };
}

/// Runs a job in place, as a test has no thread to spare.
pub(super) fn in_place(job: Job) -> bool {
    job();
    true
}

/// Keeps a job for later, as a worker or a dialog that takes its time.
pub(super) fn held(job: Job) -> bool {
    HELD.with(|h| h.borrow_mut().push(job));
    true
}

/// Ends every job `held` kept on this thread.
pub(super) fn release() {
    for job in HELD.with(|h| std::mem::take(&mut *h.borrow_mut())) {
        job();
    }
}

/// Ends the oldest job `held` kept on this thread; false when none was kept.
pub(super) fn release_one() -> bool {
    let job = HELD.with(|h| {
        let mut held = h.borrow_mut();
        (!held.is_empty()).then(|| held.remove(0))
    });
    job.map(|job| job()).is_some()
}

/// Drops every job `held` kept on this thread unrun, as a job that dies before its end.
pub(super) fn drop_held() {
    drop(HELD.with(|h| std::mem::take(&mut *h.borrow_mut())));
}

/// Starts no job, as a system out of threads.
pub(super) fn refuse(_: Job) -> bool {
    false
}

/// True while `held` keeps a job on this thread.
fn holding() -> bool {
    HELD.with(|h| !h.borrow().is_empty())
}

/// One event the actor published.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Heard {
    View(Box<SnipPage>),
    Lit(SnipLit),
    Aside(SnipAside),
    Line(Option<PanelLine>),
}

/// The actor on a fake host, its fakes, and what it published.
pub(super) struct Rig {
    pub fake: Fakes,
    pub host: FakeHost,
    pub actor: Actor,
    inbox: Receiver<Msg>,
    heard: Arc<Mutex<Vec<Heard>>>,
    _ears: Vec<Subscription>,
}

impl Rig {
    /// A begun rig on the defaults.
    pub fn new() -> Self {
        Self::build(|_, _| {})
    }

    /// A begun rig whose fakes and settings `setup` prepares first.
    pub fn build(setup: impl FnOnce(&Fakes, &mut FakeHost)) -> Self {
        Self::with(Fakes::new(), setup)
    }

    /// A begun rig over `fake`, prepared by `setup`.
    pub fn with(fake: Fakes, setup: impl FnOnce(&Fakes, &mut FakeHost)) -> Self {
        let mut host = FakeHost::new(&MANIFEST);
        host.granted = MANIFEST.capabilities.to_vec();
        setup(&fake, &mut host);
        let settings: SnipSettings = ModuleCx::new(&mut host).settings().unwrap();
        let (tx, inbox) = mpsc::channel();
        let heard = Arc::default();
        let ears = listen(&Bus::new(Arc::clone(&host.core)), &heard);
        let bus = ModuleCx::new(&mut host).bus();
        let ports = fake.ports(bus, host.handle(), tx);
        let mut rig = Self {
            actor: Actor::new(ports, settings),
            fake,
            host,
            inbox,
            heard,
            _ears: ears,
        };
        rig.actor.begin();
        rig.pump();
        rig
    }

    /// Hands each message a port or a job sent to the actor, as its thread does, until none is left.
    pub fn pump(&mut self) {
        while let Ok(msg) = self.inbox.try_recv() {
            self.actor.handle(msg);
            self.actor.tick();
        }
    }

    /// Ends every held job and hands over what each led to, until no job is held.
    pub fn drain(&mut self) {
        while holding() {
            release();
            self.pump();
        }
    }

    /// Takes the next message a port or a job sent, unhandled, so a test can hand it over later.
    pub fn next(&mut self) -> Option<Msg> {
        self.inbox.try_recv().ok()
    }

    /// Hands `msg` to the actor and ticks, as the thread does, then what it led to.
    pub fn send(&mut self, msg: Msg) {
        self.actor.handle(msg);
        self.actor.tick();
        self.pump();
    }

    /// Clicks the Snip key, then answers the keyboard's step aside as the ui does; an answer no freeze waits for does
    /// nothing.
    pub fn key(&mut self) {
        self.send(Msg::Key);
        self.act(SnipClick::Aside);
    }

    /// Clicks `click` on the view told last.
    pub fn act(&mut self, click: SnipClick) {
        let round = self.page().round;
        self.send(Msg::Act(SnipAct { round, click }));
    }

    /// A key of the keyboard aimed at a snip line.
    pub fn line(&mut self, key: PanelKey) {
        self.send(Msg::LineKey(key));
    }

    /// Moves the clock on by `by` and ticks.
    pub fn advance(&mut self, by: Duration) {
        self.fake.clock.advance(by);
        self.actor.tick();
        self.pump();
    }

    /// Freezes and clicks the corners `NEAR` and `FAR` for `way`, a region.
    pub fn take(&mut self, way: BarKey) {
        self.key();
        self.act(SnipClick::Bar(way));
        self.act(SnipClick::Spot(NEAR));
        self.act(SnipClick::Spot(FAR));
    }

    /// A picture on its strip.
    pub fn snap(&mut self) {
        self.take(BarKey::Region);
        assert!(
            matches!(self.stage(), SnipStage::Strip(_)),
            "a picture is taken"
        );
    }

    /// A video recording, past its countdown, its run started and its first time told.
    pub fn record(&mut self) {
        self.take(BarKey::VideoRegion);
        self.act(SnipClick::Strip(StripKey::Record));
        let countdown = u64::from(self.actor.held.countdown_s);
        for _ in 0..countdown {
            self.advance(Duration::from_secs(1));
        }
        assert!(self.fake.recorder.time(), "a run records");
        self.pump();
        assert!(matches!(self.stage(), SnipStage::Record(_)), "it records");
    }

    /// A recorded video on its strip, stopped after `ms`.
    pub fn recorded(&mut self, ms: u64) {
        self.record();
        self.fake.clock.advance(Duration::from_millis(ms));
        self.act(SnipClick::Record(RecKey::Stop));
        assert!(
            matches!(self.stage(), SnipStage::Strip(_)),
            "the video is kept"
        );
    }

    /// Copies or saves the strip's snip.
    pub fn keep(&mut self, keep: Keep) {
        let key = match keep {
            Keep::Copy => StripKey::Copy,
            Keep::Save => StripKey::Save,
        };
        self.act(SnipClick::Strip(key));
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
    pub fn views(&self) -> Vec<SnipPage> {
        self.picked(|h| match h {
            Heard::View(page) => Some(*page),
            _ => None,
        })
    }

    /// The view told last.
    pub fn page(&self) -> SnipPage {
        self.views().pop().unwrap()
    }

    /// The windows told last.
    pub fn stage(&self) -> SnipStage {
        self.page().stage
    }

    /// Every light and step aside told, in order.
    pub fn sides(&self) -> Vec<Heard> {
        self.picked(|h| matches!(h, Heard::Lit(_) | Heard::Aside(_)).then_some(h))
    }

    /// The line told last.
    pub fn told_line(&self) -> Option<PanelLine> {
        self.picked(|h| match h {
            Heard::Line(line) => Some(line),
            _ => None,
        })
        .pop()
        .flatten()
    }

    /// The settings as the module's own writes left them.
    pub fn saved(&self) -> SnipSettings {
        self.host.sink.read_as().unwrap()
    }
}

/// Listens on `bus` for everything the actor publishes, into `heard`.
fn listen(bus: &Bus, heard: &Arc<Mutex<Vec<Heard>>>) -> Vec<Subscription> {
    vec![
        ear(bus, heard, |e: &SnipView| {
            Heard::View(Box::new((*e.0).clone()))
        }),
        ear(bus, heard, |e: &SnipLit| Heard::Lit(*e)),
        ear(bus, heard, |e: &SnipAside| Heard::Aside(*e)),
        ear(bus, heard, |e: &SnipLine| Heard::Line(e.0.clone())),
    ]
}

/// Keeps each `E` on `bus` in `heard` as `make` says.
fn ear<E: Event>(
    bus: &Bus,
    heard: &Arc<Mutex<Vec<Heard>>>,
    make: impl Fn(&E) -> Heard + Send + Sync + 'static,
) -> Subscription {
    let heard = Arc::clone(heard);
    bus.subscribe(move |e: &E| heard.lock().unwrap().push(make(e)))
        .unwrap()
}
