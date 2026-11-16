//! The microphone's runs, the dictation's and Compare's: each opens and takes its calls in jobs off the actor, one call
//! at a time in order, and comes back to its lane between calls. Its notes and every answer reach the inbox.

use std::collections::{HashMap, VecDeque};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;

use kx_module_api::SecretBytes;
use kx_module_api::ports::{SpeechAsk, SpeechError, SpeechNote, SpeechRun};

use super::{Actor, Back, Job, Msg};
use crate::flow::{In, RunCall};

/// Whose run it is.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum Owner {
    /// The dictation flow's.
    Flow,
    /// Compare models'.
    Compare,
}

/// A run, by its owner and the take that names it.
pub(crate) type Lane = (Owner, u32);

/// A run on the platform's side, which prints nothing of itself; dropping it closes it.
pub(crate) struct Run(Box<dyn SpeechRun>);

/// What a call gave back.
pub(crate) enum Answer {
    /// It was done.
    Done,
    /// The recording, for Keep recordings.
    Sound(Vec<SecretBytes>),
}

/// One run's place: the run while no job holds it, and the calls waiting.
#[derive(Default)]
struct Slot {
    run: Option<Run>,
    /// A job holds the run, opening it or calling it.
    busy: bool,
    calls: VecDeque<RunCall>,
}

/// The open runs.
#[derive(Default)]
pub(super) struct Lanes(HashMap<Lane, Slot>);

impl Lanes {
    /// Forgets every run; those here close as they drop, and those in a job close when it ends.
    pub(super) fn clear(&mut self) {
        self.0.clear();
    }
}

impl Actor {
    /// Opens run `lane` as `ask` says, in a job; its notes come to the inbox as told.
    pub(super) fn open_run(&mut self, lane: Lane, ask: SpeechAsk) {
        let slot = Slot {
            busy: true,
            ..Slot::default()
        };
        self.lanes.0.insert(lane, slot);
        let (speech, back) = (Arc::clone(&self.ports.speech), self.ports.back.clone());
        let notes = back.clone();
        let tell = Box::new(move |note: SpeechNote| {
            drop(notes.send(Msg::Back(Back::Note { lane, note })));
        });
        let job: Job = Box::new(move || {
            let opened = catch_unwind(AssertUnwindSafe(|| speech.open(ask, tell)));
            let run = opened.unwrap_or(Err(SpeechError::Internal)).map(Run);
            // Only a stopped actor misses it, and the run then closes as it drops.
            let _ = back.send(Msg::Back(Back::Opened { lane, run }));
        });
        if !(self.ports.spawn)(job) {
            self.opened(lane, Err(SpeechError::Internal));
        }
    }

    /// Calls run `lane` after the calls before it; the recording is never fetched while it would not be kept.
    pub(super) fn call_run(&mut self, lane: Lane, call: RunCall) {
        if call == RunCall::Keep && !self.keep.rules.voice {
            return;
        }
        if let Some(slot) = self.lanes.0.get_mut(&lane) {
            slot.calls.push_back(call);
            self.next_call(lane);
        }
    }

    /// Run `lane` opened: its calls go on; a refusal tells its owner the fault.
    pub(super) fn opened(&mut self, lane: Lane, run: Result<Run, SpeechError>) {
        let Some(slot) = self.lanes.0.get_mut(&lane) else {
            return;
        };
        match run {
            Ok(run) => {
                (slot.run, slot.busy) = (Some(run), false);
                self.next_call(lane);
            }
            Err(error) => {
                self.lanes.0.remove(&lane);
                self.run_fault(lane, error);
            }
        }
    }

    /// A call on run `lane` ended: the run is back for the next one, or gone once closed.
    pub(super) fn called(
        &mut self,
        lane: Lane,
        run: Option<Run>,
        answer: Result<Answer, SpeechError>,
    ) {
        let Some(slot) = self.lanes.0.get_mut(&lane) else {
            return;
        };
        slot.busy = false;
        match run {
            Some(run) => slot.run = Some(run),
            None => drop(self.lanes.0.remove(&lane)),
        }
        match answer {
            Ok(Answer::Sound(pieces)) if lane.0 == Owner::Flow => self.keep_sound(lane.1, &pieces),
            Ok(_) => {}
            Err(error) => self.run_fault(lane, error),
        }
        self.next_call(lane);
    }

    /// A note from run `lane`, for its owner; the flow's written text comes after a look at the box when none was told.
    pub(super) fn noted(&mut self, (owner, take): Lane, note: SpeechNote) {
        match owner {
            Owner::Flow => {
                if matches!(note, SpeechNote::Done(_)) {
                    self.look();
                }
                self.step(In::Note { take, note });
            }
            Owner::Compare => {
                let dos = self.panel.compare_note(take, note);
                self.dos(dos);
            }
        }
    }

    /// Starts run `lane`'s next call in a job, while no job holds it.
    fn next_call(&mut self, lane: Lane) {
        let Some(slot) = self.lanes.0.get_mut(&lane) else {
            return;
        };
        if slot.busy || slot.calls.is_empty() {
            return;
        }
        let (Some(run), Some(call)) = (slot.run.take(), slot.calls.pop_front()) else {
            return;
        };
        slot.busy = true;
        let back = self.ports.back.clone();
        let job: Job = Box::new(move || {
            let (run, answer) = call_on(run, call);
            // Only a stopped actor misses it, and the run then closes as it drops.
            let _ = back.send(Msg::Back(Back::Called { lane, run, answer }));
        });
        if !(self.ports.spawn)(job) {
            self.called(lane, None, Err(SpeechError::Internal));
        }
    }

    /// A port error on run `lane`, logged by kind and told to its owner as the run's fault.
    fn run_fault(&mut self, lane: Lane, error: SpeechError) {
        tracing::warn!(%error, "a voice run's call failed");
        self.noted(lane, SpeechNote::Fault(error));
    }
}

/// Carries out `call` on `run`; the run comes back unless it closed, and a panic in the port counts as `Internal`.
fn call_on(mut run: Run, call: RunCall) -> (Option<Run>, Result<Answer, SpeechError>) {
    let close = call == RunCall::Close;
    let done = |result: Result<(), SpeechError>| result.map(|()| Answer::Done);
    let caught = catch_unwind(AssertUnwindSafe(|| match call {
        RunCall::Stop => done(run.0.stop()),
        RunCall::Cancel => done(run.0.cancel()),
        RunCall::Again(ask) => done(run.0.again(ask)),
        RunCall::Keep => run.0.keep().map(Answer::Sound),
        RunCall::Close => {
            run.0.close();
            Ok(Answer::Done)
        }
    }));
    let Ok(answer) = caught else {
        shut(run);
        return (None, Err(SpeechError::Internal));
    };
    if close {
        shut(run);
        return (None, answer);
    }
    (Some(run), answer)
}

/// Drops `run`; a port that breaks again as it closes is caught, so the job still answers its lane.
fn shut(run: Run) {
    let _ = catch_unwind(AssertUnwindSafe(|| drop(run)));
}
