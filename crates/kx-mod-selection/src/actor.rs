//! The `kx-selection` actor (ADR-0041): it looks at the focused box when the watch, a release or an act asks, feeds
//! the strip machine, sends the edit keys through the keyboard and opens Translate's page after the user's one yes.

mod acts;
mod looks;
mod translate;

use std::sync::Arc;
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender};
use std::time::Duration;

use kx_module_api::ports::{
    AssistAsk, Board, BoardPlaceAsk, ChipAct, ChipsView, EditDone, Piece, PressesWanted, SelChip,
    SelRun, TypingModeAsk, WebLink,
};
use kx_module_api::{Bus, Clock, Event, Mono, Notice, SettingsHandle};

use crate::link::{Site, sites};
use crate::manifest::{ASKER, ID};
use crate::settings::SelSettings;
use crate::strip::{Input, Pick, Strip};

/// One open of a link, run apart from the actor.
pub(crate) type Job = Box<dyn FnOnce() + Send>;

/// Starts a job on its own thread, or says false when it cannot; tests run it in place.
pub(crate) type Spawn = fn(Job) -> bool;

/// A message for the actor, from the watch, the bus, a link's open or the module.
#[derive(Debug)]
pub(crate) enum Msg {
    /// The watch told a change: look now, on this thread.
    Wake,
    /// A button went down on our windows (true) or in another app.
    Pressed(bool),
    /// A button came up on our windows (true) or in another app.
    Released(bool),
    /// The keyboard typed a piece: a break (true), such as our Esc, or another; its text never comes here.
    Typed(bool),
    /// One of our panels' lines is aimed at (true) or let go.
    Aimed(bool),
    /// Grab carries a press (true), or not.
    Carrying(bool),
    /// The keyboard moved.
    Board(Board),
    /// A keycap was clicked.
    Act(ChipAct),
    /// The keyboard answered an edit.
    Done(EditDone),
    /// The user answered `notice`, one of the module's own.
    Answered {
        /// The notice answered, which must equal the question raised.
        notice: Notice,
        /// The user chose Yes.
        yes: bool,
    },
    /// The mouse module asks whether presses are wanted.
    PressesAsk,
    /// A link's open for the strip of `round` ended.
    Opened {
        /// The round of the strip clicked.
        round: u32,
        /// The page opened.
        ok: bool,
    },
    /// The module stops.
    Quit,
}

impl Msg {
    /// The message for a typed `piece`, which keeps only whether it was a break.
    pub(crate) fn typed(piece: &Piece) -> Self {
        Self::Typed(matches!(piece, Piece::Break))
    }
}

/// The actor's ports, from the module's start or a test.
pub(crate) struct Ports {
    /// The bus, limited to the module's grants.
    pub bus: Bus,
    /// Monotonic time, for the caret's Paste and the consent's wait.
    pub clock: Arc<dyn Clock>,
    /// The running watch, dropped at the stop.
    pub run: Option<Box<dyn SelRun>>,
    /// Opens Translate's page in the browser.
    pub web: Arc<dyn WebLink>,
    /// The module's own settings, for the consent.
    pub handle: SettingsHandle,
    /// The actor's own inbox, where each open sends its end.
    pub back: Sender<Msg>,
    /// Starts each open off the actor.
    pub spawn: Spawn,
}

/// The act in flight: an edit asked of the keyboard, or a link opening.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct Busy {
    /// The round of the strip clicked.
    round: u32,
    /// The keycap clicked.
    chip: SelChip,
}

/// Translate's question, raised and not yet answered.
#[derive(Clone, Debug)]
struct Asking {
    /// The exact notice raised, which only its own answer equals.
    notice: Notice,
    /// When it was raised.
    at: Mono,
    /// The round of the strip clicked.
    round: u32,
    /// What the click acts on.
    pick: Pick,
}

/// The selection module's runtime: one per start, on its own thread.
#[allow(
    clippy::struct_excessive_bools,
    reason = "Each flag is one fact kept between messages: a click, a carry, the edit's own break, a stop."
)]
pub(crate) struct Actor {
    ports: Ports,
    settings: SelSettings,
    strip: Strip,
    /// The translate site the settings name, when the table loaded.
    site: Option<&'static Site>,
    /// The site's code for the target language.
    code: String,
    /// Where the last press landed (true on our windows), until its release.
    press: Option<bool>,
    /// The next look comes just after a press and release that began in another app.
    click: bool,
    /// Grab carries a press, so a release is no look.
    carrying: bool,
    /// The keyboard's one break after our own edit is still to come.
    swallow: bool,
    busy: Option<Busy>,
    asking: Option<Asking>,
    stopped: bool,
}

