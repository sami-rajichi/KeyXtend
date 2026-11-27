//! One run's notes in the job's order: the first says it started or why not, a switch's answer settles that switch, a
//! fault or a lost job ends the run and deletes its broken video (edge 13), and nothing is told once cancelled.

use super::{error, retry, scrap};
use crate::worker::Told;
use crate::worker::sound_fault;
use kx_ipc::{RecordFault, RecordNote as Note, Switch};
use kx_module_api::ports::{
    RecordDone, RecordEnd, RecordError, RecordNote, SnipFile, SnipFolder, Source,
};
use kx_module_api::{Clock, Mono};
use std::sync::Arc;
use std::sync::mpsc::Sender;
use std::time::Duration;

/// Where a start's or a switch's answer goes.
pub(super) type Answer = Sender<Result<(), RecordError>>;

/// Where a run stands.
enum Phase {
    /// Its start waits for the job's first note.
    Starting(Answer),
    /// It records.
    Live,
    /// It told its last note, lost its job, or was left.
    Ended,
}

/// A switch waiting for its answer since `since`.
struct Waiting {
    source: Source,
    since: Mono,
    answer: Answer,
}

/// Whether a run's end kept its video, as its owner knows it.
#[derive(Copy, Clone, PartialEq, Eq)]
enum Kept {
    /// No end kept it.
    No,
    /// Its end kept it.
    Yes,
    /// Its end kept it, but its owner never heard, so a Cancel still takes it back.
    Unheard,
}

/// One run's state, which its stream's notes and its owner's calls both change.
pub(super) struct Route {
    phase: Phase,
    waiting: Option<Waiting>,
    cancelled: bool,
    kept: Kept,
    /// The folder refused to delete its video, as its job still held it; leaving tries again.
    held: bool,
    file: SnipFile,
    folder: Arc<dyn SnipFolder>,
    clock: Arc<dyn Clock>,
    /// How long a switch waits for its answer by the clock.
    switch: Duration,
}

impl Route {
    /// A run recording into `file` whose start waits; `started` hears whether it did, and a switch waits `switch`.
    pub(super) fn new(
        file: SnipFile,
        folder: Arc<dyn SnipFolder>,
        clock: Arc<dyn Clock>,
        started: Answer,
        switch: Duration,
    ) -> Self {
        Self {
            phase: Phase::Starting(started),
            waiting: None,
            cancelled: false,
            kept: Kept::No,
            held: false,
            file,
            folder,
            clock,
            switch,
        }
    }

    /// Takes what the stream told; the note to tell the run's callback, if any.
    pub(super) fn heard(&mut self, told: Told<Note>) -> Option<RecordNote> {
        match told {
            Told::Note(note) => self.note(&note),
            Told::Lost(_) | Told::Idle => self.lose(),
        }
    }

    /// Waits for `source`'s switch to be answered at `answer`, settling an older one as gone; false once it ended.
    pub(super) fn wait(&mut self, source: Source, answer: Answer) -> bool {
        let live = matches!(self.phase, Phase::Live);
        if live {
            self.settle(Err(RecordError::Gone));
            let since = self.clock.now();
            self.waiting = Some(Waiting {
                source,
                since,
                answer,
            });
        }
        live
    }

    /// Marks the run cancelled, its owner's ear already closed; true when its job is still to be told. An end its
    /// owner never heard loses its video, as Cancel wins.
    pub(super) fn cancel(&mut self) -> bool {
        let tell = matches!(self.phase, Phase::Live) && !self.cancelled;
        self.cancelled = true;
        self.take_back();
        tell
    }

    /// The note just taken never reached the owner, whose ear was closed; a cancelled run loses the video it kept.
    pub(super) fn unheard(&mut self) {
        if self.kept == Kept::Yes {
            self.kept = Kept::Unheard;
        }
        self.take_back();
    }

    /// True once the run told its last note, lost its job, or was left.
    pub(super) const fn ended(&self) -> bool {
        matches!(self.phase, Phase::Ended)
    }

    /// The run's job was told goodbye and ended: a cancelled video it kept goes, one the folder held is tried again,
    /// and nothing more is heard.
    pub(super) fn leave(&mut self) {
        if self.held || (self.cancelled && !self.ended()) {
            self.held = false;
            retry(&*self.folder, &self.file);
        }
        self.phase = Phase::Ended;
        self.settle(Err(RecordError::Gone));
    }

