//! A worker over its two pipes: one thread writes each ask, so a worker that stops reading never blocks the host past
//! its deadline, and one reads each reply into a channel that holds none, so the host keeps one unasked frame at most.
//! Asks go to the pipe unbuffered and replies through a buffer wiped after each frame, so no password, part or value
//! is left in a buffer of ours.

mod wipe;

use super::{Frames, Heard, SpawnError, Worker};
use kx_ipc::IpcError;
use std::io::{Read, Write};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender, SyncSender};
use std::thread;
use std::time::Duration;
use wipe::Wipe;

/// What each pipe thread's name starts with, before the job's word.
const THREAD: &str = "kx";
/// The end of the thread's name that writes asks to the worker's stdin.
const WRITER: &str = "in";
/// The end of the thread's name that reads replies from the worker's stdout.
const READER: &str = "out";

/// The process behind a piped worker; dropping it ends the process.
pub trait Child: Send {
    /// Ends the process now; harmless once it has ended.
    fn end(&mut self);

    /// Its exit code, waiting `within` at most for it to end; `None` while it runs.
    fn code(&mut self, within: Duration) -> Option<u32>;
}

/// A worker that talks job `F`'s frames over its stdin and stdout. Dropping it lets its threads end and drops the
/// child.
pub struct Piped<C, F: Frames> {
    asks: Sender<F::Ask>,
    replies: Receiver<Result<F::Reply, IpcError>>,
    /// Dropped last, which ends the process and so the reader.
    child: C,
}

impl<C: Child, F: Frames> Piped<C, F> {
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
        let (writer, reader) = (named::<F>(WRITER), named::<F>(READER));
        start(&writer, move || write_asks::<F, W>(input, &todo))?;
        start(&reader, move || read_replies::<F, R>(output, &heard))?;
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

impl<C: Child, F: Frames> Worker<F> for Piped<C, F> {
    fn send(&mut self, ask: F::Ask) {
        // A writer that stopped means the worker ended, which the next reply tells.
        let _ = self.asks.send(ask);
    }

    fn reply(&mut self, within: Duration) -> Heard<F::Reply> {
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

/// Job `F`'s pipe thread name for `end`, such as `kx-vault-in`, so a dump tells the jobs apart.
fn named<F: Frames>(end: &str) -> String {
    format!("{THREAD}-{}-{end}", F::JOB)
}

/// Starts `body` as thread `name`, never waiting for it.
fn start(name: &str, body: impl FnOnce() + Send + 'static) -> Result<(), SpawnError> {
    let started = thread::Builder::new().name(name.to_owned()).spawn(body);
    started.map(drop).map_err(|e| {
        tracing::warn!(error = %e, thread = name, "a worker thread did not start");
        SpawnError::Thread
    })
}

/// Writes each ask in turn, straight to the pipe, until the host drops its sender or the stdin fails.
fn write_asks<F: Frames, W: Write>(mut input: W, todo: &Receiver<F::Ask>) {
    for ask in todo {
        if F::write(&ask, &mut input).is_err() {
            return;
        }
    }
}

/// Reads each reply in turn, handing it over only when the host waits for it, until a frame fails or the host leaves.
fn read_replies<F: Frames, R: Read>(output: R, heard: &SyncSender<Result<F::Reply, IpcError>>) {
    let mut output = Wipe::new(output);
    loop {
        let reply = F::read(&mut output);
        output.wipe();
        let last = reply.is_err();
        if heard.send(reply).is_err() || last {
            return;
        }
    }
}

#[cfg(test)]
mod tests;
