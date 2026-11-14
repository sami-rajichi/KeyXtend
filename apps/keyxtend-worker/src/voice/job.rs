//! The job's state: at most one run listening, one engine writing, one recording held until Close and one hand-in
//! under way. It takes each event in turn on one thread: asks, the source's chunks, the engine's notes and Again's
//! next stretch to cut.

mod asks;
mod notes;

use super::cut::{Cutter, Detectors};
use super::engine::{self, Engine, Writer};
use super::hand::{self, HandIn, Held};
use super::listen::{Replay, Run};
use super::mic::{Mic, Source};
use super::{Event, Feed, Notes, Tell, span};
use asks::{answer, stage};
use kx_ipc::{IpcError, Setup, Stage, VoiceAsk, VoiceFault, VoiceNote};
use notes::listened;
use std::sync::mpsc::Sender;

/// The job's parts: the sound source, the voice detector and the engine.
pub struct Parts {
    /// Where the sound comes from.
    pub source: Box<dyn Source>,
    /// Opens the detector that cuts pieces.
    pub detectors: Box<dyn Detectors>,
    /// Loads models and writes pieces.
    pub engine: Box<dyn Engine>,
}

impl Parts {
    /// The shipped parts: the microphone, and this build's detector and engine.
    pub fn live() -> Self {
        Self::over(Box::new(Mic))
    }

    /// This build's detector and engine over `source`.
    pub fn over(source: Box<dyn Source>) -> Self {
        Self {
            source,
            detectors: engine::detectors(),
            engine: engine::live(),
        }
    }
}

/// What the loop does after an event.
pub enum Flow {
    /// Takes the next event.
    Go,
    /// Ends with this outcome: goodbye or the end of input, or the frame that could not be read.
    Exit(Result<(), IpcError>),
}

/// Why a step failed: the pipe broke, or a fault the ask's note names.
#[derive(Copy, Clone)]
pub enum Fail {
    /// The notes cannot be written; the job ends.
    Out(IpcError),
    /// A fixed reason, told as a `Fault` note.
    Fault(VoiceFault),
}

impl From<IpcError> for Fail {
    fn from(e: IpcError) -> Self {
        Self::Out(e)
    }
}

impl From<VoiceFault> for Fail {
    fn from(fault: VoiceFault) -> Self {
        Self::Fault(fault)
    }
}

/// An engine still writing a run's pieces after its source closed; dropping it stops the writing.
struct Writing {
    id: u32,
    writer: Box<dyn Writer>,
    /// Again's cutting of the held recording, while pieces are left to give.
    replay: Option<Replay>,
}

/// The voice job's state.
pub struct Job {
    parts: Parts,
    tx: Sender<Event>,
    /// Counts the source's openings, so a closed one's late chunks are dropped.
    epoch: u64,
    run: Option<Run>,
    writing: Option<Writing>,
    held: Option<Held>,
    hand: Option<HandIn>,
    /// The engine asks whose answer has not come yet.
    asks: Vec<u32>,
}

impl Job {
    /// A job with nothing held, whose sources and engine send their events on `tx`.
    pub fn new(parts: Parts, tx: Sender<Event>) -> Self {
        Self {
            parts,
            tx,
            epoch: 0,
            run: None,
            writing: None,
            held: None,
            hand: None,
            asks: Vec::new(),
        }
    }

    /// Takes one event, telling its notes on `out`.
    ///
    /// # Errors
    /// The pipe's `IpcError` when a note cannot be written.
    pub fn take(&mut self, event: Event, out: &mut dyn Notes) -> Result<Flow, IpcError> {
        match event {
            Event::Ask(VoiceAsk::Bye) => return Ok(self.exit(Ok(()))),
            Event::End(end) => return Ok(self.exit(end)),
            Event::Ask(ask) => self.ask(ask, out)?,
            Event::Sound { epoch, chunk } => self.sound(epoch, &chunk, out)?,
            Event::Lost { epoch, fault } if self.live(epoch) => self.fail_run(fault, out)?,
            Event::Lost { .. } => {}
            Event::Note(note) => self.engine_note(note, out)?,
            Event::Replay(id) => self.replay(id, out)?,
        }
        Ok(Flow::Go)
    }

    /// The run the job serves: the listening one, or the one its engine writes.
    pub fn busy(&self) -> Option<u32> {
        let writing = self.writing.as_ref().map(|w| w.id);
        self.run.as_ref().map(|r| r.id).or(writing)
    }

