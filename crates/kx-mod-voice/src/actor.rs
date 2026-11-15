//! The `kx-voice` actor (ADR-0042): it hands the bus's and the ports' inputs to the dictation flow and the panel, and
//! carries out what they give back, in order. The microphone's runs and the slow reads go to jobs off its thread and
//! texts to the typer's own thread, so it never waits on the worker or the keyboard.

mod dos;
mod held;
mod jobs;
mod keep;
mod lanes;
mod msg;
mod outs;
mod ports;
mod typing;

pub(crate) use msg::{Back, Msg};
pub(crate) use ports::{Job, Ports};
pub(crate) use typing::{Typist, typer};

use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::time::Duration;

use kx_module_api::ports::{
    DevicePick, FieldChange, FieldKind, LayoutAsk, LayoutChanged, PanelButton, PressAsker,
    PressesWanted, VoiceAct, VoiceClick, VoicePanel,
};
use kx_module_api::{Event, Notice};

use crate::flow::{Flow, In, Lend};
use crate::panel::Panel;
use crate::{ID, VoiceSettings};
use held::Held;
use jobs::Jobs;
use keep::Keep;
use lanes::Lanes;
use typing::Typing;

/// The box the first look names, before the watch tells one.
const NO_BOX: u64 = 0;

/// The voice module's runtime: one per start, on its own thread.
pub(crate) struct Actor {
    ports: Ports,
    held: Held,
    flow: Flow,
    panel: Panel,
    keep: Keep,
    lanes: Lanes,
    typing: Typing,
    jobs: Jobs,
    /// The flow's inputs still to feed, so an effect never feeds it while it is fed.
    inputs: VecDeque<In>,
    stepping: bool,
    /// The panel the flow draws.
    drawn: Option<VoicePanel>,
    /// What Compare holds of the microphone, as the flow knows it.
    lend: Lend,
    /// Presses are wanted, as Undo insert can act.
    presses: bool,
    stopped: bool,
}

impl Actor {
    /// An actor following `settings`; `hidden` is a run whose saves would be hidden (ADR-0037).
    pub(crate) fn new(ports: Ports, settings: VoiceSettings, hidden: bool) -> Self {
        let clips = ports.speech.clips();
        Self {
            flow: Flow::new(Arc::clone(&ports.clock), settings.clone(), hidden),
            panel: Panel::new(&settings),
            keep: Keep::new(&settings, hidden),
            held: Held::new(settings, clips),
            ports,
            lanes: Lanes::default(),
            typing: Typing::default(),
            jobs: Jobs::default(),
            inputs: VecDeque::new(),
            stepping: false,
            drawn: None,
            lend: Lend::Off,
            presses: false,
            stopped: false,
        }
    }

    /// `[voice]` as the actor follows it.
    #[cfg(test)]
    pub(crate) fn settings(&self) -> &VoiceSettings {
        &self.held.settings
    }

    /// The thread's loop: begins, then serves each message and the flow's deadlines until `Quit`.
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

    /// Tells the hidden bar and the dark key, opens the store, reads the folder, and asks for the layout.
    pub(crate) fn begin(&mut self) {
        let kind = (self.ports.field.as_ref()).map_or(FieldKind::Unknown, |run| run.kind());
        self.step(In::Field(FieldChange {
            kind,
            box_id: NO_BOX,
        }));
        self.open_store();
        self.scan();
        if self.card_could_serve() {
            self.read_devices();
        }
        self.publish(&LayoutAsk);
        self.after();
    }

    /// Handles one message; nothing after the stop.
    pub(crate) fn handle(&mut self, msg: Msg) {
        if self.stopped {
            return;
        }
        match msg {
            Msg::Quit => return self.quit(),
            Msg::Key => self.step(In::Key),
            Msg::Act(act) => self.act(act),
            Msg::Open => self.open_panel(),
            Msg::Ask => self.step(In::Ask),
            Msg::Layout(layout) => self.layout(layout),
            Msg::Field(change) => self.step(In::Field(change)),
            Msg::Session(note) => self.step(In::Session(note)),
            Msg::Elsewhere => self.step(In::Elsewhere),
            Msg::PressesAsk => self.retell_presses(),
            Msg::Typed(done) => self.typed(done),
            Msg::Back(back) => self.answer(back),
        }
        self.after();
    }

