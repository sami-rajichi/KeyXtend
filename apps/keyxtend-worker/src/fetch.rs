//! The fetch job (ADR-0042, ADR-0043 decision 8): `keyxtend-worker fetch` downloads one model file at a time over
//! HTTPS into its `.part` file, with no microphone, and names it only once its size and SHA-256 pass. Asks are read on
//! a thread of their own, so Pause and Cancel are answered between reads, and notes go on stdout in order.

mod get;
mod hash;
mod job;

use crate::serve;
use job::Job;
use kx_ipc::{FetchAsk, FetchFault, FetchNote, IpcError};
use kx_platform_net::Http;
use std::io::{ErrorKind, Read, Write};
use std::panic::{self, AssertUnwindSafe};
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::thread;

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

/// The pipe: a note that fails its own checks goes as an internal fault instead.
struct Pipe<'w, W: Write + ?Sized>(&'w mut W);

impl<W: Write + ?Sized> Notes for Pipe<'_, W> {
    fn tell(&mut self, note: FetchNote) -> Result<(), IpcError> {
        match note.write(self.0) {
            Err(IpcError::Io(kind)) => Err(IpcError::Io(kind)),
            Err(_) => {
                let (id, fault) = (note.id(), FetchFault::Internal);
                FetchNote::Fault { id, fault }.write(self.0)
            }
            Ok(()) => Ok(()),
        }
    }
}

/// The client the shipped job downloads with: `WinHTTP` on Windows.
#[cfg(windows)]
pub fn live() -> Box<dyn Http> {
    Box::new(kx_platform_net::WinHttp::new())
}

/// The client elsewhere: every download fails as `Net` until P15 brings an adapter.
#[cfg(not(windows))]
pub fn live() -> Box<dyn Http> {
    Box::new(Offline)
}

/// A client with no network, for systems without an adapter yet; tests check it everywhere.
#[cfg(any(test, not(windows)))]
struct Offline;

#[cfg(any(test, not(windows)))]
impl Http for Offline {
    fn send(
        &self,
        _: &kx_platform_net::Hop<'_>,
    ) -> Result<kx_platform_net::Reply, kx_platform_net::NetError> {
        Err(kx_platform_net::NetError::Net)
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
    read(input, tx)?;
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

/// Reads asks from `input` on the reader thread until goodbye, the end of input or a broken frame.
fn read<R: Read + Send + 'static>(mut input: R, tx: Sender<Event>) -> Result<(), IpcError> {
    let reader = move || {
        loop {
            let event = next(&mut input);
            let last = matches!(event, Event::End(_) | Event::Ask(FetchAsk::Bye));
            if tx.send(event).is_err() || last {
                return;
            }
        }
    };
    thread::Builder::new()
        .name(READER.to_owned())
        .spawn(reader)
        .map(drop)
        .map_err(|e| IpcError::Io(e.kind()))
}

/// The next ask on `input`; the end of input, a broken frame or a panic inside the read ends the job.
fn next<R: Read>(input: &mut R) -> Event {
    match panic::catch_unwind(AssertUnwindSafe(|| FetchAsk::read(input))) {
        Ok(Ok(ask)) => Event::Ask(ask),
        Ok(Err(IpcError::Closed)) => Event::End(Ok(())),
        Ok(Err(e)) => Event::End(Err(e)),
        Err(_) => Event::End(Err(IpcError::Io(ErrorKind::Other))),
    }
}

#[cfg(test)]
mod tests;