    /// Closes the source, drops the engine's writer and wipes every recording.
    pub fn wipe(&mut self) {
        (self.run, self.writing, self.held, self.hand) = (None, None, None, None);
    }

    /// Wipes everything and ends with `end`.
    fn exit(&mut self, end: Result<(), IpcError>) -> Flow {
        self.wipe();
        Flow::Exit(end)
    }

    /// True while the source of opening `epoch` is the listening run's.
    fn live(&self, epoch: u64) -> bool {
        self.run.as_ref().is_some_and(|r| r.epoch == epoch)
    }

    /// Takes a chunk of the listening run's source; a chunk of a closed one is dropped.
    fn sound(&mut self, epoch: u64, chunk: &[f32], out: &mut dyn Notes) -> Result<(), IpcError> {
        let Some(run) = self.run.as_mut().filter(|r| r.epoch == epoch) else {
            return Ok(());
        };
        let id = run.id;
        match run.take(chunk, out) {
            Ok(None) => Ok(()),
            Ok(Some(why)) => {
                let ended = self.end_run(why, out);
                answer(out, id, ended)
            }
            Err(Fail::Out(e)) => Err(e),
            Err(Fail::Fault(fault)) => self.fail_run(fault, out),
        }
    }

    /// Ends the listening run at `why`: the stage, then the engine finishes, or the recording is handed over.
    fn end_run(&mut self, why: Stage, out: &mut dyn Notes) -> Result<(), Fail> {
        let Some(run) = self.run.take() else {
            return Ok(());
        };
        let id = run.id;
        let (sound, writer) = run.end(out)?;
        out.tell(stage(id, why))?;
        let heard = span::heard_ms(sound.len());
        if let Some(mut writer) = writer {
            writer.end(heard);
            self.writing = Some(Writing {
                id,
                writer,
                replay: None,
            });
        } else {
            hand::over(id, &sound, out)?;
            out.tell(listened(id, heard))?;
        }
        self.held = Some(Held::new(id, sound));
        Ok(())
    }

    /// Ends the listening run for `fault`, telling no more text; what it recorded stays held until Close.
    fn fail_run(&mut self, fault: VoiceFault, out: &mut dyn Notes) -> Result<(), IpcError> {
        let Some(mut run) = self.run.take() else {
            return Ok(());
        };
        let id = run.id;
        run.mute();
        match run.end(out) {
            Ok((sound, _)) => self.held = Some(Held::new(id, sound)),
            Err(Fail::Out(e)) => return Err(e),
            Err(Fail::Fault(_)) => {}
        }
        out.tell(stage(id, Stage::Shut))?;
        out.tell(VoiceNote::Fault { id, fault })
    }

    /// Opens run `id` as `setup` says: its detector, its engine's writer, then its source.
    fn begin(&mut self, id: u32, setup: &Setup<'_>) -> Result<Run, VoiceFault> {
        let cut = self.cutter(setup)?;
        let writer = self.writer(id, setup, cut.is_some())?;
        self.epoch += 1;
        let feed = Feed::new(self.tx.clone(), self.epoch);
        let opened = self.parts.source.open(feed.clone())?;
        feed.limit(span::source(setup.record_max_s, opened.format));
        Run::new(id, self.epoch, opened, setup, cut, writer)
    }

    /// A cutter over the setup's detector, or `None` when this build has no detector.
    fn cutter(&mut self, setup: &Setup<'_>) -> Result<Option<Cutter>, VoiceFault> {
        let det = self.parts.detectors.open(&setup.detector)?;
        let (piece, pad) = (span::secs(setup.piece_s.into()), span::millis(setup.pad_ms));
        Ok(det.map(|det| Cutter::new(det, piece, pad)))
    }

    /// The writer of run `id`'s model, or `None` to listen only; a model needs a detector to cut its pieces.
    fn writer(
        &mut self,
        id: u32,
        setup: &Setup<'_>,
        cuts: bool,
    ) -> Result<Option<Box<dyn Writer>>, VoiceFault> {
        let Some(model) = &setup.model else {
            return Ok(None);
        };
        if !cuts {
            return Err(VoiceFault::Runner);
        }
        let tell = Tell::new(self.tx.clone());
        self.parts.engine.load(id, model, tell).map(Some)
    }
}

#[cfg(test)]
mod tests;