impl Actor {
    /// An actor following `settings`, whose Translate targets the system's display language `os` when
    /// `translate_to` is empty.
    pub(crate) fn new(ports: Ports, settings: &SelSettings, os: &str) -> Self {
        let site = sites()
            .ok()
            .and_then(|table| table.get(&settings.translate_site));
        let code = site.map(|s| s.target(&settings.translate_to, os));
        Self {
            ports,
            strip: Strip::new(settings.strip_config(site.is_some())),
            settings: settings.clone(),
            site,
            code: code.unwrap_or_default(),
            press: None,
            click: false,
            carrying: false,
            swallow: false,
            busy: None,
            asking: None,
            stopped: false,
        }
    }

    /// The thread's loop: begins, then serves each message and the caret Paste's end until `Quit`.
    pub(crate) fn run(mut self, inbox: &Receiver<Msg>) {
        self.begin();
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

    /// Wants presses, and asks for the keyboard's place, Grab's state and the aimed line, for a late start.
    pub(crate) fn begin(&mut self) {
        self.tell_presses(true);
        self.publish(&BoardPlaceAsk);
        self.publish(&AssistAsk);
        // The keyboard tells the aimed line again with its typing mode.
        self.publish(&TypingModeAsk);
    }

    /// Handles one message; nothing after the stop.
    pub(crate) fn handle(&mut self, msg: Msg) {
        if self.stopped {
            return;
        }
        match msg {
            Msg::Wake => self.look(),
            Msg::Pressed(ours) => self.pressed(ours),
            Msg::Released(ours) => self.released(ours),
            Msg::Typed(broke) => self.typed(broke),
            Msg::Aimed(on) => self.step(Input::Aimed(on)),
            Msg::Carrying(on) => self.carrying = on,
            Msg::Board(board) => self.step(Input::Board(board)),
            Msg::Act(act) => self.act(act),
            Msg::Done(done) => self.done(done),
            Msg::Answered { notice, yes } => self.answered(&notice, yes),
            Msg::PressesAsk => self.tell_presses(true),
            Msg::Opened { round, ok } => self.opened(round, ok),
            Msg::Quit => self.quit(),
        }
    }

    /// Ends the caret's Paste once its time is up.
    pub(crate) fn tick(&mut self) {
        let now = self.ports.clock.now();
        if !self.stopped && self.strip.ends().is_some_and(|end| now >= end) {
            self.step(Input::Tick);
        }
    }

    /// How long the loop waits for a message: until the caret's Paste ends, else no limit.
    pub(crate) fn wait(&self) -> Option<Duration> {
        let now = self.ports.clock.now();
        self.strip.ends().map(|end| end.saturating_since(now))
    }

    /// Stops: the watch goes first, then the strip hides and presses are no longer wanted.
    pub(crate) fn quit(&mut self) {
        if std::mem::replace(&mut self.stopped, true) {
            return;
        }
        self.ports.run = None;
        self.publish(&ChipsView::Hidden);
        self.tell_presses(false);
    }

    /// Feeds `input` to the strip and tells the view when it changed.
    fn step(&mut self, input: Input) {
        let now = self.ports.clock.now();
        if let Some(view) = self.strip.step(input, now) {
            self.publish(&view);
        }
    }

    /// Hides the strip of `round` after a refused act, unless another strip shows by now.
    fn refuse(&mut self, round: u32) {
        if matches!(self.strip.view(), ChipsView::Shown { round: r, .. } if r == round) {
            self.step(Input::Lost);
        }
    }

    /// Tells the mouse module whether presses are wanted.
    fn tell_presses(&self, on: bool) {
        self.publish(&PressesWanted { asker: ASKER, on });
    }

    /// Tells the user `key`.
    fn say(&self, key: &'static str) {
        self.publish(&Notice::about(key, ID));
    }

    /// Publishes `event`; a refusal is the bus's gate on a capability the module holds, so it never happens.
    fn publish<E: Event>(&self, event: &E) {
        let _ = self.ports.bus.publish(event);
    }
}

#[cfg(test)]
mod tests;
