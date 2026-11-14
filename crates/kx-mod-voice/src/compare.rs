//! Compare models (design "Compare models"): the set text is read aloud once into a run that loads no model, then each
//! ready model writes that recording in a fresh job, one after another. Only the numbers are kept, through
//! `Scores::put`; a text shows only for a language with no set text, and is never kept. Pure: effects out, notes in.

mod act;
mod end;

pub use act::{Act, Entrant, Line, Plan};

use std::collections::BTreeMap;

use kx_module_api::Secret;
use kx_module_api::ports::{BarHint, CardId, SpeechAsk, SpeechDone, SpeechNote, SpeechStage};

use crate::VoiceSettings;
use crate::flow::{Lend, RunCall};

/// Where a run stands.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
enum Stage {
    /// The set text shows, with Start.
    #[default]
    Ready,
    /// The user reads it aloud.
    Recording,
    /// The recording's run ends (`None`), or model `at` writes it.
    Running(Option<usize>),
    /// The table shows.
    Results,
}

/// A model's turn: the model and its ask.
#[derive(Clone, Debug)]
struct Turn {
    entrant: Entrant,
    ask: SpeechAsk,
}

/// Compare's state.
#[derive(Clone, Debug, Default)]
pub struct Compare {
    stage: Stage,
    /// The last run's id; an older run's notes are dropped.
    take: u32,
    plan: Option<Plan>,
    turns: Vec<Turn>,
    heard: bool,
    pieces: BTreeMap<u16, Secret>,
    lines: Vec<Line>,
    hint: Option<BarHint>,
}

impl Compare {
    /// Starts a run of `plan`: its run opens and listens with no model; nothing with no ready model or while busy.
    pub fn start(&mut self, plan: Plan, settings: &VoiceSettings) -> Vec<Act> {
        if plan.entrants.is_empty() || self.busy() {
            return Vec::new();
        }
        let turn = |entrant: &Entrant| Turn {
            ask: settings.ask(
                Some(entrant.ready.model.clone()),
                entrant.ready.card.clone(),
            ),
            entrant: entrant.clone(),
        };
        self.turns = plan.entrants.iter().map(turn).collect();
        self.reset();
        self.plan = Some(plan);
        self.take = self.take.wrapping_add(1);
        self.stage = Stage::Recording;
        let ask = settings.ask(None, None);
        vec![Act::Open {
            take: self.take,
            ask,
        }]
    }

    /// Stop while the set text is read: the models write it in turn; with no speech heard, nothing runs.
    pub fn stop(&mut self) -> Vec<Act> {
        if self.stage != Stage::Recording {
            return Vec::new();
        }
        if !self.heard {
            return self.nothing();
        }
        self.stage = Stage::Running(None);
        vec![self.call(RunCall::Stop)]
    }

    /// The job stopped listening by itself (`record_max_s` or silence): as Stop, so silence alone runs no model.
    fn stop_self(&mut self) -> Vec<Act> {
        if !self.heard {
            return self.nothing();
        }
        self.stage = Stage::Running(None);
        Vec::new()
    }

    /// Nothing was heard: the run ends and the page says so.
    fn nothing(&mut self) -> Vec<Act> {
        self.hint = Some(BarHint::Nothing);
        self.end(true)
    }

    /// Cancel: the run stops and closes, keeping nothing.
    pub fn cancel(&mut self) -> Vec<Act> {
        if !self.busy() {
            return Vec::new();
        }
        self.end(true)
    }

    /// Back to the set text, forgetting the shown results and note.
    pub fn reset(&mut self) {
        if !self.busy() {
            self.stage = Stage::Ready;
            self.lines.clear();
            self.pieces.clear();
            (self.heard, self.hint) = (false, None);
        }
    }

