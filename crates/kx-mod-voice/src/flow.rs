//! The dictation flow (design "How a dictation runs"): Idle, Listening, Transcribing, Text and Note, on an injected
//! `Clock`. Pure: inputs in, effects out; the actor carries them out on the ports and the bus.

mod acts;
mod io;
mod lend;
mod notes;
mod take;
mod text;
mod view;

pub use io::{Count, In, Kept, Lack, Out, Ready, RunCall, Supply};
pub use lend::Lend;
pub(crate) use notes::mic_hint;
pub(crate) use take::suffixed;

use std::sync::Arc;

use kx_module_api::ports::{BarButton, BarHint, FieldChange, FieldKind, VoicePanel, VoiceText};
use kx_module_api::{Clock, Mono};

use crate::VoiceSettings;
use take::{Lang, Take};
use view::Told;

/// Where the flow is; a take lives beside it from Start until its bar closes.
#[derive(Clone, Debug)]
enum State {
    /// No bar.
    Idle,
    /// The microphone listens since `since`; `nothing` once `nothing_s` passed with no speech.
    Listening {
        /// When listening began.
        since: Mono,
        /// The nothing-heard hint shows.
        nothing: bool,
    },
    /// The last piece is being written.
    Transcribing,
    /// The text, typed or kept, with the line under it.
    Text {
        /// The text, or dots for a password box.
        text: VoiceText,
        /// The line under it.
        hint: Option<BarHint>,
        /// The last Copy was refused, so its line shows in place of `hint` until a Copy goes through.
        unclipped: bool,
    },
    /// A note about keyboard language `iso`, with the action it names.
    Note {
        /// What happened.
        hint: BarHint,
        /// The action it names.
        act: Option<BarButton>,
        /// The language, which Download fetches a model for.
        iso: String,
    },
}

/// The state's kind, which the bar's buttons go by.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Phase {
    Idle,
    Listening,
    Transcribing,
    Text,
    Note,
}

/// The dictation state machine.
pub struct Flow {
    clock: Arc<dyn Clock>,
    settings: VoiceSettings,
    /// The run's saves would be hidden (ADR-0037), so nothing is kept.
    hidden: bool,
    state: State,
    /// The dictation from Start until its bar closes.
    take: Option<Take>,
    /// The last take's id.
    takes: u32,
    /// The last `TextAsk`'s round.
    asks: u32,
    /// The language the keyboard shows now.
    front: Lang,
    /// The focused box.
    field: FieldChange,
    panel: Option<VoicePanel>,
    /// The pointer is over the bar.
    over: bool,
    /// When the Text or Note bar showed, or the pointer or the panel last left it.
    shown_at: Mono,
    /// When the key was last counted.
    key_at: Option<Mono>,
    /// The microphone is open.
    lit: bool,
    /// Compare models holds the microphone's run, so no dictation starts.
    lent: Option<lend::Lent>,
    told: Told,
}

impl Flow {
    /// A flow with no bar, reading time from `clock`; `hidden` is a run whose saves would be hidden (ADR-0037).
    #[must_use]
    pub fn new(clock: Arc<dyn Clock>, settings: VoiceSettings, hidden: bool) -> Self {
        Self {
            clock,
            settings,
            hidden,
            state: State::Idle,
            take: None,
            takes: 0,
            asks: 0,
            front: Lang::unknown(),
            field: FieldChange {
                kind: FieldKind::Unknown,
                box_id: 0,
            },
            panel: None,
            over: false,
            shown_at: Mono::default(),
            key_at: None,
            lit: false,
            lent: None,
            told: Told::default(),
        }
    }

