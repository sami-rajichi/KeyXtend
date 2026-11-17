//! Which run each note of the dictation job belongs to, and what each run still holds: its ids, its recording, its
//! worker's life, a `keep` waiting for its pieces and a listen-only run's speech. A run's callback is called with the
//! router unlocked and is held while it hears, so a close waits out a note being told and none comes after it.

use super::note::{lost, note};
use super::spoken::Spoken;
use crate::ear::Ear;
use crate::worker::Told;
use kx_ipc::{VoiceFault, VoiceNote, Wiped};
use kx_module_api::ports::{SpeechError, SpeechNote};
use kx_module_api::{Redacted, SecretBytes, Zeroizing};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::mpsc::Sender;

/// The callback a run tells its notes to.
pub(super) type Tell = Box<dyn Fn(SpeechNote) + Send>;
/// A run's callback, closed with the run.
pub(super) type Hearer = Arc<Ear<SpeechNote>>;
/// A kept piece and whether it is the last, or why the keep failed.
pub(super) type Kept = Result<(SecretBytes, bool), SpeechError>;
/// Notes to tell once the router is unlocked, each list to its run.
pub(super) type Out = Vec<(Hearer, Vec<SpeechNote>)>;

/// One open run.
struct Entry {
    hearer: Hearer,
    /// The job holds its recording.
    held: bool,
    /// Its worker died, which took the recording.
    dead: bool,
    /// A `keep` waiting for the recording's pieces.
    keep: Option<Sender<Kept>>,
    /// Its speech since its newest ask, when it listens alone.
    spoken: Spoken,
}

/// Where a run stands, as its calls check it.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(super) struct State {
    pub(super) held: bool,
    pub(super) dead: bool,
}

/// The open runs by number, and the run each ask id of the dictation job belongs to.
#[derive(Default)]
pub(super) struct Router {
    next: u64,
    runs: HashMap<u64, Entry>,
    ids: HashMap<u32, u64>,
}

impl Router {
    /// Opens a run that tells `tell`, holding no recording yet; its number.
    pub(super) fn add(&mut self, tell: Tell) -> u64 {
        self.next += 1;
        let entry = Entry {
            hearer: Arc::new(Ear::new(tell)),
            held: false,
            dead: false,
            keep: None,
            spoken: Spoken::default(),
        };
        self.runs.insert(self.next, entry);
        self.next
    }

    /// Gives `id` to `run`, whose job then holds a recording for it; the speech of its asks before goes.
    pub(super) fn map(&mut self, run: u64, id: u32) {
        if let Some(entry) = self.runs.get_mut(&run) {
            (entry.held, entry.dead) = (true, false);
            entry.spoken = Spoken::default();
            self.ids.insert(id, run);
        }
    }

    /// Takes `id` back after its ask was not sent, so its run's job holds nothing for it.
    pub(super) fn unmap(&mut self, id: u32) {
        let run = self.ids.remove(&id);
        if let Some(entry) = run.and_then(|run| self.runs.get_mut(&run)) {
            entry.held = false;
        }
    }

    /// Where `run` stands; `None` once it closed.
    pub(super) fn state(&self, run: u64) -> Option<State> {
        let entry = self.runs.get(&run)?;
        let (held, dead) = (entry.held, entry.dead);
        Some(State { held, dead })
    }

    /// Marks `run`'s worker dead after a call found it gone, which took its speech too.
    pub(super) fn kill(&mut self, run: u64) {
        if let Some(entry) = self.runs.get_mut(&run) {
            (entry.held, entry.dead) = (false, true);
            entry.spoken = Spoken::default();
        }
    }

    /// Fresh copies of `run`'s speech, once its end came.
    ///
    /// # Errors
    /// `NoSound` before its end or once it closed or was wiped, `Internal` when the job broke its order.
    pub(super) fn spoken(&self, run: u64) -> Result<Vec<SecretBytes>, SpeechError> {
        let entry = self.runs.get(&run).ok_or(SpeechError::NoSound)?;
        entry.spoken.give()
    }

    /// Sends `run`'s next kept pieces to `to`, until the last.
    pub(super) fn keep(&mut self, run: u64, to: Sender<Kept>) {
        if let Some(entry) = self.runs.get_mut(&run) {
            entry.keep = Some(to);
        }
    }

