//! A rig for the actor: the fake host's bus, the fakes behind its ports, and every event it publishes. The test
//! hands each message to the actor as its thread would and runs the helpers' jobs on its own thread, so no test needs
//! a thread or a sleep.

use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use kx_module_api::ports::{
    AppRef, BlobName, ClipAct, ClipNote, ClipPage, ClipPart, ClipView, Clipboard, PasteAsk, Pasted,
    PictureRun, Pictures, SealedShelf,
};
use kx_module_api::{Bus, ModuleCx, Subscription};
use kx_test_support::host::FakeHost;

use super::super::{Actor, Msg, Ports};
use super::ears::{Heard, listen};
use super::fakes::Fakes;
use crate::MANIFEST;
use crate::fixture::text;
use crate::media::{self, Job, Look, Sides};
use crate::settings::ClipSettings;
use crate::store::Store;

/// The rows the tests' ui shows.
pub(super) const ROWS: u8 = 5;

/// The actor on a fake host, its fakes, and what it published.
pub(super) struct Rig {
    pub fakes: Fakes,
    pub settings: ClipSettings,
    actor: Actor,
    notes: Receiver<Msg>,
    jobs: Receiver<Job>,
    looks: Receiver<Look>,
    run: Box<dyn PictureRun>,
    heard: Arc<Mutex<Vec<Heard>>>,
    _ears: Vec<Subscription>,
}

impl Rig {
    /// A begun rig on the default settings, on 31 October 2026 at 16:32 in Paris.
    pub fn new() -> Self {
        Self::build(|_, _| {})
    }

    /// A begun rig whose fakes and settings `setup` prepares first.
    pub fn build(setup: impl FnOnce(&Fakes, &mut FakeHost)) -> Self {
        let fakes = Fakes::new();
        let mut host = FakeHost::new(&MANIFEST);
        host.granted = MANIFEST.capabilities.to_vec();
        setup(&fakes, &mut host);
        let settings: ClipSettings = ModuleCx::new(&mut host).settings().unwrap();
        let mut store = Store::new(Arc::new(fakes.shelf.clone()), settings.store_config());
        let loaded = store.load().unwrap();
        let (tx, notes) = mpsc::channel();
        let tell = Box::new(move |n| drop(tx.send(Msg::Note(n))));
        let watch = fakes.clip.watch(settings.clip_config(), tell).unwrap();
        let ((media, jobs), (files, looks)) = (mpsc::channel(), mpsc::channel());
        let ports = Ports {
            bus: ModuleCx::new(&mut host).bus(),
            clock: Arc::new(fakes.clock.clone()),
            calendar: Arc::new(fakes.calendar.clone()),
            shelf: Arc::new(fakes.shelf.clone()),
            writer: Arc::new(fakes.writer.clone()),
            media,
            files,
            watch: Some(watch),
        };
        let heard = Arc::default();
        let ears = listen(&Bus::new(Arc::clone(&host.core)), &heard);
        let run = fakes.pics.open(settings.picture_config());
        let actor = Actor::new(ports, &settings, store, loaded);
        let mut rig = Self {
            fakes,
            settings,
            actor,
            notes,
            jobs,
            looks,
            run,
            heard,
            _ears: ears,
        };
        rig.actor.begin();
        rig.pump();
        rig
    }

    /// Hands `msg` to the actor and ticks, as the thread does, then the helpers' work.
    pub fn send(&mut self, msg: Msg) {
        self.actor.handle(msg);
        self.actor.tick();
        self.pump();
    }

    /// Runs each note the watch told, each picture asked and each look asked, handing the answers back, until none.
    fn pump(&mut self) {
        loop {
            let sides = Sides {
                thumb: self.settings.thumb_px,
                preview: self.settings.preview_px,
            };
            let msg = if let Ok(note) = self.notes.try_recv() {
                note
            } else if let Ok(job) = self.jobs.try_recv() {
                Msg::Drawn(media::draw(&mut *self.run, job, sides))
            } else if let Ok(look) = self.looks.try_recv() {
                Msg::Facts(media::look(&self.fakes.clip, &look))
            } else {
                return;
            };
            self.actor.handle(msg);
            self.actor.tick();
        }
    }

    /// The watch tells `note`.
    pub fn push(&mut self, note: ClipNote) {
        self.fakes.clip.push(note);
        self.pump();
    }

    /// A copy of `parts` by `app`.
    pub fn copy(&mut self, parts: Vec<ClipPart>, app: Option<AppRef>) {
        self.push(ClipNote::Copied { parts, app });
    }

