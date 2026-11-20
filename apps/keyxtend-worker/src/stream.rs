//! What the streaming jobs share: asks read on a thread of their own until goodbye or the end of input, and a pipe
//! that sends the job's fault in place of a note that fails its own checks.

#[cfg(test)]
pub mod pipes;

use kx_ipc::IpcError;
use std::io::{ErrorKind, Read, Write};
use std::panic::{self, AssertUnwindSafe};
use std::sync::mpsc::Sender;
use std::thread;

/// A streaming job's asks, as its reader thread reads them.
pub trait Streamed: Sized + Send + 'static {
    /// Reads one ask.
    ///
    /// # Errors
    /// The frame's `IpcError`; `Closed` when the input ended between frames.
    fn read_from(r: &mut dyn Read) -> Result<Self, IpcError>;

    /// True for goodbye, after which nothing more is read.
    fn last(&self) -> bool;
}

/// What the reader hands its job: an ask, or the end of input, clean or with the frame that could not be read.
pub enum Heard<A> {
    /// An ask.
    Ask(A),
    /// The end of input.
    End(Result<(), IpcError>),
}

/// A streaming job's notes, as its pipe writes them.
pub trait Note: Sized {
    /// Writes this note as one frame.
    ///
    /// # Errors
    /// `Io` when the stream fails; any other `IpcError` for a note refused before any byte is written.
    fn put<W: Write + ?Sized>(&self, w: &mut W) -> Result<(), IpcError>;

    /// The job's fault on this note's id, sent in its place when it fails its own checks.
    fn instead(&self) -> Self;
}

/// The pipe a job's notes go down.
pub struct Pipe<'w, W: Write + ?Sized>(pub &'w mut W);

impl<W: Write + ?Sized> Pipe<'_, W> {
    /// Writes `note`, or the job's fault in its place when it fails its own checks.
    ///
    /// # Errors
    /// The pipe's `IpcError` when it fails.
    pub fn send<N: Note>(&mut self, note: &N) -> Result<(), IpcError> {
        match note.put(self.0) {
            Err(IpcError::Io(kind)) => Err(IpcError::Io(kind)),
            Err(_) => note.instead().put(self.0),
            Ok(()) => Ok(()),
        }
    }
}

/// Reads asks from `input` on a thread named `name`, sending each to `tx`, until goodbye, the end of input or a broken
/// frame.
///
/// # Errors
/// `Io` when the thread does not start.
pub fn listen<A, E, R>(mut input: R, tx: Sender<E>, name: &str) -> Result<(), IpcError>
where
    A: Streamed,
    E: From<Heard<A>> + Send + 'static,
    R: Read + Send + 'static,
{
    let reader = move || {
        loop {
            let heard = next::<A>(&mut input);
            let last = match &heard {
                Heard::Ask(ask) => ask.last(),
                Heard::End(_) => true,
            };
            if tx.send(E::from(heard)).is_err() || last {
                return;
            }
        }
    };
    thread::Builder::new()
        .name(name.to_owned())
        .spawn(reader)
        .map(drop)
        .map_err(|e| IpcError::Io(e.kind()))
}

/// The next ask on `input`; the end of input, a broken frame or a panic inside the read ends the job.
fn next<A: Streamed>(input: &mut dyn Read) -> Heard<A> {
    match panic::catch_unwind(AssertUnwindSafe(|| A::read_from(input))) {
        Ok(Ok(ask)) => Heard::Ask(ask),
        Ok(Err(IpcError::Closed)) => Heard::End(Ok(())),
        Ok(Err(e)) => Heard::End(Err(e)),
        Err(_) => Heard::End(Err(IpcError::Io(ErrorKind::Other))),
    }
}
