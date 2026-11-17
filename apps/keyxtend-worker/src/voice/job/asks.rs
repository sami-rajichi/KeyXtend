//! Each ask and the notes that answer it. Start tells `Open`; Stop, Cancel and Close tell `Shut`; Keep hands the
//! recording over; the engine answers devices, facts and checks by notes; a fault answers the rest, and a hand-in piece
//! is answered only by a fault.

use super::super::hand::{self, HandIn, Held};
use super::super::listen::Replay;
use super::super::span;
use super::super::{Event, Notes};
use super::{Fail, Job, Writing};
use kx_ipc::{IpcError, Setup, SoundPiece, Stage, VoiceAsk, VoiceFault, VoiceNote};

impl Job {
    /// Answers one ask; goodbye never comes here.
    pub(super) fn ask(
        &mut self,
        ask: VoiceAsk<'static>,
        out: &mut dyn Notes,
    ) -> Result<(), IpcError> {
        let id = ask.id();
        let done = match ask {
            VoiceAsk::Start { setup, .. } => self.start(id, &setup, out),
            VoiceAsk::Stop { .. } => self.stop(id, out),
            VoiceAsk::Cancel { .. } => self.cancel(id, out),
            VoiceAsk::Again { setup, .. } => self.again(id, &setup),
            VoiceAsk::Keep { .. } => self.keep(id, out),
            VoiceAsk::Close { .. } => self.close(id, out),
            VoiceAsk::Sound { piece, .. } => self.hand_in(id, &piece),
            VoiceAsk::Devices { .. } => self.engine_ask(id, |e, tell| e.devices(id, tell)),
            VoiceAsk::Facts { path, .. } => self.engine_ask(id, |e, tell| e.facts(id, &path, tell)),
            VoiceAsk::Check { model, clip, .. } => {
                self.engine_ask(id, |e, tell| e.check(id, &model, &clip, tell))
            }
            VoiceAsk::Bye => Ok(()),
        };
        answer(out, id, done)
    }

    /// Opens run `id`, wiping the recording held before; refused while a run listens or an engine writes.
    fn start(&mut self, id: u32, setup: &Setup<'_>, out: &mut dyn Notes) -> Result<(), Fail> {
        if self.run.is_some() || self.writing.is_some() {
            return Err(VoiceFault::OutOfStep.into());
        }
        (self.held, self.hand) = (None, None);
        let run = self.begin(id, setup)?;
        self.run = Some(run);
        out.tell(stage(id, Stage::Open))?;
        Ok(())
    }

    /// Closes run `id`'s source and finishes it; a run already stopped is only told `Shut` again.
    fn stop(&mut self, id: u32, out: &mut dyn Notes) -> Result<(), Fail> {
        if self.listening(id) {
            return self.end_run(Stage::Shut, out);
        }
        self.shut_again(id, out)
    }

    /// Closes run `id`'s source, or drops its engine's writing, telling no more text; the recording stays held.
    fn cancel(&mut self, id: u32, out: &mut dyn Notes) -> Result<(), Fail> {
        if let Some(mut run) = self.run.take_if(|r| r.id == id) {
            run.mute();
            let (sound, _) = run.end(out)?;
            self.held = Some(Held::new(id, sound));
            out.tell(stage(id, Stage::Shut))?;
            return Ok(());
        }
        if self.writing.take_if(|w| w.id == id).is_some() {
            out.tell(stage(id, Stage::Shut))?;
            return Ok(());
        }
        self.shut_again(id, out)
    }

    /// Writes the held recording again as run `id`, with the setup's model; the job then holds it for `id` too. Its
    /// cutting goes a stretch per turn of the loop, so asks and notes never wait for a long recording.
    fn again(&mut self, id: u32, setup: &Setup<'_>) -> Result<(), Fail> {
        if self.run.is_some() || self.writing.is_some() || setup.model.is_none() {
            return Err(VoiceFault::OutOfStep.into());
        }
        if self.held.is_none() {
            return Err(VoiceFault::NoSound.into());
        }
        let cut = self.cutter(setup)?.ok_or(VoiceFault::Runner)?;
        let writer = self.writer(id, setup, true)?.ok_or(VoiceFault::Internal)?;
        self.held.as_mut().ok_or(VoiceFault::NoSound)?.serve(id);
        let replay = Some(Replay::new(cut, span::millis(setup.pad_ms)));
        self.writing = Some(Writing { id, writer, replay });
        let _ = self.tx.send(Event::Replay(id));
        Ok(())
    }