    /// A copy of the text `s` by an unknown app.
    pub fn text(&mut self, s: &str) {
        self.copy(vec![text(s)], None);
    }

    /// Opens the panel and tells the ui's rows, as the ui does.
    pub fn open(&mut self) {
        self.send(Msg::Open);
        self.send(Msg::Act(ClipAct::Rows(ROWS)));
    }

    /// A click on row `row` of the last page.
    pub fn tick_row(&mut self, row: u8) {
        let round = self.page().round;
        self.send(Msg::Act(ClipAct::Tick { round, row }));
    }

    /// The keyboard answers the last paste request.
    pub fn answer(&mut self, ok: bool) {
        let round = self.pastes().last().unwrap().round;
        self.send(Msg::Pasted(Pasted { round, ok }));
    }

    /// Moves the clock on by `by` and ticks.
    pub fn advance(&mut self, by: Duration) {
        self.fakes.clock.advance(by);
        self.actor.tick();
        self.pump();
    }

    /// Sets the wall clock to `hour`:`minute` on `day`, moves the clock on by the wait, and ticks.
    pub fn set_time(&mut self, day: kx_module_api::ports::Date, hour: u8, minute: u8) {
        let wait = self.actor.wait();
        self.fakes.calendar.set(day, hour, minute);
        self.advance(wait);
    }

    /// How long the thread would wait now.
    pub fn wait(&self) -> Duration {
        self.actor.wait()
    }

    /// The module stops.
    pub fn stop(&mut self) {
        self.send(Msg::Quit);
    }

    /// Everything published so far.
    pub fn all(&self) -> Vec<Heard> {
        self.heard.lock().unwrap().clone()
    }

    /// Every published event `pick` takes, as it says.
    fn picked<T>(&self, pick: impl Fn(Heard) -> Option<T>) -> Vec<T> {
        self.all().into_iter().filter_map(pick).collect()
    }

    /// Every view published.
    pub fn views(&self) -> Vec<ClipView> {
        self.picked(|h| {
            if let Heard::View(v) = h {
                Some(v)
            } else {
                None
            }
        })
    }

    /// The last page published.
    pub fn page(&self) -> ClipPage {
        self.views().last().unwrap().0.as_ref().clone()
    }

    /// The last page's titles.
    pub fn titles(&self) -> Vec<String> {
        let rows = self.page().rows;
        rows.iter().map(|r| r.title.expose().clone()).collect()
    }

    /// Every `ClipPanel` published, as its flag.
    pub fn panels(&self) -> Vec<bool> {
        self.picked(|h| {
            if let Heard::Panel(p) = h {
                Some(p.open)
            } else {
                None
            }
        })
    }

    /// Every `HistoryWanted` published, as its flag.
    pub fn wanted(&self) -> Vec<bool> {
        self.picked(|h| {
            if let Heard::Wanted(w) = h {
                Some(w.on)
            } else {
                None
            }
        })
    }

    /// Every paste request.
    pub fn pastes(&self) -> Vec<PasteAsk> {
        self.picked(|h| {
            if let Heard::Paste(p) = h {
                Some(p)
            } else {
                None
            }
        })
    }

    /// Every notice key published.
    pub fn notices(&self) -> Vec<&'static str> {
        self.picked(|h| {
            if let Heard::Notice(n) = h {
                Some(n.key)
            } else {
                None
            }
        })
    }

    /// The blob names in `folder`.
    pub fn names(&self, folder: &str) -> Vec<String> {
        let folder = BlobName::new(folder).unwrap();
        let names = self.fakes.shelf.names(&folder).unwrap();
        names.into_iter().map(|n| n.as_str().to_owned()).collect()
    }

    /// Deletes the blobs of the entry titled `title` behind the actor's back, so its content is gone.
    pub fn lose(&mut self, title: &str) {
        let entries = self.actor.history.entries();
        let entry = entries.iter().find(|e| e.title.expose() == title).unwrap();
        self.actor.store.delete(&entry.clone()).unwrap();
    }

    /// Ends the files thread, as if it had died.
    pub fn lose_files(&mut self) {
        self.looks = mpsc::channel().1;
    }
}

/// The name of today's folder in the rig.
pub(super) const TODAY: &str = "2026-10-31";
/// The pinned folder's name.
pub(super) const PINNED: &str = "pinned";
/// A folder list's head blob.
pub(super) const LIST: &str = "list";
