//! The OS-neutral worker host (ADR-0004, ADR-0034 decision 5): a session sends one ask at a time within a deadline, a
//! stream (ADR-0042 decision 1) also tells the notes a worker sends on its own, and both end a late or lying worker,
//! start it again after a doubling backoff and stop it when idle. An adapter only starts the program.

mod backoff;
mod frames;
mod host;
mod piped;
mod session;
mod stream;

pub(crate) use backoff::ms;
pub(crate) use frames::sound_fault;
pub use frames::{
    FetchFrames, MediaFrames, NetFrames, PickFrames, RecordFrames, VaultFrames, VoiceFrames,
};
pub(crate) use host::Host;
pub use piped::{Child, Piped};
pub(crate) use session::Session;
pub(crate) use stream::Stream;
pub use stream::{Split, Streams, Told, Unsent, Wait};

use kx_ipc::{CLEAN_EXIT, IpcError, REFUSED_EXIT};
use std::io::{Read, Write};
use std::thread;
use std::time::Duration;
use thiserror::Error;

/// The idle time of a job that lives as long as its run does, reading or waiting, and never ends on its own.
pub(crate) const NEVER_IDLE_S: u32 = u32::MAX;

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

    /// The most payload bytes the reply to `ask` may carry, when it is not `read`'s own bound.
    fn most(_ask: &Self::Ask) -> Option<usize> {
        None
    }

    /// Reads one reply within `most` payload bytes when given, else as `read`.
    ///
    /// # Errors
    /// As `read`; `TooLong` past `most`.
    fn read_within<R: Read + ?Sized>(
        r: &mut R,
        most: Option<usize>,
    ) -> Result<Self::Reply, IpcError> {
        let _ = most;
        Self::read(r)
    }
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

    /// Ends the worker now, without waiting until it exited; harmless once it has ended.
    fn end(&mut self);

    /// Its exit code, waiting `within` at most for it to end; `None` while it runs.
    fn code(&mut self, within: Duration) -> Option<u32>;

    /// The most memory its job has taken, in MB; `None` when the system cannot tell.
    fn peak_mb(&mut self) -> Option<u32> {
        None
    }

    /// Lets the worker bring its own window to the front, as a system dialog needs; false when the system refuses.
    fn grant_front(&mut self) -> bool {
        false
    }
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

impl Times {
    /// How long the worker may sit idle.
    fn idle(self) -> Duration {
        Duration::from_secs(u64::from(self.idle_s))
    }
}

/// Says goodbye to `worker`, waits `within` at most for it to quit, then ends it; an unclean exit is logged.
fn bye<F: Frames, W: Worker<F>>(worker: &mut W, within: Duration) {
    worker.send(F::bye());
    let code = worker.code(within);
    worker.end();
    if code != Some(u32::from(CLEAN_EXIT)) {
        let note = exit_note(code);
        tracing::warn!(job = F::JOB, note, "a worker did not quit cleanly");
    }
}

/// Starts `body` as thread `name`, never waiting for it.
fn start(name: &str, body: impl FnOnce() + Send + 'static) -> Result<(), SpawnError> {
    let started = thread::Builder::new().name(name.to_owned()).spawn(body);
    started.map(drop).map_err(|e| {
        tracing::warn!(error = %e, thread = name, "a worker thread did not start");
        SpawnError::Thread
    })
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
