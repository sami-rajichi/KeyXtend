//! The `kx-clip` actor (spec §7, ADR-0034): it keeps the watch's copies within the limits, answers the panel's keys
//! and clicks, inserts through the keyboard and saves on a short debounce. A plain struct driven by messages, plus a
//! loop that waits for the next message or timer.

mod copy;
mod days;
mod edit;
mod look;
mod paste;
mod show;

use std::collections::BTreeSet;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender};
use std::time::Duration;

use kx_module_api::ports::{
    Calendar, ClipAct, ClipNote, ClipPanel, ClipRun, ClipboardWriter, Date, HistoryWanted,
    PanelKey, PanelNote, Pasted, Room, SealedShelf,
};
use kx_module_api::{Bus, Clock, Event, Mono, Notice, NoticeArg, args, keys};

use crate::ID;
use crate::follow::Follow;
use crate::keep::Limits;
use crate::media::{Drawn, Facts, Job, Look};
use crate::order::History;
use crate::panel::Panel;
use crate::settings::ClipSettings;
use crate::store::{Loaded, Store};
use paste::Pasting;

/// A message for the actor, from the watch, the bus, the helper threads or the module.
#[derive(Debug)]
pub(crate) enum Msg {
    /// The watch told a copy, a skip or a clear.
    Note(ClipNote),
    /// The keyboard's key or chord toggles the panel.
    Open,
    /// A key for the open panel.
    Key(PanelKey),
    /// A click on the panel, or the ui's rows.
    Act(ClipAct),
    /// The keyboard's answer to a paste request.
    Pasted(Pasted),
    /// A keyboard that starts later asks for the chord flag.
    HistoryAsk,
    /// The media thread drew a picture, or failed.
    Drawn(Drawn),
    /// The files thread looked at some paths.
    Facts(Facts),
    /// The module stops.
    Quit,
}

/// The actor's ports, from the module's start or a test.
pub(crate) struct Ports {
    /// The bus, limited to the module's grants.
    pub bus: Bus,
    /// Monotonic time, for the save, paste and midnight timers.
    pub clock: Arc<dyn Clock>,
    /// The wall time, for the days.
    pub calendar: Arc<dyn Calendar>,
    /// The shelf, for its room; the store holds it too.
    pub shelf: Arc<dyn SealedShelf>,
    /// Sets the clipboard for an insert.
    pub writer: Arc<dyn ClipboardWriter>,
    /// The media thread's inbox.
    pub media: Sender<Job>,
    /// The files thread's inbox.
    pub files: Sender<Look>,
    /// The running watch, dropped at stop so no note comes after.
    pub watch: Option<Box<dyn ClipRun>>,
}

/// The clipboard module's runtime: one per start, on its own thread.
#[allow(
    clippy::struct_excessive_bools,
    reason = "Each flag is one fact kept between messages: rows heard, a redraw, a stale room, an aside, a stop."
)]
pub(crate) struct Actor {
    ports: Ports,
    settings: ClipSettings,
    limits: Limits,
    store: Store,
    history: History,
    follow: Follow,
    panel: Panel,
    /// The ui told its rows, so views may show (Ruling 32).
    rows_heard: bool,
    /// The calendar day the history was last swept on.
    today: Date,
    /// The shelf's room as last read, and whether a change made it stale.
    room: Option<Room>,
    room_stale: bool,
    /// The note the panel shows, until a copy is kept or the panel closes.
    note: Option<PanelNote>,
    /// The first unsaved change, and the last save failed.
    dirty: Option<Mono>,
    failing: bool,
    /// The open panel draws again after this message.
    redraw: bool,
    paste: Pasting,
    /// The entries whose thumbnail was asked this run.
    asked: BTreeSet<u64>,
    /// The notices told this run.
    told: BTreeSet<&'static str>,
    /// The load set a list aside.
    aside: bool,
    stopped: bool,
}