    /// Closes `run`; its callback, to wait out a note being told.
    pub(super) fn close(&mut self, run: u64) -> Option<Hearer> {
        self.ids.retain(|_, owner| *owner != run);
        self.runs.remove(&run).map(|entry| entry.hearer)
    }

    /// The other runs' recordings went with their job: each that held one is told `Wiped`, all but `except`, and its
    /// speech goes too.
    pub(super) fn wipe(&mut self, except: Option<u64>) -> Out {
        let mut out = Out::new();
        for (run, entry) in &mut self.runs {
            if Some(*run) != except && entry.held {
                entry.held = false;
                entry.spoken = Spoken::default();
                out.push((Arc::clone(&entry.hearer), vec![SpeechNote::Wiped]));
            }
        }
        self.ids.retain(|_, owner| Some(*owner) == except);
        out
    }

    /// Takes what the dictation job's stream told: a note goes to its run or its `keep`, a lost worker kills every
    /// run, and an idle one wipes every recording.
    pub(super) fn hear(&mut self, told: Told<VoiceNote>) -> Out {
        match told {
            Told::Note(voice) => self.note(voice),
            Told::Lost(gone) => self.lose(lost(gone)),
            Told::Idle => self.wipe(None),
        }
    }

    /// Routes one note to its run, a piece of the recording to the waiting `keep`, speech to the run's own, and any
    /// fault that ends the id to that `keep` too, so it never waits on; a job that says it holds no recording dims the
    /// run's keep and again.
    fn note(&mut self, voice: VoiceNote) -> Out {
        let Some(entry) = self.ids.get(&voice.id()).and_then(|r| self.runs.get_mut(r)) else {
            return Out::new();
        };
        let lost_sound = VoiceNote::Fault {
            id: voice.id(),
            fault: VoiceFault::NoSound,
        };
        entry.held &= voice != lost_sound;
        let told = match voice {
            VoiceNote::Sound { piece, .. } => {
                if let Some(keep) = &entry.keep {
                    let _ = keep.send(Ok((secret(piece.bytes), piece.last)));
                }
                if piece.last {
                    entry.keep = None;
                }
                return Out::new();
            }
            VoiceNote::Speech { piece, .. } => {
                entry.spoken.take(piece);
                return Out::new();
            }
            VoiceNote::Done { .. } => {
                entry.spoken.end();
                note(voice)
            }
            VoiceNote::Fault { fault, .. }
                if SpeechError::from(fault).ends() && entry.keep.is_some() =>
            {
                let _ = entry.keep.take().map(|keep| keep.send(Err(fault.into())));
                if refuses_keep(fault) {
                    return Out::new();
                }
                note(voice)
            }
            other => note(other),
        };
        vec![(Arc::clone(&entry.hearer), told.into_iter().collect())]
    }

    /// Kills every run on a worker that died with `error`, telling it as a fault.
    fn lose(&mut self, error: SpeechError) -> Out {
        self.ids.clear();
        let mut out = Out::new();
        for entry in self.runs.values_mut() {
            (entry.held, entry.dead) = (false, true);
            entry.spoken = Spoken::default();
            let _ = entry.keep.take().map(|keep| keep.send(Err(error)));
            out.push((Arc::clone(&entry.hearer), vec![SpeechNote::Fault(error)]));
        }
        out
    }
}

/// A fault the job answers a `keep` with.
fn refuses_keep(fault: VoiceFault) -> bool {
    matches!(fault, VoiceFault::NoSound | VoiceFault::OutOfStep)
}

/// A kept piece's bytes as the port's secret, without a copy when the frame kept them.
pub(super) fn secret(bytes: Wiped<'_>) -> SecretBytes {
    match bytes {
        Wiped::Kept(kept) => Redacted::new(kept),
        Wiped::Lent(lent) => Redacted::new(Zeroizing::new(lent.to_vec())),
    }
}

/// Tells each run its notes in order, holding its callback while it hears; a closed run hears none.
pub(super) fn tell(out: Out) {
    for (hearer, notes) in out {
        hearer.tell(notes);
    }
}

/// Closes a run's callback once a note being told to it is done, so none comes after, even from inside it.
pub(super) fn hush(hearer: &Hearer) {
    hearer.close();
}
