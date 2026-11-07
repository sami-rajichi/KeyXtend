//! The OS-neutral worker host (ADR-0004, ADR-0034 decision 5), one core for every job: a port's session thread starts
//! its job on demand, sends one ask at a time within a deadline, checks each reply, ends a late or lying worker and
//! starts it again after a doubling backoff, and stops it when idle. An adapter only starts the program.

mod frames;
mod host;
mod piped;
mod session;

pub use frames::{MediaFrames, VaultFrames};
pub(crate) use host::Host;
pub use piped::{Child, Piped};
pub(crate) use session::Session;

use kx_ipc::{CLEAN_EXIT, IpcError, REFUSED_EXIT};
use std::io::{Read, Write};
use std::time::Duration;
use thiserror::Error;

/// How to start the worker: its command line after the program, and the most memory it may take.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Launch {
    /// The arguments, in order.
    pub args: Vec<String>,
    /// The memory cap, in MB.
    pub memory_mb: u32,
}

/// Why a worker did not start; no variant names a path.
#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum SpawnError {
    /// The worker program is not where it should be.
    #[error("the worker program is missing")]
    Missing,
    /// The system refused a step of the start, with its error code.
    #[error("the system refused to {step}: code {code:#x}")]
    Refused {
        /// What the step does.
        step: &'static str,
        /// The system's error code.
        code: i32,
    },
    /// The started worker kept rights it must not have, so it was ended.
    #[error("the worker kept rights it must not have")]
    Rights,
    /// A thread that drives the worker did not start.
    #[error("a worker thread did not start")]
    Thread,
}

/// One job's frames, as the host writes its asks and reads its replies.
pub trait Frames: 'static {
    /// The job's word on the command line, which names it in the log.
    const JOB: &'static str;
    /// What the host sends.
    type Ask: Send + 'static;
    /// What the job answers.
    type Reply: Send + 'static;

    /// Goodbye, after which the job exits.
    fn bye() -> Self::Ask;

    /// The id of the ask `reply` answers.
    fn id(reply: &Self::Reply) -> u32;

    /// Writes `ask` as one frame, then flushes.
    ///
    /// # Errors
    /// The frame's `IpcError`, before any byte is written for an ask the job would refuse.
    fn write<W: Write + ?Sized>(ask: &Self::Ask, w: &mut W) -> Result<(), IpcError>;

    /// Reads one reply.
    ///
    /// # Errors
    /// The frame's `IpcError`; `Closed` when the job ended between frames.
    fn read<R: Read + ?Sized>(r: &mut R) -> Result<Self::Reply, IpcError>;
}

/// What a wait for the worker's reply heard.
#[derive(Debug, PartialEq, Eq)]
pub enum Heard<R> {
    /// A frame, or why none was read; `Closed` once the worker ended.
    Frame(Result<R, IpcError>),
    /// Nothing within the wait.
    Late,
}

/// One started worker of job `F`, as its host drives it.
pub trait Worker<F: Frames>: Send {
    /// Hands `ask` to the worker without waiting for it to be read.
    fn send(&mut self, ask: F::Ask);

    /// The worker's next reply, waiting `within` at most.
    fn reply(&mut self, within: Duration) -> Heard<F::Reply>;

    /// Ends the worker now; harmless once it has ended.
    fn end(&mut self);

    /// Its exit code, waiting `within` at most for it to end; `None` while it runs.
    fn code(&mut self, within: Duration) -> Option<u32>;
}

/// What an OS adapter gives the host: the worker program, started with reduced rights, talking job `F`'s frames.
pub trait Spawner<F: Frames>: Send + Sync + 'static {
    /// A started worker; dropping it ends the worker.
    type Worker: Worker<F> + 'static;

    /// Starts the worker program as `launch` says.
    ///
    /// # Errors
    /// `Missing` without the program, `Refused` or `Rights` when it cannot start safely, `Thread` without its threads.
    fn spawn(&self, launch: &Launch) -> Result<Self::Worker, SpawnError>;
}

/// Why the host has no answer: the worker was late, failed, or rests before its next start.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Lost {
    /// No reply within the deadline, so the worker was ended.
    Late,
    /// The worker ended, or answered out of step or badly, so it was ended.
    Gone,
    /// No worker runs: it did not start, or the backoff still runs for `retry_in`.
    Down {
        /// How long until a start is tried again.
        retry_in: Duration,
    },
}

/// The host's times, which each port takes from its config.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) struct Times {
    /// The longest one ask may take, in ms, before the worker is ended.
    pub deadline_ms: u32,
    /// How long the worker may sit idle before it stops, in seconds.
    pub idle_s: u32,
    /// The first wait before a failed worker starts again, in ms; it doubles with each failure in a row.
    pub restart_ms: u32,
    /// The longest such wait, in ms.
    pub restart_max_ms: u32,
}

/// What a worker's exit code says, for the log.
#[must_use]
pub fn exit_note(code: Option<u32>) -> &'static str {
    match code.map(u8::try_from) {
        Some(Ok(CLEAN_EXIT)) => "it quit",
        Some(Ok(REFUSED_EXIT)) => "it refused its command line or a frame",
        Some(_) => "it died",
        None => "it still runs",
    }
}

#[cfg(test)]
pub(crate) mod fake;
#[cfg(test)]
mod tests;
