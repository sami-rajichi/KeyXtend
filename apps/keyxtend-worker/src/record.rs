//! The record job (ADR-0056 decisions 3 to 5): `keyxtend-worker record` films a part of the screen with the microphone
//! and the PC's sound into an MP4 file, and writes re-timed copies, with no network. Asks are read on a thread of their
//! own and each piece of work runs on another, one at a time, so switches, Stop and Cancel reach a run between frames;
//! notes go on stdout in order, and none holds a pixel, a sound or a path.

mod job;

use crate::serve;
use crate::stream::{self, Heard, Note, Pipe, Streamed};
use job::Job;
use kx_ipc::{IpcError, RecordAsk, RecordFault, RecordNote};
use kx_platform_record::record::Studio;
use std::io::{Read, Write};
use std::sync::mpsc;

/// The name of the thread that reads the asks.
const READER: &str = "record-asks";

/// What wakes the job.
pub enum Event {
    /// An ask from the host.
    Ask(RecordAsk<'static>),
    /// A note the work told while it goes on.
    Note(RecordNote),
    /// The work `id` ended, all its parts closed, with its last note: none after Cancel.
    Done(u32, Option<RecordNote>),
    /// The input ended cleanly, or with the frame that could not be read.
    End(Result<(), IpcError>),
}

/// What the loop does after an event.
pub enum Flow {
    /// Goes on.
    Go,
    /// Ends with this outcome: goodbye or the end of input, or the frame that could not be read.
    Exit(Result<(), IpcError>),
}

/// Where the job's notes go: the pipe, or a list in tests.
pub trait Notes {
    /// Sends `note`.
    ///
    /// # Errors
    /// The pipe's `IpcError` when it fails.
    fn tell(&mut self, note: RecordNote) -> Result<(), IpcError>;
}

impl<W: Write + ?Sized> Notes for Pipe<'_, W> {
    fn tell(&mut self, note: RecordNote) -> Result<(), IpcError> {
        self.send(&note)
    }
}

/// A note that fails its own checks goes as a failure instead.
impl Note for RecordNote {
    fn put<W: Write + ?Sized>(&self, w: &mut W) -> Result<(), IpcError> {
        self.write(w)
    }

    fn instead(&self) -> Self {
        let (id, fault) = (self.id(), RecordFault::Failed);
        Self::Fault { id, fault }
    }
}

impl Streamed for RecordAsk<'static> {
    fn read_from(r: &mut dyn Read) -> Result<Self, IpcError> {
        Self::read(r)
    }

    fn last(&self) -> bool {
        matches!(self, Self::Bye)
    }
}

impl From<Heard<RecordAsk<'static>>> for Event {
    fn from(heard: Heard<RecordAsk<'static>>) -> Self {
        match heard {
            Heard::Ask(ask) => Self::Ask(ask),
            Heard::End(end) => Self::End(end),
        }
    }
}

/// The record job: `run` with every panic kept off stderr.
///
/// # Errors
/// As `run`.
pub fn serve<R, W, S>(input: R, w: &mut W, studio: S) -> Result<(), IpcError>
where
    R: Read + Send + 'static,
    W: Write + ?Sized,
    S: Studio + Clone,
{
    serve::hush();
    run(input, w, studio)
}

/// Reads asks from `input` on a thread of their own and works with `studio`'s parts, telling notes on `w`, until
/// goodbye or the end of input; a run still going is stopped first and keeps its video.
///
/// # Errors
/// The frame that could not be read or written, or `Io` when the reader thread does not start.
pub fn run<R, W, S>(input: R, w: &mut W, studio: S) -> Result<(), IpcError>
where
    R: Read + Send + 'static,
    W: Write + ?Sized,
    S: Studio + Clone,
{
    let (tx, rx) = mpsc::channel();
    stream::listen::<RecordAsk<'static>, _, _>(input, tx.clone(), READER)?;
    let (mut job, mut pipe) = (Job::new(studio, tx), Pipe(w));
    let end = loop {
        // The job holds a sender, so the channel never closes while it waits.
        let event = rx.recv().unwrap_or(Event::End(Ok(())));
        match job.take(event, &mut pipe) {
            Ok(Flow::Go) => {}
            Ok(Flow::Exit(end)) => break end,
            Err(e) => break Err(e),
        }
    };
    job.end();
    end
}

#[cfg(test)]
mod tests;
