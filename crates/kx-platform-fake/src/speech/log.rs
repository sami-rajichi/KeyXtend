//! The fake's log of runs and what the test set, and the one path every note takes: a run's callback is held while
//! its notes are marked and told, so a close waits out a note being told and none comes after it.

use super::info::Info;
use super::{Hearer, RunLog, SpeechCall, Tell};
use crate::lock;
use kx_module_api::SecretBytes;
use kx_module_api::ports::{SpeechError, SpeechNote};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Condvar, Mutex};

/// True for an error that is the worker's death, told or returned: the host kills a silent worker.
pub(super) fn fatal(error: SpeechError) -> bool {
    matches!(error, SpeechError::Gone | SpeechError::Timeout)
}

/// True for a note that tells the worker's death.
fn death(note: &SpeechNote) -> bool {
    matches!(note, SpeechNote::Fault(error) if fatal(*error))
}

/// A run's callback until it closes, and whether `cancel` muted its text until `again`.
pub(super) struct Ear {
    pub(super) hearer: Option<Hearer>,
    pub(super) muted: bool,
}

impl Ear {
    /// A new run's ear, which hears everything.
    pub(super) fn new(tell: Tell) -> Self {
        let hearer = Some(Arc::new(Mutex::new(tell)));
        Self {
            hearer,
            muted: false,
        }
    }
}

/// What the runs did, and what the test set for them.
#[derive(Default)]
pub(super) struct Log {
    pub(super) runs: Vec<RunLog>,
    /// Each run's ear, by its place in `runs`.
    pub(super) ears: Vec<Ear>,
    pub(super) cues: HashMap<SpeechCall, VecDeque<Vec<SpeechNote>>>,
    pub(super) fails: HashMap<SpeechCall, SpeechError>,
    pub(super) sound: Vec<SecretBytes>,
    pub(super) info: Info,
}

impl Log {
    /// The notes scripted for the next `call` of run `id`, or none; a closed run leaves them for the next one.
    pub(super) fn cue(&mut self, id: usize, call: SpeechCall) -> Vec<SpeechNote> {
        if !self.runs[id].live() {
            return Vec::new();
        }
        let queued = self.cues.get_mut(&call).and_then(VecDeque::pop_front);
        queued.unwrap_or_default()
    }

    /// Refuses `call` on run `id`: `NoSound` without the recording, `Gone` after a death, else the failure set, which
    /// kills the worker when it is `Gone` or `Timeout`.
    pub(super) fn refuse(&mut self, id: usize, call: SpeechCall) -> Result<(), SpeechError> {
        let run = &self.runs[id];
        if matches!(call, SpeechCall::Again | SpeechCall::Keep) && !run.held {
            return Err(SpeechError::NoSound);
        }
        if run.dead {
            return Err(SpeechError::Gone);
        }
        let failed = self.fails.remove(&call);
        if failed.is_some_and(fatal) {
            self.kill(id);
        }
        failed.map_or(Ok(()), Err)
    }

    /// Marks run `id` dead: it hears no more and its recording is lost.
    fn kill(&mut self, id: usize) {
        let run = &mut self.runs[id];
        run.dead = true;
        run.held = false;
    }

    /// Cuts `notes` to what run `id` hears: none once it closed or died, no text after Cancel, nothing after a death;
    /// a death kills it and a `Wiped` drops its recording.
    fn mark(&mut self, id: usize, notes: &mut Vec<SpeechNote>) {
        if !self.runs[id].live() {
            return notes.clear();
        }
        if self.ears[id].muted {
            notes.retain(|n| !matches!(n, SpeechNote::Piece { .. }));
        }
        if let Some(end) = notes.iter().position(death) {
            notes.truncate(end + 1);
            self.kill(id);
        }
        if notes.iter().any(|n| matches!(n, SpeechNote::Wiped)) {
            self.runs[id].held = false;
        }
    }
}

/// The log, and the signal each change gives, for a test waiting on another thread.
#[derive(Default)]
pub(super) struct Shared {
    pub(super) log: Mutex<Log>,
    pub(super) changed: Condvar,
}

impl Shared {
    /// Tells run `id` what it hears of `notes`, in order, on this thread with the log unlocked; false when it heard
    /// none. Its callback stays held from the marking to the last note, so notes keep their order across threads.
    pub(super) fn tell(&self, id: usize, mut notes: Vec<SpeechNote>) -> bool {
        let hearer = lock(&self.log).ears[id].hearer.clone();
        let Some(hearer) = hearer else {
            self.changed.notify_all();
            return false;
        };
        let tell = lock(&hearer);
        lock(&self.log).mark(id, &mut notes);
        self.changed.notify_all();
        let told = !notes.is_empty();
        for note in notes {
            tell(note);
        }
        told
    }
}
