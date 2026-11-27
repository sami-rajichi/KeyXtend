//! The `kx-snip` actor (ADR-0056): it hands the bus's events and the recordings' notes to the capture flow, the picking
//! rules and the editor's model, and carries out the flow's steps in order. Blocking port calls run in jobs off its
//! thread and answer through its inbox, so it never waits on the worker or a dialog.

mod clicks;
mod dib;
mod files;
mod keep;
mod keys;
mod limits;
mod media;
mod msg;
mod name;
mod outs;
mod panel;
mod ports;
mod rec;
mod reply;
mod run;
mod view;

pub(crate) use msg::{Answer, Back, Kept, Msg};
pub(crate) use ports::{Job, Leftovers, Ports};

use std::collections::VecDeque;
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::time::Duration;

use kx_module_api::ports::{
    Gear, LocalTime, RecordFacts, Rect, Shot, SnipFit, SnipLit, SnipNote, SnipTab,
};
use kx_module_api::{Event, Mono};

use crate::SnipSettings;
use crate::edit::Canvas;
use crate::flow::{Flow, In};
use files::Files;
use keep::Keeping;
use rec::Runs;
use view::Told;

/// What the system and the Snips folder allow now, which dims keys.
#[derive(Debug)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "Each flag is one fact seen between messages: a hidden run, a bare window, a low drive, its strip shown."
)]
struct Seen {
    /// What the system can record, as last read; all ready until then.
    facts: RecordFacts,
    /// A hidden run made no Snips folder (ADR-0037), so Video and Save dim.
    hidden: bool,
    /// The ui could not keep a capture window out of captures, so video dims; kept until the module stops.
    bare: bool,
    /// The drive held less than `min_free_mb` when the strip of a video to record showed (edge 8).
    low: bool,
    /// That strip showed after the last message.
    ready: bool,
    /// The last freeze's screens, where the editor's remembered place must lie.
    screens: Vec<Rect>,
    /// Each live screen's work area as the ui last told, where the record bar and a recorded video's strip go.
    work: Vec<Rect>,
}

impl Default for Seen {
    fn default() -> Self {
        let facts = RecordFacts {
            video: true,
            mic: Gear::Ready,
            sound: Gear::Ready,
        };
        Self {
            facts,
            hidden: false,
            bare: false,
            low: false,
            ready: false,
            screens: Vec::new(),
            work: Vec::new(),
        }
    }
}

/// The snip module's runtime: one per start, on its own thread.
pub(crate) struct Actor {
    ports: Ports,
    flow: Flow,
    /// `[snip]` as last written.
    settings: SnipSettings,
    /// The settings the running snip started with, which the actor reads for it, as the flow holds its own.
    held: SnipSettings,
    seen: Seen,
    /// The editor's model, while a picture is edited.
    canvas: Option<Canvas>,
    /// The editor's picture.
    picture: Option<Shot>,
    /// The theme's sizes, as last told.
    fit: SnipFit,
    /// The note shown, and when it goes.
    note: Option<(SnipNote, Mono)>,
    /// The Snip panel's page, while it is open.
    panel: Option<SnipTab>,
    /// The folder dialogs whose jobs have not answered.
    folders: u8,
    /// The freezes asked, so only the newest one's screens count.
    freezes: u32,
    /// When the snip started, which names it in the Save dialog.
    stamp: Option<LocalTime>,
    /// When the Snip key started a snip, until its screens froze, for the debug log's time.
    keyed: Option<Mono>,
    files: Files,
    runs: Runs,
    keeping: Keeping,
    told: Told,
    /// The flow's inputs still to feed, so a step never feeds it while it is fed.
    inputs: VecDeque<In>,
    stepping: bool,
    stopped: bool,
}

impl Actor {
    /// An actor following `settings`.
    pub(crate) fn new(ports: Ports, settings: SnipSettings) -> Self {
        Self {
            flow: Flow::new(ports.clock.clone(), settings.clone()),
            held: settings.clone(),
            settings,
            ports,
            seen: Seen::default(),
            canvas: None,
            picture: None,
            fit: view::unfit(),
            note: None,
            panel: None,
            folders: 0,
            freezes: 0,
            stamp: None,
            keyed: None,
            files: Files::default(),
            runs: Runs::default(),
            keeping: Keeping::default(),
            told: Told::default(),
            inputs: VecDeque::new(),
            stepping: false,
            stopped: false,
        }
    }

