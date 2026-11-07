//! The `kx-vault` actor (design "Part 2", ADR-0038): it runs the panel over the notebook, sends fills to the keyboard
//! and takes to the clipboard history, saves after each change and tells the emails. A plain struct driven by
//! messages, plus a loop that waits for the next message, Paste's wait, the backup's or a failed save's retry.

mod backs;
mod does;

use std::sync::Arc;
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::time::Duration;

use kx_module_api::ports::{
    Calendar, ClipTaken, FillAct, FillAsk, FillClose, FillLive, FillOpen, FillPanel, FillTyped,
    PanelKey, PanelLine, Random, VaultEmails,
};
use kx_module_api::{Bus, Clock, Event, Mono, Secret};

use crate::backup::{Backup, BackupStore, Config, Done, Kept, Links};
use crate::book::{Limits, Notebook};
use crate::entry::Stamp;
use crate::fill::Filling;
use crate::find::LabelNames;
use crate::panel::{Panel, PanelConfig};
use crate::paste::Pasting;
use crate::settings::VaultSettings;
use crate::store::Store;
use backs::Opening;

/// A message for the actor, from the bus or the module.
#[derive(Debug)]
pub(crate) enum Msg {
    /// The Quick-fill key or chip toggles the panel.
    Open,
    /// The clipboard panel opened, so ours closes.
    Close,
    /// A late starter asks what it missed.
    Ask,
    /// A key for the open panel.
    Key(PanelKey),
    /// A click on the panel, or the ui's rows.
    Act(FillAct),
    /// The keyboard's answer to a fill.
    Typed(FillTyped),
    /// The clipboard history's answer to a take.
    Taken(ClipTaken),
    /// A backup job's answer.
    Done(Done),
    /// The ui's table label names, in the keyboard's language.
    Names(LabelNames),
    /// The user answered one of the module's notices: its key, and Yes or not.
    Answered { key: &'static str, yes: bool },
    /// The module stops.
    Quit,
}

/// The actor's ports, from the module's start or a test.
pub(crate) struct Ports {
    /// The bus, limited to the module's grants.
    pub bus: Bus,
    /// Monotonic time, for Paste's wait, the backup's and a failed save's retry.
    pub clock: Arc<dyn Clock>,
    /// The wall time, for the change and use stamps.
    pub calendar: Arc<dyn Calendar>,
    /// Random bytes, for new ids and Make one.
    pub random: Arc<dyn Random>,
}

/// What the actor loads its state from: the notebook and its store, and the backup's blob and its store.
pub(crate) struct Stores {
    pub store: Store,
    pub book: Notebook,
    pub backup: BackupStore,
    pub kept: Kept,
    /// The notebook loaded damaged, so the boxes are not written until Back up now or Bring back.
    pub damaged: bool,
}

/// The vault module's runtime: one per start, on its own thread.
#[allow(
    clippy::struct_excessive_bools,
    reason = "Each flag is one fact kept between messages: rows heard, a redraw, an unsaved change, its note, a stop."
)]
pub(crate) struct Actor {
    ports: Ports,
    book: Notebook,
    store: Store,
    panel: Panel,
    filling: Filling,
    pasting: Pasting,
    backup: Backup,
    tab_gap: Duration,
    /// The emails last told, so only a change that alters them tells them again.
    emails: Vec<Secret>,
    /// The line last told, so only a change tells it again.
    line: PanelLine,
    /// The ui told its rows per page, so views may show.
    rows_heard: bool,
    /// The open panel draws again after this message.
    redraw: bool,
    /// The last save failed, so the note line says so and the retry and the stop try again.
    unsaved: bool,
    /// A save was failing as the last message ended, so a change of that draws the note line again.
    told_failing: bool,
    /// When a failing save is tried again, and the wait between tries (`place_check_s`).
    retry_at: Option<Mono>,
    retry_every: Duration,
    /// What a notice's answer opens, kept over an edit until it closes.
    opening: Option<Opening>,
    /// The notebook's limits, which a Bring back keeps.
    limits: Limits,
    stopped: bool,
}

impl Actor {
    /// An actor over what `stores` loaded, which it saves from now on, with the backup over `links`.
    pub(crate) fn new(
        ports: Ports,
        links: Links,
        settings: &VaultSettings,
        stores: Stores,
    ) -> Self {
        let mut backup = Backup::new(Config::of(settings), stores.kept, stores.backup, links);
        backup.begin(ports.clock.now(), settings.memory_checks, stores.damaged);
        Self {
            ports,
            book: stores.book,
            store: stores.store,
            backup,
            panel: Panel::new(PanelConfig::of(settings)),
            filling: Filling::default(),
            pasting: Pasting::new(settings.paste_wait(), settings.field_chars),
            tab_gap: settings.tab_gap(),
            emails: Vec::new(),
            line: PanelLine::None,
            rows_heard: false,
            redraw: false,
            unsaved: false,
            told_failing: false,
            retry_at: None,
            retry_every: settings.place_check(),
            opening: None,
            limits: settings.limits(),
            stopped: false,
        }
    }

