//! The engine's notes: a run's go on only while its writer is here, and an engine ask's until its answer; a fault that
//! ends a listening run closes its source first. A card's fault leaves the run going on the processor.

use super::super::engine::Engine;
use super::super::{Notes, Tell};
use super::{Fail, Job};
use kx_ipc::{IpcError, VoiceFault, VoiceNote};

impl Job {
    /// Passes on a note of the engine's writing run or of an engine ask; a dropped writer's late note is dropped.
    pub(super) fn engine_note(
        &mut self,
        note: VoiceNote,
        out: &mut dyn Notes,
    ) -> Result<(), IpcError> {
        let id = note.id();
        if self.run.as_ref().is_some_and(|r| r.id == id && r.writes()) {
            return match note {
                VoiceNote::Fault { fault, .. } if ends(fault) => self.fail_run(fault, out),
                note => out.tell(note),
            };
        }
        if let Some(at) = self.asks.iter().position(|&a| a == id) {
            if answers(&note) {
                self.asks.swap_remove(at);
            }
            return out.tell(note);
        }
        if self.writing.as_ref().is_none_or(|w| w.id != id) {
            return Ok(());
        }
        if last(&note) {
            self.writing = None;
        }
        out.tell(note)
    }

    /// Hands ask `id` to the engine, whose answer comes as a note; refused while an ask of that id waits.
    pub(super) fn engine_ask(
        &mut self,
        id: u32,
        ask: impl FnOnce(&mut dyn Engine, Tell) -> Result<(), VoiceFault>,
    ) -> Result<(), Fail> {
        if self.asks.contains(&id) {
            return Err(VoiceFault::OutOfStep.into());
        }
        ask(self.parts.engine.as_mut(), Tell::new(self.tx.clone()))?;
        self.asks.push(id);
        Ok(())
    }
}

/// True for a note that answers an engine ask: its answer or any fault, a card's too, as a check is of the card.
fn answers(note: &VoiceNote) -> bool {
    matches!(
        note,
        VoiceNote::Devices { .. }
            | VoiceNote::Facts { .. }
            | VoiceNote::Checked { .. }
            | VoiceNote::Fault { .. }
    )
}

/// True for a run's last note: its end, or a fault other than a card's.
fn last(note: &VoiceNote) -> bool {
    match note {
        VoiceNote::Done { .. } => true,
        VoiceNote::Fault { fault, .. } => ends(*fault),
        _ => false,
    }
}

/// False for a card's fault, after which the processor goes on with the same run.
fn ends(fault: VoiceFault) -> bool {
    !matches!(
        fault,
        VoiceFault::NoCard | VoiceFault::CardRoom | VoiceFault::CardFailed
    )
}

/// The end of a listen-only run: no text and `sound_ms` heard; its memory figure is the host's to read from the job.
pub fn listened(id: u32, sound_ms: u32) -> VoiceNote {
    VoiceNote::Done {
        id,
        pieces: 0,
        sound_ms,
        wait_ms: 0,
        load_ms: 0,
        memory_mb: 0,
        card: false,
    }
}