    /// Cuts the next stretch of the recording Again writes as run `id`, and asks for another turn until all is given.
    pub(super) fn replay(&mut self, id: u32, out: &mut dyn Notes) -> Result<(), IpcError> {
        let Some(writing) = self.writing.as_mut().filter(|w| w.id == id) else {
            return Ok(());
        };
        let Some(replay) = writing.replay.as_mut() else {
            return Ok(());
        };
        let step = match self.held.as_ref().filter(|h| h.owns(id)) {
            Some(held) => replay.step(held.sound(), writing.writer.as_mut()),
            None => Err(VoiceFault::NoSound),
        };
        match step {
            Ok(true) => writing.replay = None,
            Ok(false) => drop(self.tx.send(Event::Replay(id))),
            Err(fault) => {
                self.writing = None;
                return out.tell(VoiceNote::Fault { id, fault });
            }
        }
        Ok(())
    }

    /// Hands run `id`'s held recording over in `Sound` notes.
    fn keep(&mut self, id: u32, out: &mut dyn Notes) -> Result<(), Fail> {
        if self.listening(id) {
            return Err(VoiceFault::OutOfStep.into());
        }
        let held = self.held.as_ref().filter(|h| h.owns(id));
        hand::over(id, held.ok_or(VoiceFault::NoSound)?.sound(), out)
    }

    /// Ends run `id`: its source closes, its writing stops, and its recording and hand-in are wiped.
    fn close(&mut self, id: u32, out: &mut dyn Notes) -> Result<(), Fail> {
        drop(self.run.take_if(|r| r.id == id));
        drop(self.writing.take_if(|w| w.id == id));
        drop(self.held.take_if(|h| h.owns(id)));
        drop(self.hand.take_if(|h| h.id() == id));
        out.tell(stage(id, Stage::Shut))?;
        Ok(())
    }

    /// Takes a piece of a recording handed in for run `id`; piece 0 starts it and wipes the one held before. Refused
    /// while a run listens or Again still cuts the held recording.
    fn hand_in(&mut self, id: u32, piece: &SoundPiece<'_>) -> Result<(), Fail> {
        let cutting = self.writing.as_ref().is_some_and(|w| w.replay.is_some());
        if self.run.is_some() || cutting {
            return Err(VoiceFault::OutOfStep.into());
        }
        if piece.index == 0 {
            (self.held, self.hand) = (None, Some(HandIn::new(id)));
        }
        let Some(hand) = self.hand.as_mut().filter(|h| h.id() == id) else {
            self.hand = None;
            return Err(VoiceFault::OutOfStep.into());
        };
        match hand.take(piece) {
            Ok(false) => Ok(()),
            Ok(true) => {
                self.held = self.hand.take().map(HandIn::held);
                Ok(())
            }
            Err(fault) => {
                self.hand = None;
                Err(fault.into())
            }
        }
    }

    /// Answers a Stop or Cancel of run `id` that no longer listens: `Shut` while its recording or writing is here.
    fn shut_again(&self, id: u32, out: &mut dyn Notes) -> Result<(), Fail> {
        let held = self.held.as_ref().is_some_and(|h| h.owns(id));
        let writing = self.writing.as_ref().is_some_and(|w| w.id == id);
        if !(held || writing) {
            return Err(VoiceFault::OutOfStep.into());
        }
        out.tell(stage(id, Stage::Shut))?;
        Ok(())
    }

    /// True while run `id` listens.
    fn listening(&self, id: u32) -> bool {
        self.run.as_ref().is_some_and(|r| r.id == id)
    }
}

/// Tells a step's fault as a note on `id`; a broken pipe ends the job.
///
/// # Errors
/// The pipe's `IpcError`.
pub fn answer(out: &mut dyn Notes, id: u32, done: Result<(), Fail>) -> Result<(), IpcError> {
    match done {
        Ok(()) => Ok(()),
        Err(Fail::Out(e)) => Err(e),
        Err(Fail::Fault(fault)) => out.tell(VoiceNote::Fault { id, fault }),
    }
}

/// Run `id` reached `stage`.
pub fn stage(id: u32, stage: Stage) -> VoiceNote {
    VoiceNote::Stage { id, stage }
}