    /// Takes one input and gives the effects to carry out, in order; `supply` names the model a language gets.
    pub fn on(&mut self, input: In, supply: &dyn Supply) -> Vec<Out> {
        let mut out = Vec::new();
        match input {
            In::Key => self.key(supply, &mut out),
            In::Act(act) => self.act(&act, supply, &mut out),
            In::Ask => self.retell(&mut out),
            In::Panel(panel) => self.set_panel(panel),
            In::Opening => self.opening(&mut out),
            In::Note { take, note } => self.note(take, note, &mut out),
            In::Field(change) => self.field(change),
            In::Elsewhere => self.elsewhere(),
            In::Session(note) if note.away() => self.stop_all(&mut out),
            In::Session(_) => {}
            In::Layout(layout) => self.front = Lang::from(&layout),
            In::Typed(done) => self.typed(done),
            In::Copied(ok) => self.copied(ok),
            In::Settings(settings) => self.set_settings(*settings, &mut out),
            In::Tick => self.tick(&mut out),
            In::Lend(lend) => self.lend(lend, &mut out),
            In::Quit => {
                self.stop_all(&mut out);
                self.panel = None;
            }
        }
        // A hidden bar has no pointer over it, so the next bar still fades.
        self.over &= self.phase() != Phase::Idle;
        self.tell(&mut out);
        out
    }

    /// When the next hint or fade is due, for `In::Tick`.
    #[must_use]
    pub fn deadline(&self) -> Option<Mono> {
        let heard = self.take.as_ref().is_some_and(|t| t.heard);
        match &self.state {
            State::Listening {
                since,
                nothing: false,
            } if !heard => Some(since.saturating_add(self.settings.nothing())),
            State::Text { .. } | State::Note { .. } if !self.over && self.panel.is_none() => {
                Some(self.shown_at.saturating_add(self.settings.done()))
            }
            _ => None,
        }
    }

    /// Try again can act: a text is on the bar and its recording is still held.
    #[must_use]
    pub fn can_again(&self) -> bool {
        self.phase() == Phase::Text && self.held()
    }

    /// A dictation listens or writes, so Compare waits (`Facts::dictating`).
    #[must_use]
    pub fn dictating(&self) -> bool {
        self.busy()
    }

    /// The box told last is no box, so a written text would only show.
    #[must_use]
    pub fn no_box(&self) -> bool {
        self.field.kind == FieldKind::None
    }

    /// Every run stops: the dictation's bar closes and Compare's run is cancelled.
    fn stop_all(&mut self, out: &mut Vec<Out>) {
        self.end_lend(out);
        self.close(out);
    }

    /// The state's kind.
    fn phase(&self) -> Phase {
        match self.state {
            State::Idle => Phase::Idle,
            State::Listening { .. } => Phase::Listening,
            State::Transcribing => Phase::Transcribing,
            State::Text { .. } => Phase::Text,
            State::Note { .. } => Phase::Note,
        }
    }

    /// The microphone listens or the last piece is being written.
    fn busy(&self) -> bool {
        matches!(self.phase(), Phase::Listening | Phase::Transcribing)
    }

    /// The take's run is open and still holds the recording.
    fn held(&self) -> bool {
        self.take.as_ref().is_some_and(|t| t.open && t.held)
    }

    fn now(&self) -> Mono {
        self.clock.now()
    }

    /// The hint or the fade whose time came.
    fn tick(&mut self, out: &mut Vec<Out>) {
        if self.deadline().is_none_or(|due| self.now() < due) {
            return;
        }
        if let State::Listening { nothing, .. } = &mut self.state {
            *nothing = true;
        } else {
            self.close(out);
        }
    }

    /// The panel shown; `done_ms` starts again once it closes.
    fn set_panel(&mut self, panel: Option<Box<VoicePanel>>) {
        if self.panel.is_some() && panel.is_none() {
            self.shown_at = self.now();
        }
        self.panel = panel.map(|p| *p);
    }

    /// New settings; the feature turned off closes the bar and cancels Compare's run.
    fn set_settings(&mut self, settings: VoiceSettings, out: &mut Vec<Out>) {
        self.settings = settings;
        if !self.settings.on {
            self.stop_all(out);
        }
    }
}

#[cfg(test)]
mod tests;
