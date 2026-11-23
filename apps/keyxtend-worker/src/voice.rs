//! The voice job (ADR-0042): `keyxtend-worker voice` opens the microphone, makes its sound 16 kHz mono, tells levels
//! and speech, cuts pieces at pauses for the engine and hands its recording over, with no network. Asks come on stdin
//! and notes go on stdout in order; sound is held only in wiped buffers and is never written anywhere else.

#[cfg(any(test, feature = "engine"))]
mod cards;
#[cfg(any(test, feature = "engine"))]
mod check;
mod cut;
#[cfg(any(test, feature = "engine"))]
mod detect;
mod engine;
#[cfg(test)]
mod fake;
mod hand;
mod job;
mod level;
#[cfg(any(test, feature = "engine"))]
mod library;
mod listen;
mod mic;
mod pack;
#[cfg(feature = "gate")]
mod played;
mod resample;
#[cfg(any(test, feature = "engine"))]
mod run;
mod span;
mod split;
#[cfg(any(test, feature = "engine", feature = "gate"))]
pub(crate) mod wav;
mod wipe;

pub use job::Parts;

use crate::serve;
use crate::stream::{self, Heard, Note, Pipe, Streamed};
use job::{Flow, Job};
use kx_ipc::{IpcError, VoiceAsk, VoiceFault, VoiceNote};
use std::io::{Read, Write};
use std::panic::{self, AssertUnwindSafe};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Sender};
use wipe::Buf;

/// The name of the thread that reads the asks.
const READER: &str = "voice-asks";

/// Gate builds only: the parts with the WAV file at `path` played in place of the microphone.
///
/// # Errors
/// `NoFile` or `BadFile` when the file cannot be read.
#[cfg(feature = "gate")]
pub fn played(path: &std::path::Path) -> Result<Parts, VoiceFault> {
    Ok(Parts::over(Box::new(played::Played::read(path)?)))
}

/// What wakes the job.
pub enum Event {
    /// An ask from the host.
    Ask(VoiceAsk<'static>),
    /// A chunk from the source.
    #[cfg_attr(
        not(any(test, windows, feature = "gate")),
        expect(dead_code, reason = "only the microphone and the gate send sound")
    )]
    Sound {
        /// The opening it came from.
        epoch: u64,
        /// Interleaved samples, as the source gave them.
        chunk: Buf<f32>,
    },
    /// The source stopped by itself.
    #[cfg_attr(
        not(any(test, windows, feature = "gate")),
        expect(dead_code, reason = "only the microphone and the gate stop alone")
    )]
    Lost {
        /// The opening that stopped.
        epoch: u64,
        /// Why.
        fault: VoiceFault,
    },
    /// A note the engine tells.
    #[cfg_attr(
        not(any(test, feature = "engine")),
        expect(dead_code, reason = "only the engine tells notes")
    )]
    Note(VoiceNote),
    /// Again's next stretch of the held recording to cut, for this run.
    Replay(u32),
    /// The input ended cleanly, or with the frame that could not be read.
    End(Result<(), IpcError>),
}

/// Hands one opening's sound to the job, so a closed opening's late chunks are dropped. It stops once it has handed
/// over all its run can record, so a stalled job's queue stays bounded.
#[derive(Clone)]
pub struct Feed {
    #[cfg_attr(
        not(any(test, windows, feature = "gate")),
        expect(dead_code, reason = "only the microphone and the gate feed the job")
    )]
    tx: Sender<Event>,
    #[cfg_attr(
        not(any(test, windows, feature = "gate")),
        expect(dead_code, reason = "only the microphone and the gate feed the job")
    )]
    epoch: u64,
    /// Source samples it may still hand over; a chunk goes while any are left.
    room: Arc<AtomicU64>,
}

impl Feed {
    /// A feed for opening `epoch`, sending on `tx`, with no limit until `limit`.
    fn new(tx: Sender<Event>, epoch: u64) -> Self {
        let room = Arc::new(AtomicU64::new(u64::MAX));
        Self { tx, epoch, room }
    }

    /// Lets it hand over `samples` more source samples, the chunk that crosses that count included.
    fn limit(&self, samples: u64) {
        self.room.store(samples, Ordering::Relaxed);
    }

    /// Hands over one chunk of interleaved samples as the source gave them; past the limit it is dropped and wiped.
    #[cfg_attr(
        not(any(test, windows, feature = "gate")),
        expect(dead_code, reason = "only the microphone and the gate feed the job")
    )]
    pub fn sound(&self, chunk: Buf<f32>) {
        let n = u64::try_from(chunk.len()).unwrap_or(u64::MAX);
        let take = |room: u64| (room > 0).then(|| room.saturating_sub(n));
        if self
            .room
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, take)
            .is_ok()
        {
            let epoch = self.epoch;
            let _ = self.tx.send(Event::Sound { epoch, chunk });
        }
    }

    /// Says the source stopped by itself, and why.
    #[cfg_attr(
        not(any(test, windows, feature = "gate")),
        expect(dead_code, reason = "only the microphone and the gate feed the job")
    )]
    pub fn lost(&self, fault: VoiceFault) {
        let epoch = self.epoch;
        let _ = self.tx.send(Event::Lost { epoch, fault });
    }
}

