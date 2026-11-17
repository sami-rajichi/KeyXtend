//! Writing a run: each piece's text in turn, a card that fails handing the same sound to the processor, and the end.

use super::{Desk, Held, fault};
use crate::voice::library::{Failed, Library, Place, Stop, ms_since};
use crate::voice::resample::floats;
use crate::voice::split;
use kx_ipc::{MAX_SOUND_MS, VoiceFault, VoiceNote};
use std::time::Instant;
use zeroize::Zeroizing;

/// The memory figure a done note carries: the worker cannot read its own peak, so the host reads its job's.
const MEMORY_UNREAD: u32 = 0;

impl<L: Library> Desk<L> {
    /// Writes one piece of the run `stop` belongs to; a fault ends the run, and a stopped one is dropped unwritten and
    /// tells nothing, even a text that came back after the stop.
    pub(super) fn piece(&mut self, stop: &Stop, sound: &[i16]) {
        let Some(run) = self.mine(stop) else {
            return;
        };
        let (id, tell) = (run.id, run.tell.clone());
        if stop.is_set() {
            self.run = None;
            return;
        }
        match self.write(sound) {
            Ok(_) if stop.is_set() => self.run = None,
            Ok(text) => {
                if let Some(run) = self.run.as_mut() {
                    tell.note(VoiceNote::Piece {
                        id,
                        index: run.told,
                        text,
                    });
                    run.told = run.told.saturating_add(1);
                }
            }
            Err(why) => {
                self.run = None;
                if let Some(why) = why {
                    tell.note(fault(id, why));
                }
            }
        }
    }

    /// The text of `sound`; a card that fails hands the same sound to the processor. `None` when stopped.
    fn write(&mut self, sound: &[i16]) -> Result<Zeroizing<String>, Option<VoiceFault>> {
        let pcm = floats(sound).map_err(Some)?;
        let on_card = self.run.as_ref().is_some_and(|r| r.card);
        match self.spoken(&pcm) {
            Err(why) if on_card && why.card() => {
                self.fall_back()?;
                self.spoken(&pcm).map_err(lost)
            }
            other => other.map_err(lost),
        }
    }

    /// The held model's text of `pcm` in the run's language.
    fn spoken(&mut self, pcm: &[f32]) -> Result<Zeroizing<String>, Failed> {
        let (Some(run), Some(held)) = (self.run.as_ref(), self.held.as_mut()) else {
            return Err(Failed::Other);
        };
        split::text(&mut held.model, pcm, &run.lang, &run.stop)
    }

    /// Tells the card failed, drops its model and loads the same file on the processor.
    fn fall_back(&mut self) -> Result<(), Option<VoiceFault>> {
        let run = self.run.as_mut().ok_or(None)?;
        run.card = false;
        run.tell.note(fault(run.id, VoiceFault::CardFailed));
        let Held { path, threads, .. } = self.held.take().ok_or(Some(VoiceFault::Internal))?;
        let model = self.lib.load(&path, &Place::Cpu, threads);
        let model = model.map_err(|why| Some(why.fault()))?;
        let place = Place::Cpu;
        self.held = Some(Held {
            path,
            place,
            threads,
            model,
        });
        Ok(())
    }

    /// Ends the run `stop` belongs to with its done note, unless it was stopped.
    pub(super) fn end(&mut self, stop: &Stop, heard_ms: u32, at: Instant) {
        let Some(run) = self.run.take_if(|r| r.stop.same(stop)) else {
            return;
        };
        if stop.is_set() {
            return;
        }
        run.tell.note(VoiceNote::Done {
            id: run.id,
            pieces: run.told,
            sound_ms: heard_ms.min(MAX_SOUND_MS),
            wait_ms: ms_since(at),
            load_ms: run.load_ms,
            memory_mb: MEMORY_UNREAD,
            card: run.card,
        });
    }
}

/// A failed run's fault, or `None` when it was stopped.
fn lost(why: Failed) -> Option<VoiceFault> {
    (why != Failed::Stopped).then(|| why.fault())
}
