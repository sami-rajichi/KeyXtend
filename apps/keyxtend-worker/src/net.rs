//! The speech job (ADR-0047 decisions 1 to 6): `keyxtend-worker speech-net` sends a dictation's pieces to the one
//! service a run names, over HTTPS with no redirect, and tells each piece's text in order, with no microphone. Asks
//! are read on a thread of their own and each request runs on one, so `Cancel` is answered while requests wait; the
//! key and the sound stay in wiped buffers and never reach a note, a log or stderr.

mod assembly;
mod base64;
pub mod clock;
mod eleven;
mod flag;
mod form;
mod gemini;
mod job;
mod multipart;
mod pace;
mod reason;
mod reply;
mod run;
mod send;
mod shape;
mod status;
mod wav;

use crate::stream::{self, Heard, Note, Pipe, Streamed};
use crate::{client, serve};
use clock::Clock;
use job::Job;
use kx_ipc::{IpcError, NetAsk, NetFault, NetNote};
use kx_platform_net::Http;
use std::io::{Read, Write};
use std::panic::{self, AssertUnwindSafe};
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use zeroize::Zeroizing;

/// The name of the thread that reads the asks.
const READER: &str = "speech-net-asks";

/// What every request thread shares: the client and the clock.
#[derive(Clone)]
pub struct Kit {
    /// The HTTPS client.
    pub http: Arc<dyn Http>,
    /// The job's time.
    pub clock: Arc<dyn Clock>,
}

/// What a request thread tells the job.
pub struct Told {
    /// The run or test.
    pub id: u32,
    /// The run's or test's serial, so an older one's outcome is never told.
    pub serial: u64,
    /// What it said.
    pub said: Said,
}

/// A request's outcome.
pub enum Said {
    /// A piece's text, by its index, or its fault.
    Piece(u16, Result<Zeroizing<String>, NetFault>),
    /// Whether a key works.
    Test(Result<(), NetFault>),
}

/// What wakes the job.
pub enum Event {
    /// An ask from the host, boxed as it is far larger than an outcome.
    Ask(Box<NetAsk<'static>>),
    /// A request's outcome.
    Told(Told),
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
    fn tell(&mut self, note: NetNote) -> Result<(), IpcError>;
}

impl<W: Write + ?Sized> Notes for Pipe<'_, W> {
    fn tell(&mut self, note: NetNote) -> Result<(), IpcError> {
        self.send(&note)
    }
}

/// A note that fails its own checks goes as `Gone` instead.
impl Note for NetNote {
    fn put<W: Write + ?Sized>(&self, w: &mut W) -> Result<(), IpcError> {
        self.write(w)
    }

    fn instead(&self) -> Self {
        let (id, fault) = (self.id(), NetFault::Gone);
        Self::Fault { id, fault }
    }
}

impl Streamed for NetAsk<'static> {
    fn read_from(r: &mut dyn Read) -> Result<Self, IpcError> {
        Self::read(r)
    }

    fn last(&self) -> bool {
        bye(self)
    }
}

impl From<Heard<NetAsk<'static>>> for Event {
    fn from(heard: Heard<NetAsk<'static>>) -> Self {
        match heard {
            Heard::Ask(ask) => Self::Ask(Box::new(ask)),
            Heard::End(end) => Self::End(end),
        }
    }
}

/// The kit the shipped job sends with: the system's client and the real clock.
pub fn live() -> Kit {
    Kit {
        http: Arc::from(client::live()),
        clock: Arc::new(clock::Live::new()),
    }
}

/// The speech job: `run` with every panic kept off stderr.
///
/// # Errors
/// As `run`.
pub fn serve<R, W>(input: R, w: &mut W, kit: Kit) -> Result<(), IpcError>
where
    R: Read + Send + 'static,
    W: Write + ?Sized,
{
    serve::hush();
    run(input, w, kit)
}

/// Reads asks from `input` on a thread of their own and sends over `kit`, telling notes on `w`, until goodbye or the
/// end of input; runs still live are dropped then.
///
/// # Errors
/// The frame that could not be read or written, or `Io` when the reader thread does not start.
pub fn run<R, W>(input: R, w: &mut W, kit: Kit) -> Result<(), IpcError>
where
    R: Read + Send + 'static,
    W: Write + ?Sized,
{
    let (tx, rx) = mpsc::channel();
    stream::listen::<NetAsk<'static>, _, _>(input, tx.clone(), READER)?;
    let (mut job, mut pipe) = (Job::new(kit, tx), Pipe(w));
    let end = loop {
        match act(&mut job, wake(&rx), &mut pipe) {
            Ok(Flow::Go) => {}
            Ok(Flow::Exit(end)) => break end,
            Err(e) => break Err(e),
        }
    };
    job.end();
    end
}

/// The next event; the job holds a sender itself, so the channel never ends and the `End` fallback only keeps this
/// total.
fn wake(rx: &Receiver<Event>) -> Event {
    rx.recv().unwrap_or(Event::End(Ok(())))
}

/// Takes `event`. A panic inside drops every run and test, told as `Gone` on the ask's id and on each live id.
fn act(job: &mut Job, event: Event, out: &mut dyn Notes) -> Result<Flow, IpcError> {
    let asked = match &event {
        Event::Ask(ask) if !bye(ask) => Some(ask.id()),
        _ => None,
    };
    let live = job.live();
    let taken = panic::catch_unwind(AssertUnwindSafe(|| match event {
        Event::Ask(ask) if bye(&ask) => Ok(Flow::Exit(Ok(()))),
        Event::End(end) => Ok(Flow::Exit(end)),
        Event::Ask(ask) => job.take(*ask, &mut *out).map(|()| Flow::Go),
        Event::Told(told) => job.told(told, &mut *out).map(|()| Flow::Go),
    }));
    let Ok(taken) = taken else {
        job.end();
        let live = live.into_iter().filter(|&id| asked != Some(id));
        for id in asked.into_iter().chain(live) {
            out.tell(NetNote::Fault {
                id,
                fault: NetFault::Gone,
            })?;
        }
        return Ok(Flow::Go);
    };
    taken
}

/// True when `ask` is the host's goodbye.
fn bye(ask: &NetAsk<'_>) -> bool {
    matches!(ask, NetAsk::Bye)
}

/// Runs `work` on a thread named `name` and tells `tx` its outcome, a panic as `Gone`, as `said` makes it for `id`
/// under `serial`.
///
/// # Errors
/// `Gone` when the thread does not start.
pub fn spawn<T: 'static>(
    name: &str,
    (id, serial): (u32, u64),
    tx: Sender<Event>,
    work: impl FnOnce() -> Result<T, NetFault> + Send + 'static,
    said: impl FnOnce(Result<T, NetFault>) -> Said + Send + 'static,
) -> Result<(), NetFault> {
    let body = move || {
        let got = panic::catch_unwind(AssertUnwindSafe(work)).unwrap_or(Err(NetFault::Gone));
        let said = said(got);
        let _ = tx.send(Event::Told(Told { id, serial, said }));
    };
    thread::Builder::new()
        .name(name.to_owned())
        .spawn(body)
        .map(drop)
        .map_err(|_| NetFault::Gone)
}

#[cfg(test)]
mod tests;
