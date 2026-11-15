//! The slow reads, each in a job off the actor: the devices, a file's fingerprint (one at a time) and facts, a card's
//! check and showing the models folder. Their answers come back through the inbox; a panic in a port counts as a fault.

use std::collections::VecDeque;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use kx_module_api::ports::{
    BarHint, CheckDone, Device, FactsAsk, Fingerprint, FolderError, FolderPath, ModelError,
    SpeechError, SpeechNote,
};

use super::dos::Fetches;
use super::{Actor, Back, Job, Msg};
use crate::panel::Check;

/// The downloads running and the reads waiting or in flight.
#[derive(Default)]
pub(super) struct Jobs {
    /// Each download by its row.
    pub fetches: Fetches,
    /// The files whose fingerprints wait, with their sizes.
    prints: VecDeque<(PathBuf, u64)>,
    /// The file whose fingerprint is being read, with its size.
    printing: Option<(PathBuf, u64)>,
    /// The devices are being read.
    reading: bool,
}

/// Runs `read`, a panic in the port giving `fault`.
fn guarded<T, E>(read: impl FnOnce() -> Result<T, E>, fault: E) -> Result<T, E> {
    catch_unwind(AssertUnwindSafe(read)).unwrap_or(Err(fault))
}

impl Actor {
    /// Carries out a job's answer.
    pub(super) fn answer(&mut self, back: Back) {
        match back {
            Back::Note { lane, note } => self.noted(lane, note),
            Back::Opened { lane, run } => self.opened(lane, run),
            Back::Called { lane, run, answer } => self.called(lane, run, answer),
            Back::Devices(devices) => self.devices(devices),
            Back::Facts(print, facts) => self.panel.facts_read(print, facts),
            Back::Printed { file, bytes, print } => self.printed(&file, bytes, print),
            Back::Warming { card, percent } => self.panel.warming(&card, percent),
            Back::Checked(check, done) => self.checked(&check, done),
            Back::Fetched { row, note } => self.fetched(&row, note),
            Back::Shown(shown) => self.shown(shown),
        }
    }

    /// Reads the devices with the job's limits, unless a read is on its way.
    pub(super) fn read_devices(&mut self) {
        if std::mem::replace(&mut self.jobs.reading, true) {
            return;
        }
        let (speech, back) = (Arc::clone(&self.ports.speech), self.ports.back.clone());
        let limits = self.held.settings.job();
        let job: Job = Box::new(move || {
            let read = guarded(|| speech.devices(&limits), SpeechError::Internal);
            // Only a stopped actor misses an answer, and nothing waits on it then.
            let _ = back.send(Msg::Back(Back::Devices(read)));
        });
        if !(self.ports.spawn)(job) {
            self.devices(Err(SpeechError::Internal));
        }
    }

    /// Reads an other file's facts as `ask` says.
    pub(super) fn read_facts(&mut self, print: Fingerprint, ask: FactsAsk) {
        let (speech, back) = (Arc::clone(&self.ports.speech), self.ports.back.clone());
        let job: Job = Box::new(move || {
            let read = guarded(|| speech.facts(&ask), SpeechError::Internal);
            let _ = back.send(Msg::Back(Back::Facts(print, read)));
        });
        if !(self.ports.spawn)(job) {
            self.panel.facts_read(print, Err(SpeechError::Internal));
        }
    }

    /// Reads `file`'s fingerprint after those before it, unless it waits or is being read, as each scan asks again.
    pub(super) fn print(&mut self, file: PathBuf, bytes: u64) {
        let asked = (file, bytes);
        let jobs = &mut self.jobs;
        if jobs.printing.as_ref() == Some(&asked) || jobs.prints.contains(&asked) {
            return;
        }
        jobs.prints.push_back(asked);
        self.next_print();
    }

    /// Runs a card's check; its warm-up and its end come to the panel.
    pub(super) fn check(&mut self, check: &Check) {
        let (speech, back) = (Arc::clone(&self.ports.speech), self.ports.back.clone());
        let (warm, card, ask) = (back.clone(), check.card.clone(), check.ask.clone());
        let tell = Box::new(move |note: SpeechNote| {
            if let SpeechNote::Warming { percent } = note {
                let card = card.clone();
                drop(warm.send(Msg::Back(Back::Warming { card, percent })));
            }
        });
        let kept = Box::new(check.clone());
        let job: Job = Box::new(move || {
            let done = guarded(|| speech.check(ask, tell), SpeechError::Internal);
            let _ = back.send(Msg::Back(Back::Checked(kept, done)));
        });
        if !(self.ports.spawn)(job) {
            self.checked(check, Err(SpeechError::Internal));
        }
    }

    /// Shows `dir` in the file manager, once checked; a refusal shows the folder band.
    pub(super) fn reveal(&mut self, dir: PathBuf) {
        let folder = match FolderPath::new(dir) {
            Ok(folder) => folder,
            Err(error) => return self.shown(Err(error)),
        };
        let (reveal, back) = (Arc::clone(&self.ports.reveal), self.ports.back.clone());
        let job: Job = Box::new(move || {
            let _ = back.send(Msg::Back(Back::Shown(reveal.show(&folder))));
        });
        if !(self.ports.spawn)(job) {
            self.panel.note = Some(BarHint::Folder);
        }
    }

    /// The devices as read: the panel rechecks the card; a failure shows the fault band.
    fn devices(&mut self, read: Result<Vec<Device>, SpeechError>) {
        self.jobs.reading = false;
        match read {
            Ok(devices) => {
                self.held.devices = devices;
                let facts = self.held.facts(self.ports.calendar.now(), &self.flow);
                let dos = self.panel.recheck(&facts);
                self.dos(dos);
            }
            Err(error) => {
                tracing::warn!(%error, "the devices could not be read");
                if self.panel.page().is_some() {
                    self.panel.note = Some(BarHint::Fault);
                }
            }
        }
    }

    /// Starts the next fingerprint's read while none runs.
    fn next_print(&mut self) {
        if self.jobs.printing.is_some() {
            return;
        }
        let Some((file, bytes)) = self.jobs.prints.pop_front() else {
            return;
        };
        self.jobs.printing = Some((file.clone(), bytes));
        let (files, back, path) = (
            Arc::clone(&self.ports.files),
            self.ports.back.clone(),
            file.clone(),
        );
        let job: Job = Box::new(move || {
            let print = guarded(|| files.fingerprint(&path), ModelError::Io);
            let _ = back.send(Msg::Back(Back::Printed {
                file: path,
                bytes,
                print,
            }));
        });
        if !(self.ports.spawn)(job) {
            self.printed(&file, bytes, Err(ModelError::Io));
        }
    }

    /// A fingerprint as read, for the panel; then the next one.
    fn printed(&mut self, file: &Path, bytes: u64, print: Result<Fingerprint, ModelError>) {
        self.jobs.printing = None;
        let facts = self.held.facts(self.ports.calendar.now(), &self.flow);
        let dos = self.panel.printed(file, bytes, print, &facts);
        self.dos(dos);
        self.next_print();
    }

    /// A card's check ended, for the panel.
    fn checked(&mut self, check: &Check, done: Result<CheckDone, SpeechError>) {
        let facts = self.held.facts(self.ports.calendar.now(), &self.flow);
        let dos = self.panel.checked(check, done, &facts);
        self.dos(dos);
    }

    /// The folder was shown, or the folder band says why not.
    fn shown(&mut self, shown: Result<(), FolderError>) {
        if let Err(error) = shown {
            tracing::warn!(%error, "the models folder could not be shown");
            self.panel.note = Some(BarHint::Folder);
        }
    }
}