/// Hands the engine's notes to the job, which writes them in order with its own.
#[derive(Clone)]
pub struct Tell {
    #[cfg_attr(
        not(any(test, feature = "engine")),
        expect(dead_code, reason = "only the engine tells notes")
    )]
    tx: Sender<Event>,
}

impl Tell {
    /// A teller sending on `tx`.
    fn new(tx: Sender<Event>) -> Self {
        Self { tx }
    }

    /// Tells `note` after the notes already told.
    #[cfg_attr(
        not(any(test, feature = "engine")),
        expect(dead_code, reason = "only the engine tells notes")
    )]
    pub fn note(&self, note: VoiceNote) {
        let _ = self.tx.send(Event::Note(note));
    }
}

/// Where the job's notes go: the pipe, or a list in tests.
pub trait Notes {
    /// Sends `note`.
    ///
    /// # Errors
    /// The pipe's `IpcError` when it fails.
    fn tell(&mut self, note: VoiceNote) -> Result<(), IpcError>;
}

impl<W: Write + ?Sized> Notes for Pipe<'_, W> {
    fn tell(&mut self, note: VoiceNote) -> Result<(), IpcError> {
        self.send(&note)
    }
}

/// A note that fails its own checks goes as an internal fault instead.
impl Note for VoiceNote {
    fn put<W: Write + ?Sized>(&self, w: &mut W) -> Result<(), IpcError> {
        self.check()?;
        self.write(w)
    }

    fn instead(&self) -> Self {
        let (id, fault) = (self.id(), VoiceFault::Internal);
        Self::Fault { id, fault }
    }
}

impl Streamed for VoiceAsk<'static> {
    fn read_from(r: &mut dyn Read) -> Result<Self, IpcError> {
        Self::read(r)
    }

    fn last(&self) -> bool {
        matches!(self, Self::Bye)
    }
}

impl From<Heard<VoiceAsk<'static>>> for Event {
    fn from(heard: Heard<VoiceAsk<'static>>) -> Self {
        match heard {
            Heard::Ask(ask) => Self::Ask(ask),
            Heard::End(end) => Self::End(end),
        }
    }
}

/// The voice job: `run` with every panic kept off stderr.
///
/// # Errors
/// As `run`.
pub fn serve<R, W>(input: R, w: &mut W, parts: Parts) -> Result<(), IpcError>
where
    R: Read + Send + 'static,
    W: Write + ?Sized,
{
    serve::hush();
    run(input, w, parts)
}

/// Reads asks from `input` on a thread of its own and writes notes on `w`, until goodbye or the end of input.
///
/// # Errors
/// The frame that could not be read or written, or `Io` when the reader thread does not start.
pub fn run<R, W>(input: R, w: &mut W, parts: Parts) -> Result<(), IpcError>
where
    R: Read + Send + 'static,
    W: Write + ?Sized,
{
    let (tx, rx) = mpsc::channel();
    stream::listen::<VoiceAsk<'static>, _, _>(input, tx.clone(), READER)?;
    let (mut job, mut pipe) = (Job::new(parts, tx), Pipe(w));
    for event in rx {
        if let Flow::Exit(end) = step(&mut job, event, &mut pipe)? {
            return end;
        }
    }
    Ok(())
}

/// Takes one event. A panic inside wipes the job and is told as an internal fault on the ask's id and on the run it
/// served; goodbye or the end of input still ends it.
fn step(job: &mut Job, event: Event, out: &mut dyn Notes) -> Result<Flow, IpcError> {
    let (asked, busy, end) = (asked(&event), job.busy(), ending(&event));
    let taken = panic::catch_unwind(AssertUnwindSafe(|| job.take(event, &mut *out)));
    taken.unwrap_or_else(|_| {
        job.wipe();
        let fault = VoiceFault::Internal;
        let busy = busy.filter(|&id| asked != Some(id));
        for id in [asked, busy].into_iter().flatten() {
            out.tell(VoiceNote::Fault { id, fault })?;
        }
        Ok(end.map_or(Flow::Go, Flow::Exit))
    })
}

/// The id of the ask inside `event`; goodbye has none.
fn asked(event: &Event) -> Option<u32> {
    match event {
        Event::Ask(ask) if !matches!(ask, VoiceAsk::Bye) => Some(ask.id()),
        _ => None,
    }
}

/// What a goodbye or the end of input ends the job with; `None` for any other event.
fn ending(event: &Event) -> Option<Result<(), IpcError>> {
    match event {
        Event::Ask(VoiceAsk::Bye) => Some(Ok(())),
        Event::End(end) => Some(*end),
        _ => None,
    }
}

#[cfg(test)]
mod tests;