    /// A note from Compare's run `take`.
    pub fn note(&mut self, take: u32, note: SpeechNote) -> Vec<Act> {
        if take != self.take || !self.busy() {
            return Vec::new();
        }
        match note {
            SpeechNote::Level(level) if self.recording() => vec![Act::Level(level)],
            SpeechNote::Heard => {
                self.heard = true;
                Vec::new()
            }
            SpeechNote::Stage(SpeechStage::Limit | SpeechStage::Silence) if self.recording() => {
                self.stop_self()
            }
            SpeechNote::Piece { index, text } => self.piece(index, text),
            SpeechNote::Done(done) => self.done(&done),
            SpeechNote::Wiped => self.lost(),
            SpeechNote::Fault(error) => self.fault(error),
            _ => Vec::new(),
        }
    }

    /// The user reads the set text now.
    #[must_use]
    pub fn recording(&self) -> bool {
        self.stage == Stage::Recording
    }

    /// Its run is open.
    #[must_use]
    pub fn busy(&self) -> bool {
        matches!(self.stage, Stage::Recording | Stage::Running(_))
    }

    /// What it holds of the microphone's run, for the flow (`In::Lend`): the bar while it records, then no dictation
    /// until it ends.
    #[must_use]
    pub fn lend(&self) -> Lend {
        match self.stage {
            Stage::Recording => Lend::Listen,
            Stage::Running(_) => Lend::Write,
            Stage::Ready | Stage::Results => Lend::Off,
        }
    }

    /// The models to run, the one writing (from 0; `None` while the recording ends) and the results so far.
    #[must_use]
    pub fn running(&self) -> Option<(Option<usize>, Vec<&Entrant>, &[Line])> {
        let Stage::Running(at) = self.stage else {
            return None;
        };
        Some((
            at,
            self.turns.iter().map(|t| &t.entrant).collect(),
            &self.lines,
        ))
    }

    /// The results shown, once the run ended.
    #[must_use]
    pub fn results(&self) -> &[Line] {
        if self.stage == Stage::Results {
            &self.lines
        } else {
            &[]
        }
    }

    /// The day and device pick of the last run's results.
    #[must_use]
    pub fn plan(&self) -> Option<&Plan> {
        self.plan.as_ref()
    }

    /// The card that wrote the results to the end, if any; `None` for the processor.
    #[must_use]
    pub fn served(&self) -> Option<&CardId> {
        self.lines.iter().find_map(|l| l.card.as_ref())
    }

    /// The note the page shows, such as Nothing heard.
    #[must_use]
    pub fn note_shown(&self) -> Option<BarHint> {
        self.hint.clone()
    }

    fn call(&self, call: RunCall) -> Act {
        Act::Run {
            take: self.take,
            call,
        }
    }

    fn piece(&mut self, index: u16, text: Secret) -> Vec<Act> {
        if matches!(self.stage, Stage::Running(Some(_))) {
            self.pieces.insert(index, text);
        }
        Vec::new()
    }

    /// The recording ended, or a model wrote it: the next model's turn, or the end.
    fn done(&mut self, done: &SpeechDone) -> Vec<Act> {
        let at = match self.stage {
            Stage::Running(Some(at)) => {
                self.score(at, done);
                at + 1
            }
            Stage::Running(None) | Stage::Recording => 0,
            Stage::Ready | Stage::Results => return Vec::new(),
        };
        self.next(at)
    }

    /// Model `at`'s turn, or the end once every model ran.
    fn next(&mut self, at: usize) -> Vec<Act> {
        let Some(turn) = self.turns.get(at) else {
            return self.finish();
        };
        let call = self.call(RunCall::Again(turn.ask.clone()));
        self.stage = Stage::Running(Some(at));
        vec![call]
    }

    /// Ends the run, cancelled first when `cancel`; the page goes back to the set text.
    fn end(&mut self, cancel: bool) -> Vec<Act> {
        let mut acts = Vec::new();
        if cancel {
            acts.push(self.call(RunCall::Cancel));
        }
        acts.push(self.call(RunCall::Close));
        self.pieces.clear();
        self.stage = Stage::Ready;
        acts
    }
}

#[cfg(test)]
mod tests;