impl Actor {
    /// An actor over the history `loaded` by `store`, which it saves from now on.
    pub(crate) fn new(ports: Ports, settings: &ClipSettings, store: Store, loaded: Loaded) -> Self {
        let today = ports.calendar.now().date;
        Self {
            ports,
            limits: settings.limits(),
            store,
            history: loaded.history,
            follow: settings.follow(),
            panel: Panel::new(settings.panel_config()),
            rows_heard: false,
            today,
            room: None,
            room_stale: true,
            note: None,
            dirty: None,
            failing: false,
            redraw: false,
            paste: Pasting::default(),
            asked: BTreeSet::new(),
            told: BTreeSet::new(),
            aside: !loaded.aside.is_empty(),
            stopped: false,
            settings: settings.clone(),
        }
    }

    /// The thread's loop: begins, then serves each message and timer until `Quit` or the module is gone.
    pub(crate) fn run(mut self, inbox: &Receiver<Msg>) {
        self.begin();
        loop {
            match inbox.recv_timeout(self.wait()) {
                Ok(Msg::Quit) | Err(RecvTimeoutError::Disconnected) => return self.quit(),
                Ok(msg) => self.handle(msg),
                Err(RecvTimeoutError::Timeout) => {}
            }
            self.tick();
        }
    }

    /// Tells a damaged load once, sweeps what passed its limits since the last run, saves, and tells the chord flag.
    pub(crate) fn begin(&mut self) {
        if self.aside {
            self.say_once(keys::CLIP_DAMAGED);
        }
        self.sweep();
        let on = self.settings.history_key;
        self.publish(&HistoryWanted { on });
    }

    /// Handles one message; nothing after the stop, so a late note from the watch is dropped.
    pub(crate) fn handle(&mut self, msg: Msg) {
        if self.stopped {
            return;
        }
        self.new_day();
        match msg {
            Msg::Note(note) => self.note(note),
            Msg::Open => self.toggle(),
            Msg::Key(key) => {
                let done = self.panel.key(&key, &self.history);
                self.act_on(done);
            }
            Msg::Act(act) => self.act(act),
            Msg::Pasted(pasted) => self.pasted(pasted),
            Msg::HistoryAsk => self.history_again(),
            Msg::Drawn(drawn) => self.drawn(drawn),
            Msg::Facts(facts) => self.facts(facts),
            Msg::Quit => self.quit(),
        }
    }

    /// Runs the timers that are due, then draws the open panel once if anything changed.
    pub(crate) fn tick(&mut self) {
        if self.stopped {
            return;
        }
        self.new_day();
        let now = self.ports.clock.now();
        let save = self.settings.save();
        if self.dirty.is_some_and(|t| now.saturating_since(t) >= save) {
            self.save();
        }
        self.paste_tick(now);
        self.flush();
    }

    /// How long the loop waits for a message: until the save, the paste's next step or wait, or local midnight.
    pub(crate) fn wait(&self) -> Duration {
        let now = self.ports.clock.now();
        let save = (self.dirty).map(|t| t.saturating_add(self.settings.save()));
        let due = [save, self.paste_due()].into_iter().flatten();
        let midnight = days::until_midnight(&self.ports.calendar.now());
        due.map(|t| t.saturating_since(now))
            .fold(midnight, Duration::min)
    }

    /// Stops: the watch goes first, then the history saves, and the keyboard hears the panel closed and the chord free.
    pub(crate) fn quit(&mut self) {
        if self.stopped {
            return;
        }
        self.stopped = true;
        self.ports.watch = None;
        self.paste = Pasting::default();
        self.save();
        self.panel.close();
        self.publish(&ClipPanel { open: false });
        self.publish(&HistoryWanted { on: false });
    }

    /// Tells the user `key` once per run.
    fn say_once(&mut self, key: &'static str) {
        if self.told.insert(key) {
            self.publish(&notice(key));
        }
    }

    /// Publishes `event`; a refusal is the bus's gate on a capability the module holds, so it never happens.
    fn publish<E: Event>(&self, event: &E) {
        let _ = self.ports.bus.publish(event);
    }
}

/// The module's notice `key`, naming the module.
pub(crate) fn notice(key: &'static str) -> Notice {
    Notice {
        key,
        module: Some(ID),
        args: vec![(args::MODULE, NoticeArg::Text(ID.to_string()))],
    }
}

#[cfg(test)]
mod tests;