    /// Feeds the flow the hint or the fade whose time came.
    pub(crate) fn tick(&mut self) {
        let now = self.ports.clock.now();
        if !self.stopped && self.flow.deadline().is_some_and(|due| now >= due) {
            self.step(In::Tick);
            self.after();
        }
    }

    /// How long the loop waits for a message: until the flow's next hint or fade, else no limit.
    pub(crate) fn wait(&self) -> Option<Duration> {
        let now = self.ports.clock.now();
        self.flow.deadline().map(|due| due.saturating_since(now))
    }

    /// Stops: the watches go, every run closes, the bar hides, and what failed to save is tried once more.
    pub(crate) fn quit(&mut self) {
        if std::mem::replace(&mut self.stopped, true) {
            return;
        }
        (self.ports.field, self.ports.session) = (None, None);
        self.step(In::Quit);
        self.retry_saves();
        self.jobs.fetches.clear();
        self.lanes.clear();
    }

    /// Feeds `input` to the flow and carries out its effects; an input an effect brings waits its turn.
    fn step(&mut self, input: In) {
        self.inputs.push_back(input);
        if std::mem::replace(&mut self.stepping, true) {
            return;
        }
        while let Some(input) = self.inputs.pop_front() {
            let now = self.ports.calendar.now();
            let facts = self.held.facts(now, &self.flow);
            let stock = self.panel.stock(&facts);
            let outs = self.flow.on(input, &stock);
            for out in outs {
                self.out(out);
            }
        }
        self.stepping = false;
    }

    /// After each message: the flow learns what Compare holds, then the panel to draw when it changed.
    fn after(&mut self) {
        if self.stopped {
            return;
        }
        let lend = self.panel.compare().lend();
        if lend != self.lend {
            self.lend = lend;
            self.step(In::Lend(lend));
        }
        let now = self.ports.calendar.now();
        let facts = self.held.facts(now, &self.flow);
        let view = self.panel.view(&facts);
        if view != self.drawn {
            self.drawn.clone_from(&view);
            self.step(In::Panel(view.map(Box::new)));
        }
    }

    /// A click: the bar's buttons, the pointer and Try again go to the flow, the rest to the panel.
    fn act(&mut self, act: VoiceAct) {
        let flow = matches!(
            act.click,
            VoiceClick::Bar(_) | VoiceClick::Over(_) | VoiceClick::Button(PanelButton::Again)
        );
        if flow {
            return self.step(In::Act(act));
        }
        self.held.drives = self.ports.files.drives();
        let facts = self.held.facts(self.ports.calendar.now(), &self.flow);
        let dos = self.panel.click(&act, &facts);
        self.dos(dos);
    }

    /// Opens the panel on its home list, with the drives read now.
    fn open_panel(&mut self) {
        self.held.drives = self.ports.files.drives();
        let dos = self.panel.open();
        self.dos(dos);
    }

    /// The layout in front, for the flow and the panel.
    fn layout(&mut self, layout: LayoutChanged) {
        self.held.front = layout.clone();
        self.step(In::Layout(layout));
    }

    /// A card may serve a dictation now: one is picked, or Automatic has a check to go by.
    fn card_could_serve(&self) -> bool {
        match self.held.settings.device() {
            DevicePick::Cpu => false,
            DevicePick::Card(_) => true,
            DevicePick::Auto => !self.held.checks.rows().is_empty(),
        }
    }

    /// Tells the mouse module whether presses are wanted, when that changed.
    fn want_presses(&mut self, on: bool) {
        if std::mem::replace(&mut self.presses, on) != on {
            self.publish(&PressesWanted {
                asker: PressAsker::Voice,
                on,
            });
        }
    }

    /// A late mouse module hears the wish again, while there is one.
    fn retell_presses(&self) {
        if self.presses {
            self.publish(&PressesWanted {
                asker: PressAsker::Voice,
                on: true,
            });
        }
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
