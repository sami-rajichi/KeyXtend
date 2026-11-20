//! The fetch job (ADR-0042, ADR-0043 decision 8): `keyxtend-worker fetch` downloads one model file at a time over
//! HTTPS into its `.part` file, with no microphone, and names it only once its size and SHA-256 pass. Asks are read on
//! a thread of their own, so Pause and Cancel are answered between reads, and notes go on stdout in order.

mod get;
mod hash;
mod job;

use crate::serve;
use crate::stream::{self, Heard, Note, Pipe, Streamed};
use job::Job;
use kx_ipc::{FetchAsk, FetchFault, FetchNote, IpcError};
use kx_platform_net::Http;
use std::io::{Read, Write};
use std::panic::{self, AssertUnwindSafe};
use std::sync::mpsc::{self, Receiver, TryRecvError};

/// The name of the thread that reads the asks.
const READER: &str = "fetch-asks";

/// What wakes the job.
pub enum Event {
    /// An ask from the host.
    Ask(FetchAsk<'static>),
    /// The input ended cleanly, or with the frame that could not be read.
    End(Result<(), IpcError>),
}

/// What the loop does after an event or a step.
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
    fn tell(&mut self, note: FetchNote) -> Result<(), IpcError>;
}

impl<W: Write + ?Sized> Notes for Pipe<'_, W> {
    fn tell(&mut self, note: FetchNote) -> Result<(), IpcError> {
        self.send(&note)
    }
}

/// A note that fails its own checks goes as an internal fault instead.
impl Note for FetchNote {
    fn put<W: Write + ?Sized>(&self, w: &mut W) -> Result<(), IpcError> {
        self.write(w)
    }

    fn instead(&self) -> Self {
        let (id, fault) = (self.id(), FetchFault::Internal);
        Self::Fault { id, fault }
    }
}

impl Streamed for FetchAsk<'static> {
    fn read_from(r: &mut dyn Read) -> Result<Self, IpcError> {
        Self::read(r)
    }

    fn last(&self) -> bool {
        matches!(self, Self::Bye)
    }
}

impl From<Heard<FetchAsk<'static>>> for Event {
    fn from(heard: Heard<FetchAsk<'static>>) -> Self {
        match heard {
            Heard::Ask(ask) => Self::Ask(ask),
            Heard::End(end) => Self::End(end),
        }
    }
}

/// The fetch job: `run` with every panic kept off stderr.
///
/// # Errors
/// As `run`.
pub fn serve<R, W>(input: R, w: &mut W, http: Box<dyn Http>) -> Result<(), IpcError>
where
    R: Read + Send + 'static,
    W: Write + ?Sized,
{
    serve::hush();
    run(input, w, http)
}

/// Reads asks from `input` on a thread of their own and downloads over `http`, telling notes on `w`, until goodbye or
/// the end of input. A download's part stays at the end, for a later job to resume.
///
/// # Errors
/// The frame that could not be read or written, or `Io` when the reader thread does not start.
pub fn run<R, W>(input: R, w: &mut W, http: Box<dyn Http>) -> Result<(), IpcError>
where
    R: Read + Send + 'static,
    W: Write + ?Sized,
{
    let (tx, rx) = mpsc::channel();
    stream::listen::<FetchAsk<'static>, _, _>(input, tx, READER)?;
    let (mut job, mut pipe) = (Job::new(http), Pipe(w));
    let end = loop {
        let event = wake(&rx, job.busy());
        match act(&mut job, event, &mut pipe) {
            Ok(Flow::Go) => {}
            Ok(Flow::Exit(end)) => break end,
            Err(e) => break Err(e),
        }
    };
    job.end();
    end
}

/// The next event: waits for one while no download reads, else takes one only if it has come, so reads go on.
fn wake(rx: &Receiver<Event>, busy: bool) -> Option<Event> {
    let gone = || Event::End(Ok(()));
    if !busy {
        return Some(rx.recv().unwrap_or_else(|_| gone()));
    }
    match rx.try_recv() {
        Ok(event) => Some(event),
        Err(TryRecvError::Empty) => None,
        Err(TryRecvError::Disconnected) => Some(gone()),
    }
}

/// Takes `event`, or steps the download when there is none. A panic inside drops the download, keeping its part,
/// and is told as an internal fault on the ask's id and on the download it touched.
fn act(job: &mut Job, event: Option<Event>, out: &mut dyn Notes) -> Result<Flow, IpcError> {
    let ask = match event {
        Some(Event::Ask(FetchAsk::Bye)) => return Ok(Flow::Exit(Ok(()))),
        Some(Event::End(end)) => return Ok(Flow::Exit(end)),
        Some(Event::Ask(ask)) => Some(ask),
        None => None,
    };
    let (asked, live) = (ask.as_ref().map(FetchAsk::id), job.live());
    let taken = panic::catch_unwind(AssertUnwindSafe(|| match ask {
        Some(ask) => job.take(ask, &mut *out),
        None => job.work(&mut *out),
    }));
    let Ok(taken) = taken else {
        job.end();
        let live = live.filter(|&id| asked != Some(id));
        for id in [asked, live].into_iter().flatten() {
            let fault = FetchFault::Internal;
            out.tell(FetchNote::Fault { id, fault })?;
        }
        return Ok(Flow::Go);
    };
    taken.map(|()| Flow::Go)
}

#[cfg(test)]
mod tests;
