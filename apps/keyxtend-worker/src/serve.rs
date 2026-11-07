//! The worker's loop, one for every job: one ask in, one reply out, until goodbye or the end of input. A panicking
//! ask, or a reply that fails its own checks, is answered as an internal failure; a frame that cannot be read or
//! written ends the loop.

use kx_ipc::IpcError;
use std::io::{Read, Write};
use std::panic::{self, AssertUnwindSafe};

/// A job's asks, as the loop reads them.
pub trait Asked: Sized {
    /// The job's replies.
    type Reply: Answer;

    /// Reads one ask.
    ///
    /// # Errors
    /// The frame's `IpcError`; `Closed` when the input ended between frames.
    fn next<R: Read + ?Sized>(r: &mut R) -> Result<Self, IpcError>;

    /// The ask's id, or `None` for goodbye.
    fn id(&self) -> Option<u32>;
}

/// A job's replies, as the loop writes them.
pub trait Answer: Sized {
    /// The reply to ask `id` when it failed inside.
    fn internal(id: u32) -> Self;

    /// Writes this reply as one frame.
    ///
    /// # Errors
    /// `Io` when the stream fails; any other `IpcError` for a reply refused before any byte is written.
    fn send<W: Write + ?Sized>(&self, w: &mut W) -> Result<(), IpcError>;
}

/// A job's loop: `serve` with every ask's panic kept off stderr.
///
/// # Errors
/// As `serve`.
pub fn answer<A, R, W>(r: &mut R, w: &mut W, work: impl Fn(A) -> A::Reply) -> Result<(), IpcError>
where
    A: Asked,
    R: Read + ?Sized,
    W: Write + ?Sized,
{
    hush();
    serve(r, w, work)
}

/// Answers each ask read from `r` on `w` with `work`, one at a time, until goodbye or the end of input.
///
/// # Errors
/// The first frame that cannot be read or written; the stream is then out of step, so the worker exits.
pub fn serve<A, R, W>(r: &mut R, w: &mut W, work: impl Fn(A) -> A::Reply) -> Result<(), IpcError>
where
    A: Asked,
    R: Read + ?Sized,
    W: Write + ?Sized,
{
    loop {
        let ask = match A::next(r) {
            Ok(ask) => ask,
            Err(IpcError::Closed) => return Ok(()),
            Err(e) => return Err(e),
        };
        let Some(id) = ask.id() else {
            return Ok(());
        };
        // The ask's bytes drop inside `work`, before the reply is written.
        let reply = panic::catch_unwind(AssertUnwindSafe(|| work(ask)));
        send(w, &reply.unwrap_or_else(|_| A::Reply::internal(id)), id)?;
    }
}

/// Writes `reply`, or an internal failure in its place when it fails its own checks.
fn send<T: Answer, W: Write + ?Sized>(w: &mut W, reply: &T, id: u32) -> Result<(), IpcError> {
    match reply.send(w) {
        Err(IpcError::Io(kind)) => Err(IpcError::Io(kind)),
        Err(_) => T::internal(id).send(w),
        Ok(()) => Ok(()),
    }
}

/// Keeps an ask's panic off stderr: its reply says it failed inside, and its message and place are never written.
fn hush() {
    panic::set_hook(Box::new(|_| {}));
}

#[cfg(test)]
mod tests;
