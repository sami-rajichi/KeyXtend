//! A rig for the actor: the fake host's bus, the fakes behind its ports and every event it publishes. The test hands
//! each message to the actor as its thread would, so no test needs a thread or a sleep.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use kx_module_api::ports::{
    ClipTake, Date, EntryKind, FillAct, FillClick, FillNote, FillPage, FillType, PanelKey,
    PanelLine, Sheet,
};
use kx_module_api::{Bus, ModuleCx, Mono, Notice, Redacted, Subscription};
use kx_platform_fake::{FakeCalendar, FakeClock, FakeRandom, FakeSealedStore, PARIS};
use kx_test_support::host::{FakeHost, FakeSink};

use super::super::{Actor, Msg, Ports, Stores};
use super::boxes::Fakes;
use super::ears::{Heard, listen};
use crate::MANIFEST;
use crate::backup::{BackupStore, Kept};
use crate::book::Notebook;
use crate::entry::{Draft, Stamp};
use crate::fixture::{LIMITS, draft, field};
use crate::settings::VaultSettings;
use crate::store::Store;

/// The rows the tests' ui shows.
pub(super) const ROWS: u8 = 5;
/// When each test starts.
const T0: Mono = Mono::from_us(1_000_000);
/// When the sample was saved, before the rig's wall time.
const SAVED: Stamp = Stamp::from_unix_ms(1_700_000_000_000);
/// The login's user and password, and the person's email.
pub(super) const USER: &str = "sam-user";
pub(super) const PASSWORD: &str = "zebra-quartz";
pub(super) const EMAIL: &str = "alex@example.com";

/// The actor over a saved notebook, its fakes, and what it published.
pub(super) struct Rig {
    pub sealed: FakeSealedStore,
    pub clock: FakeClock,
    pub calendar: FakeCalendar,
    pub fakes: Fakes,
    pub sink: Arc<FakeSink>,
    pub actor: Actor,
    heard: Arc<Mutex<Vec<Heard>>>,
    _ears: Vec<Subscription>,
}

impl Rig {
    /// A begun rig over the sample: a login, a person with an email and a note.
    pub fn new() -> Self {
        Self::with(sample())
    }

    /// A begun rig over `drafts`, saved first as an earlier run would have, which already made the first offer.
    pub fn with(drafts: Vec<Draft>) -> Self {
        let asked = Kept {
            offered: true,
            ..Kept::default()
        };
        Self::kept(drafts, &asked)
    }

    /// A begun rig over `drafts` and the backup state `kept`, both saved first as an earlier run would have.
    pub fn kept(drafts: Vec<Draft>, kept: &Kept) -> Self {
        Self::build(drafts, kept, false)
    }

    /// A begun rig over the backup state `kept` whose notebook loaded damaged, so it starts empty.
    pub fn held(kept: &Kept) -> Self {
        Self::build(Vec::new(), kept, true)
    }

    /// A begun rig over `drafts` and `kept`, the notebook loaded damaged or not.
    fn build(drafts: Vec<Draft>, kept: &Kept, damaged: bool) -> Self {
        let (sealed, random) = (FakeSealedStore::new(), FakeRandom::new(7));
        let mut stores = saved(&sealed, &random, drafts, kept);
        stores.damaged = damaged;
        let mut host = FakeHost::new(&MANIFEST);
        host.granted = MANIFEST.capabilities.to_vec();
        let (clock, day) = (FakeClock::new(T0), Date::new(2026, 11, 2).unwrap());
        let calendar = FakeCalendar::new(day, 9, 30, PARIS);
        let ports = Ports {
            bus: ModuleCx::new(&mut host).bus(),
            clock: Arc::new(clock.clone()),
            calendar: Arc::new(calendar.clone()),
            random: Arc::new(random),
        };
        let settings = VaultSettings::from_defaults().unwrap();
        let fakes = Fakes::new(settings.box_config());
        let links = fakes.links(host.handle());
        let heard = Arc::default();
        let ears = listen(&Bus::new(Arc::clone(&host.core)), &heard);
        let mut rig = Self {
            actor: Actor::new(ports, links, &settings, stores),
            sealed,
            clock,
            calendar,
            fakes,
            sink: Arc::clone(&host.sink),
            heard,
            _ears: ears,
        };
        rig.actor.begin();
        rig
    }

    /// Hands `msg` to the actor and ticks, as the thread does.
    pub fn send(&mut self, msg: Msg) {
        self.actor.handle(msg);
        self.actor.tick();
    }

    /// Moves the clock on by `by` and ticks.
    pub fn advance(&mut self, by: Duration) {
        self.clock.advance(by);
        self.actor.tick();
    }

    /// Moves the wall time and the clock on by `by`, and ticks.
    pub fn pass(&mut self, by: Duration) {
        self.calendar.advance(by);
        self.advance(by);
    }

    /// Opens the panel and tells the ui's rows, as the ui does.
    pub fn open(&mut self) {
        self.send(Msg::Open);
        self.send(Msg::Act(FillAct {
            round: 0,
            click: FillClick::Rows(ROWS),
        }));
    }