    /// The thread's loop: begins and ticks once, for a check due at the start, then serves each message and each
    /// wait until `Quit` or the module is gone.
    pub(crate) fn run(mut self, inbox: &Receiver<Msg>) {
        self.begin();
        self.tick();
        loop {
            let next = match self.wait() {
                Some(wait) => inbox.recv_timeout(wait),
                None => inbox.recv().map_err(|_| RecvTimeoutError::Disconnected),
            };
            match next {
                Ok(Msg::Quit) | Err(RecvTimeoutError::Disconnected) => return self.quit(),
                Ok(msg) => self.handle(msg),
                Err(RecvTimeoutError::Timeout) => {}
            }
            self.tick();
        }
    }

    /// Says it runs, for a keyboard that started first, and tells the emails, for a prediction that did.
    pub(crate) fn begin(&mut self) {
        self.publish(&FillLive { live: true });
        self.tell_emails(true);
    }

    /// Handles one message; nothing after the stop.
    pub(crate) fn handle(&mut self, msg: Msg) {
        if self.stopped {
            return;
        }
        match msg {
            Msg::Open => self.toggled(),
            Msg::Close => {
                let done = self.panel.close_for(FillClose);
                self.act_on(done);
            }
            Msg::Ask => {
                self.publish(&FillLive { live: true });
                self.publish(&self.panel.state(FillAsk));
                self.tell_line(true);
                self.tell_emails(true);
            }
            Msg::Key(key) => {
                let done = self.panel.key(&key, &self.book);
                self.act_on(done);
            }
            Msg::Act(act) => self.act(act),
            Msg::Typed(typed) => self.typed(typed),
            Msg::Taken(taken) => self.taken(&taken),
            Msg::Names(names) => {
                self.panel.set_names(names);
                self.redraw = true;
            }
            Msg::Done(done) => self.job_done(done),
            Msg::Answered { key, yes } => self.answered(key, yes),
            Msg::Quit => self.quit(),
        }
    }

    /// The Quick-fill key or chip: the panel opens, and the backup looks at its waiting places, or it closes.
    fn toggled(&mut self) {
        let state = self.panel.toggle(FillOpen);
        self.publish(&state);
        if state.open {
            self.redraw = true;
            self.tell_note();
            self.backup.catch_up(&self.book, self.ports.clock.now());
            self.backup.offer(&self.book);
        } else {
            self.closed();
        }
    }

    /// Ends Paste's wait when it ran out, writes the backup when due, raises a memory check due, tries a failing save
    /// again when due, then draws the open panel once if anything changed.
    pub(crate) fn tick(&mut self) {
        if self.stopped {
            return;
        }
        let now = self.ports.clock.now();
        self.backup.tick(now, &self.book);
        self.open_later();
        self.raise_check();
        if self.pasting.expired(now) {
            let done = self.panel.paste_off();
            self.act_on(done);
        }
        self.retry_saves(now);
        self.flush();
    }

    /// How long the loop waits for a message: until Paste's wait ends, the backup is due, a failing save is tried again
    /// or a memory check falls due, else for good.
    pub(crate) fn wait(&self) -> Option<Duration> {
        let now = self.ports.clock.now();
        let due = [self.pasting.due(), self.backup.next(), self.retry_at]
            .into_iter()
            .flatten()
            .min();
        let check = self
            .backup
            .check_wait(Stamp::of(&self.ports.calendar.now()));
        [due.map(|t| t.saturating_since(now)), check]
            .into_iter()
            .flatten()
            .min()
    }

    /// Stops: a failed save is tried again, the panel closes, prediction hears no emails, and the keyboard hears it
    /// stopped.
    pub(crate) fn quit(&mut self) {
        if self.stopped {
            return;
        }
        self.stopped = true;
        if self.unsaved {
            self.save();
        }
        self.panel.close();
        self.publish(&FillPanel { open: false });
        self.tell_line(false);
        self.publish(&VaultEmails { emails: Vec::new() });
        self.publish(&FillLive::OFF);
        self.backup.stop();
    }

    /// Publishes `event`; a refusal is the bus's gate on a capability the module holds, so it never happens.
    fn publish<E: Event>(&self, event: &E) {
        let _ = self.ports.bus.publish(event);
    }
}

#[cfg(test)]
mod tests;
