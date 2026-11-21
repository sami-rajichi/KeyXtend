//! The `kx-voice` actor (ADR-0042): it hands the bus's and the ports' inputs to the dictation flow and the panel, and
//! carries out what they give back, in order. The microphone's runs and the slow reads go to jobs off its thread and
//! texts to the typer's own thread, so it never waits on the worker or the keyboard.

mod ask;
mod dos;
mod held;
mod jobs;
mod keep;
mod keyed;
mod lanes;
mod line;
mod msg;
mod net;
mod offer;
mod outs;
mod paste;
mod ports;
mod presses;
mod trial;
mod typing;

pub(crate) use msg::{Back, Msg};
pub(crate) use ports::{Job, Ports};
pub use typing::TYPED_LINE;
pub(crate) use typing::{Typist, typer};

use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::time::Duration;

use kx_module_api::ports::{
    FieldChange, FieldKind, LayoutAsk, LayoutChanged, PanelButton, PanelLine, VoiceAct, VoiceClick,
    VoicePanel,
};

use crate::VoiceSettings;
use crate::flow::{Flow, In, Lend};
use crate::keys::Keys;
use crate::panel::Panel;
use held::Held;
use jobs::Jobs;
use keep::Keep;
use lanes::Lanes;
use net::Uploads;
use offer::Offer;
use paste::Pasting;
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
    pasting: Pasting,
    jobs: Jobs,
    /// The cloud keys and uploads.
    net: Uploads,
    /// The flow's inputs still to feed, so an effect never feeds it while it is fed.
    inputs: VecDeque<In>,
    stepping: bool,
    /// The panel the flow draws.
    drawn: Option<VoicePanel>,
    /// The panel's line as the keyboard was told, `None` while closed (ADR-0049).
    line: Option<PanelLine>,
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
        let keys = Keys::new(Arc::clone(&ports.shelf), hidden);
        Self {
            flow: Flow::new(Arc::clone(&ports.clock), settings.clone(), hidden),
            panel: Panel::new(&settings),
            keep: Keep::new(&settings, hidden),
            held: Held::new(settings, clips, hidden),
            ports,
            lanes: Lanes::default(),
            typing: Typing::default(),
            pasting: Pasting::default(),
            jobs: Jobs::default(),
            net: Uploads::new(keys),
            inputs: VecDeque::new(),
            stepping: false,
            drawn: None,
            line: None,
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

    /// Tells the hidden bar and the dark key, opens the store, deletes hidden rows' keys, reads the folder, and asks
    /// for the layout.
    pub(crate) fn begin(&mut self) {
        let kind = (self.ports.field.as_ref()).map_or(FieldKind::Unknown, |run| run.kind());
        self.step(In::Field(FieldChange {
            kind,
            box_id: NO_BOX,
        }));
        self.open_store();
        self.sweep_keys();
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
            Msg::Ask => {
                self.step(In::Ask);
                self.retell_line();
            }
            Msg::LineKey(key) => self.line_key(&key),
            Msg::Close => self.close_panel(),
            Msg::Layout(layout) => self.layout(layout),
            Msg::Field(change) => self.step(In::Field(change)),
            Msg::Session(note) => self.step(In::Session(note)),
            Msg::Elsewhere => self.step(In::Elsewhere),
            Msg::PressesAsk => self.retell_presses(),
            Msg::Typed(done) => self.typed(done),
            Msg::Pasted(done) => self.pasted(done),
            Msg::Taken(taken) => self.clip_taken(taken),
            Msg::Back(back) => self.answer(back),
        }
        self.after();
    }

    /// Types each text whose paste was not answered in time, then feeds the flow the hint or the fade whose time came.
    pub(crate) fn tick(&mut self) {
        let now = self.ports.clock.now();
        if !self.stopped && self.paste_due().is_some_and(|due| now >= due) {
            self.paste_tick(now);
            self.after();
        }
        if !self.stopped && self.flow.deadline().is_some_and(|due| now >= due) {
            self.step(In::Tick);
            self.after();
        }
    }

    /// How long the loop waits for a message: until a paste's answer is due or the flow's next hint or fade, else
    /// no limit.
    pub(crate) fn wait(&self) -> Option<Duration> {
        let now = self.ports.clock.now();
        let due = [self.paste_due(), self.flow.deadline()]
            .into_iter()
            .flatten()
            .min();
        due.map(|due| due.saturating_since(now))
    }

    /// Stops: the watches go, every run closes, the bar hides, and what failed to save is tried once more.
    pub(crate) fn quit(&mut self) {
        if std::mem::replace(&mut self.stopped, true) {
            return;
        }
        (self.ports.field, self.ports.session) = (None, None);
        self.step(In::Quit);
        self.drop_line();
        self.retry_saves();
        self.jobs.fetches.clear();
        self.lanes.clear();
        self.net.clear();
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
            let offer = Offer {
                stock: self.panel.stock(&facts),
                settings: &self.held.settings,
                keys: &self.net.keys,
                cloud: facts.cloud,
            };
            let outs = self.flow.on(input, &offer);
            for out in outs {
                self.out(out);
            }
        }
        self.stepping = false;
    }

    /// After each message: the flow learns what Compare holds, a held-back check may start, then the panel to draw
    /// when it changed, and the keyboard the panel's line.
    fn after(&mut self) {
        if self.stopped {
            return;
        }
        let lend = self.panel.compare().lend();
        if lend != self.lend {
            self.lend = lend;
            self.step(In::Lend(lend));
        }
        self.dequeue();
        let now = self.ports.calendar.now();
        let facts = self.held.facts(now, &self.flow);
        let view = self.panel.view(&facts);
        let line = self.panel.line(&facts);
        if view != self.drawn {
            self.drawn.clone_from(&view);
            self.step(In::Panel(view.map(Box::new)));
        }
        self.tell_line(line);
    }

    /// A click: the bar's buttons, the pointer, Try again and "Use the local model" go to the flow, the rest to the
    /// panel.
    fn act(&mut self, act: VoiceAct) {
        let flow = matches!(
            act.click,
            VoiceClick::Bar(_)
                | VoiceClick::Over(_)
                | VoiceClick::Button(PanelButton::Again)
                | VoiceClick::Local
        );
        if flow {
            return self.step(In::Act(act));
        }
        self.held.drives = self.ports.files.drives();
        self.read_keyed();
        let facts = self.held.facts(self.ports.calendar.now(), &self.flow);
        let dos = self.panel.click(&act, &facts);
        self.dos(dos);
    }

    /// Ends a running dictation or Compare's recording as Cancel does, then opens the panel, with the drives read now.
    fn open_panel(&mut self) {
        self.step(In::Opening);
        self.held.drives = self.ports.files.drives();
        self.read_keyed();
        let dos = self.panel.open();
        self.dos(dos);
    }

    /// The layout in front, for the flow and the panel; its model gets the check the card lacks.
    fn layout(&mut self, layout: LayoutChanged) {
        self.held.front = layout.clone();
        self.step(In::Layout(layout));
        self.recheck();
    }
}

#[cfg(test)]
mod tests;