    /// Clicks `click` on the last view.
    pub fn click(&mut self, click: FillClick) {
        let round = self.page().round;
        self.send(Msg::Act(FillAct { round, click }));
    }

    /// Types `text` from our keyboard.
    pub fn text(&mut self, text: &str) {
        self.send(Msg::Key(PanelKey::Text(Redacted::new(text.to_owned()))));
    }

    /// Everything published so far.
    pub fn all(&self) -> Vec<Heard> {
        self.heard.lock().unwrap().clone()
    }

    /// Every published event `pick` takes.
    fn picked<T>(&self, pick: impl Fn(Heard) -> Option<T>) -> Vec<T> {
        self.all().into_iter().filter_map(pick).collect()
    }

    /// Every page drawn.
    pub fn views(&self) -> Vec<FillPage> {
        self.picked(|h| match h {
            Heard::View(v) => Some(v.0.as_ref().clone()),
            _ => None,
        })
    }

    /// The last page drawn.
    pub fn page(&self) -> FillPage {
        self.views().pop().unwrap()
    }

    /// The last page's note.
    pub fn note(&self) -> Option<FillNote> {
        self.page().note.map(|n| n.note)
    }

    /// The names the last page's list shows.
    pub fn names(&self) -> Vec<String> {
        match self.page().sheet {
            Sheet::List(rows) => rows.iter().map(|r| r.name.expose().clone()).collect(),
            _ => Vec::new(),
        }
    }

    /// Every fill request.
    pub fn types(&self) -> Vec<FillType> {
        self.picked(|h| match h {
            Heard::Type(t) => Some(t),
            _ => None,
        })
    }

    /// Every take Paste asked.
    pub fn takes(&self) -> Vec<ClipTake> {
        self.picked(|h| match h {
            Heard::Take(t) => Some(t),
            _ => None,
        })
    }

    /// Every set of emails told, as text.
    pub fn emails(&self) -> Vec<Vec<String>> {
        let text = |e: &kx_module_api::Secret| e.expose().to_string();
        self.picked(|h| match h {
            Heard::Emails(e) => Some(e.emails.iter().map(text).collect()),
            _ => None,
        })
    }

    /// Runs the oldest queued backup job and hands its answer back, as the job thread does; false when none waits.
    pub fn job(&mut self) -> bool {
        let Some(ask) = self.fakes.take() else {
            return false;
        };
        let done = self.fakes.run(ask);
        self.send(Msg::Done(done));
        true
    }

    /// Runs every queued backup job, oldest first, until none is left.
    pub fn jobs(&mut self) {
        while self.job() {}
    }

    /// Every notice published.
    pub fn notices(&self) -> Vec<Notice> {
        self.picked(|h| match h {
            Heard::Notice(n) => Some(n),
            _ => None,
        })
    }

    /// Every `FillLive` published, as its flag.
    pub fn lives(&self) -> Vec<bool> {
        self.picked(|h| match h {
            Heard::Live(l) => Some(l.live),
            _ => None,
        })
    }

    /// Every `FillPanel` published, as its flag.
    pub fn panels(&self) -> Vec<bool> {
        self.picked(|h| match h {
            Heard::Panel(p) => Some(p.open),
            _ => None,
        })
    }

    /// Every `FillLine` published, as its line.
    pub fn lines(&self) -> Vec<PanelLine> {
        self.picked(|h| match h {
            Heard::Line(l) => Some(l),
            _ => None,
        })
    }
}

/// A login, a person with an email and a note, A to Z: Alex, Bank, Gate.
pub(super) fn sample() -> Vec<Draft> {
    let user = field("user", USER, false);
    let login = draft(
        "Bank",
        EntryKind::Login,
        vec![user, field("password", PASSWORD, true)],
    );
    let person = vec![
        field("full-name", "Alex Martin", false),
        field("email", EMAIL, false),
    ];
    let note = vec![field("text", "4321", false)];
    vec![
        login,
        draft("Alex", EntryKind::Person, person),
        draft("Gate", EntryKind::Note, note),
    ]
}

/// Saves `drafts` and `kept` into `sealed` as an earlier run would have, and loads them as the module does.
fn saved(sealed: &FakeSealedStore, random: &FakeRandom, drafts: Vec<Draft>, kept: &Kept) -> Stores {
    let mut book = Notebook::default();
    for d in drafts {
        book.add(d, SAVED, random, &LIMITS).unwrap();
    }
    let mut store = Store::new(Arc::new(sealed.clone()));
    store.save(&book).unwrap();
    let mut backup = BackupStore::new(Arc::new(sealed.clone()));
    backup.save(kept).unwrap();
    let (loaded, kept) = (store.load().unwrap(), backup.load().unwrap());
    Stores {
        store,
        book: loaded.book,
        backup,
        kept: kept.kept,
        damaged: loaded.damaged,
    }
}