    /// Takes one note of the job.
    fn note(&mut self, note: &Note) -> Option<RecordNote> {
        match self.phase {
            Phase::Starting(_) => return self.first(note),
            Phase::Ended => return None,
            Phase::Live => self.overdue(),
        }
        match *note {
            Note::Time { ms, .. } => self.tell(RecordNote::Time { ms }),
            Note::Switched { switch, .. } => self.answer(source(switch), Ok(())),
            Note::Fault { fault, .. } if sound_fault(fault) => {
                self.answer(heard(fault), Err(error(fault)))
            }
            Note::Fault { fault, .. } => self.end(RecordNote::Fault(error(fault)), true),
            Note::End { length_ms, end, .. } => {
                let end = kept(end);
                let done = RecordNote::Done(RecordDone { length_ms, end });
                if !self.cancelled {
                    self.kept = Kept::Yes;
                }
                self.end(done, self.cancelled)
            }
            Note::Lost { switch, .. } => self.tell(RecordNote::Lost(source(switch))),
        }
    }

    /// Takes the job's first note, which says the run started or why not; a start that failed leaves no video.
    fn first(&mut self, note: &Note) -> Option<RecordNote> {
        let Phase::Starting(started) = std::mem::replace(&mut self.phase, Phase::Live) else {
            return None;
        };
        let (answer, told) = match *note {
            Note::Time { ms, .. } => (Ok(()), Some(RecordNote::Time { ms })),
            Note::Fault { fault, .. } => (Err(error(fault)), None),
            Note::End { .. } | Note::Switched { .. } | Note::Lost { .. } => {
                (Err(RecordError::Gone), None)
            }
        };
        if answer.is_err() {
            self.phase = Phase::Ended;
            self.delete();
        }
        let _ = started.send(answer);
        told
    }

    /// The job is gone: a start waiting fails, and a live run ends with one fault and loses its broken video.
    fn lose(&mut self) -> Option<RecordNote> {
        match std::mem::replace(&mut self.phase, Phase::Ended) {
            Phase::Starting(started) => {
                self.delete();
                let _ = started.send(Err(RecordError::Gone));
                None
            }
            Phase::Live => self.end(RecordNote::Fault(RecordError::Gone), true),
            Phase::Ended => None,
        }
    }

    /// Ends the run with `last`, settling a switch still waiting and deleting its video when `delete`.
    fn end(&mut self, last: RecordNote, delete: bool) -> Option<RecordNote> {
        self.phase = Phase::Ended;
        self.settle(Err(RecordError::Gone));
        if delete {
            self.delete();
        }
        self.tell(last)
    }

    /// Deletes the run's video; one the folder still held is tried again on leave.
    fn delete(&mut self) {
        self.held = !scrap(&*self.folder, &self.file);
    }

    /// Deletes the video a cancelled run's unheard end kept.
    fn take_back(&mut self) {
        if self.cancelled && self.kept == Kept::Unheard {
            self.kept = Kept::No;
            self.delete();
        }
    }

    /// Settles the waiting switch of `source` with `answer`; a fault of another sound is no answer.
    fn answer(&mut self, source: Source, answer: Result<(), RecordError>) -> Option<RecordNote> {
        if self.waiting.as_ref().is_some_and(|w| w.source == source) {
            self.settle(answer);
        }
        None
    }

    /// Settles the waiting switch, if any, with `answer`.
    fn settle(&mut self, answer: Result<(), RecordError>) {
        if let Some(waiting) = self.waiting.take() {
            let _ = waiting.answer.send(answer);
        }
    }

    /// Gives up on a switch the job left unanswered for its wait by the clock.
    fn overdue(&mut self) {
        let (now, switch) = (self.clock.now(), self.switch);
        let late = |w: &Waiting| now.saturating_since(w.since) >= switch;
        if self.waiting.as_ref().is_some_and(late) {
            self.settle(Err(RecordError::Gone));
        }
    }

    /// `note`, unless the run was cancelled, which tells nothing more.
    const fn tell(&self, note: RecordNote) -> Option<RecordNote> {
        if self.cancelled { None } else { Some(note) }
    }
}

/// The port's sound for the frames' switch.
const fn source(switch: Switch) -> Source {
    match switch {
        Switch::Mic => Source::Mic,
        Switch::Sound => Source::Sound,
    }
}

/// The sound whose switch a sound's fault answers.
const fn heard(fault: RecordFault) -> Source {
    match fault {
        RecordFault::NoSound => Source::Sound,
        _ => Source::Mic,
    }
}

/// The port's reason for the frames' end.
const fn kept(end: kx_ipc::RecordEnd) -> RecordEnd {
    match end {
        kx_ipc::RecordEnd::Stopped => RecordEnd::Stopped,
        kx_ipc::RecordEnd::Longest => RecordEnd::Longest,
        kx_ipc::RecordEnd::Full => RecordEnd::Full,
    }
}

#[cfg(test)]
mod tests;
