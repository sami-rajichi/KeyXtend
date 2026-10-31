//! A worker over its two pipes: one thread writes each ask, so a worker that stops reading never blocks the host past
//! its deadline, and one reads each reply into a channel that holds none, so the host keeps one unasked frame at most.

use super::{Heard, SpawnError, Worker};
use kx_ipc::{Ask, IpcError, Reply};
use std::io::{BufReader, BufWriter, Read, Write};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender, SyncSender};
use std::thread;
use std::time::Duration;

/// The thread that writes asks to the worker's stdin.
const WRITER: &str = "kx-pictures-in";
/// The thread that reads replies from the worker's stdout.
const READER: &str = "kx-pictures-out";

/// The process behind a piped worker; dropping it ends the process.
pub trait Child: Send {
    /// Ends the process now; harmless once it has ended.
    fn end(&mut self);

    /// Its exit code, waiting `within` at most for it to end; `None` while it runs.
    fn code(&mut self, within: Duration) -> Option<u32>;
}

/// A worker that talks `kx-ipc` frames over its stdin and stdout. Dropping it lets its threads end and drops the child.
pub struct Piped<C> {
    asks: Sender<Ask<'static>>,
    replies: Receiver<Result<Reply, IpcError>>,
    /// Dropped last, which ends the process and so the reader.
    child: C,
}

impl<C: Child> Piped<C> {
    /// Drives `child` through `input`, its stdin, and `output`, its stdout, each on a thread of its own.
    ///
    /// # Errors
    /// `Thread` when a thread does not start; `child` is then dropped, which ends it.
    pub fn new<W, R>(input: W, output: R, child: C) -> Result<Self, SpawnError>
    where
        W: Write + Send + 'static,
        R: Read + Send + 'static,
    {
        let (asks, todo) = mpsc::channel();
        let (heard, replies) = mpsc::sync_channel(0);
        start(WRITER, move || write_asks(input, &todo))?;
        start(READER, move || read_replies(output, &heard))?;
        Ok(Self {
            asks,
            replies,
            child,
        })
    }

    /// The process.
    pub const fn child(&self) -> &C {
        &self.child
    }
}

impl<C: Child> Worker for Piped<C> {
    fn send(&mut self, ask: Ask<'static>) {
        // A writer that stopped means the worker ended, which the next reply tells.
        let _ = self.asks.send(ask);
    }

    fn reply(&mut self, within: Duration) -> Heard {
        match self.replies.recv_timeout(within) {
            Ok(frame) => Heard::Frame(frame),
            Err(RecvTimeoutError::Timeout) => Heard::Late,
            Err(RecvTimeoutError::Disconnected) => Heard::Frame(Err(IpcError::Closed)),
        }
    }

    fn end(&mut self) {
        self.child.end();
    }

    fn code(&mut self, within: Duration) -> Option<u32> {
        self.child.code(within)
    }
}

/// Starts `body` as thread `name`, never waiting for it.
fn start(name: &str, body: impl FnOnce() + Send + 'static) -> Result<(), SpawnError> {
    let started = thread::Builder::new().name(name.to_owned()).spawn(body);
    started.map(drop).map_err(|e| {
        tracing::warn!(error = %e, thread = name, "a picture worker thread did not start");
        SpawnError::Thread
    })
}

/// Writes each ask in turn until the host drops its sender or the stdin fails.
fn write_asks<W: Write>(input: W, todo: &Receiver<Ask<'static>>) {
    let mut input = BufWriter::new(input);
    for ask in todo {
        if ask.write(&mut input).is_err() {
            return;
        }
    }
}

/// Reads each reply in turn, handing it over only when the host waits for it, until a frame fails or the host leaves.
fn read_replies<R: Read>(output: R, heard: &SyncSender<Result<Reply, IpcError>>) {
    let mut output = BufReader::new(output);
    loop {
        let reply = Reply::read(&mut output);
        let last = reply.is_err();
        if heard.send(reply).is_err() || last {
            return;
        }
    }
}

#[cfg(test)]
mod tests;