    /// The thread's loop: begins, then serves each message and the deadlines until `Quit`, then the answers waiting.
    pub(crate) fn run(mut self, inbox: &Receiver<Msg>) {
        self.begin();
        loop {
            let next = match self.wait() {
                Some(wait) => inbox.recv_timeout(wait),
                None => inbox.recv().map_err(|_| RecvTimeoutError::Disconnected),
            };
            match next {
                Ok(Msg::Quit) | Err(RecvTimeoutError::Disconnected) => {
                    self.quit();
                    return inbox.try_iter().for_each(|msg| self.handle(msg));
                }
                Ok(msg) => self.handle(msg),
                Err(RecvTimeoutError::Timeout) => {}
            }
            self.tick();
        }
    }

    /// Learns whether the run has a Snips folder, sweeps its unsaved part, reads what the system can record, and tells
    /// the dark key and the empty view.
    pub(crate) fn begin(&mut self) {
        self.seen.hidden = self.ports.folder.snips().is_none();
        self.sweep();
        self.read_facts();
        self.publish(&SnipLit::Off);
        self.after();
    }

    /// Handles one message. After the stop, only a re-timed copy is still mapped or deleted; a run in a dropped
    /// message is cancelled and its video deleted.
    pub(crate) fn handle(&mut self, msg: Msg) {
        if self.stopped {
            if let Msg::Back(Back::Kept(kept)) = msg {
                self.kept_late(kept);
            }
            return;
        }
        match msg {
            Msg::Quit => return self.quit(),
            Msg::Key => self.key(),
            Msg::Act(act) => self.act(act),
            Msg::LineKey(key) => self.line_key(&key),
            Msg::Ask => self.asked(),
            Msg::Note { video, note } => self.heard(video, note),
            Msg::Back(back) => self.answer(back),
        }
        self.after();
    }

    /// Feeds the flow the tick whose time came, drops a note shown long enough, and tries the clipboard again.
    pub(crate) fn tick(&mut self) {
        if self.stopped {
            return;
        }
        let now = self.now();
        if self.flow.deadline().is_some_and(|due| now >= due) {
            self.step(In::Tick);
        }
        if self.note.is_some_and(|(_, until)| now >= until) {
            self.note = None;
        }
        if self.keeping.due(now) {
            self.write_again();
        }
        self.after();
    }

    /// How long the loop waits for a message: until the flow's tick, a note's end or the clipboard's next try.
    pub(crate) fn wait(&self) -> Option<Duration> {
        let now = self.now();
        let note = self.note.map(|(_, until)| until);
        let due = [self.flow.deadline(), note, self.keeping.next()];
        let due = due.into_iter().flatten().min();
        due.map(|due| due.saturating_since(now))
    }

    /// Stops: the snip ends as the flow says, every run goes, and the windows close.
    pub(crate) fn quit(&mut self) {
        if std::mem::replace(&mut self.stopped, true) {
            return;
        }
        self.step(In::Quit);
        self.runs.clear();
        (self.panel, self.note) = (None, None);
        self.after();
    }

    /// The time now.
    fn now(&self) -> Mono {
        self.ports.clock.now()
    }

    /// Feeds `input` to the flow and carries out its steps; an input a step brings waits its turn.
    fn step(&mut self, input: In) {
        self.inputs.push_back(input);
        if std::mem::replace(&mut self.stepping, true) {
            return;
        }
        while let Some(input) = self.inputs.pop_front() {
            for out in self.flow.on(input) {
                self.out(out);
            }
        }
        self.stepping = false;
    }

    /// After each message: the room is read for a video to record, then the view and the line are told when they
    /// changed.
    fn after(&mut self) {
        self.check_room();
        let page = self.page();
        self.told.show(page, &self.ports.bus);
        let line = self.line();
        self.told.tell_line(line, &self.ports.bus);
    }

    /// Publishes `event`; the module holds `Snip`, which every snip event needs, or its start would have failed.
    fn publish<E: Event>(&self, event: &E) {
        let _ = self.ports.bus.publish(event);
    }

    /// Shows `note` for `note_s`.
    fn show_note(&mut self, note: SnipNote) {
        let shown = Duration::from_secs(u64::from(self.settings.note_s));
        self.note = Some((note, self.now().saturating_add(shown)));
    }
}

#[cfg(test)]
mod tests;
