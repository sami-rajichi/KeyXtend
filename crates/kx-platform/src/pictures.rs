//! The OS-neutral worker host behind `Pictures` (ADR-0004, ADR-0034 decision 5): each session's `kx-pictures` thread
//! starts the media worker on demand, sends one draw at a time and checks each reply. An adapter only starts it.

mod host;
mod piped;

pub use piped::{Child, Piped};

use host::Host;
use kx_ipc::{Ask, CLEAN_EXIT, IpcError, MAX_PIXELS_FLAG, MEDIA_JOB, REFUSED_EXIT, Reply};
use kx_module_api::Clock;
use kx_module_api::ports::{PictureAsk, PictureConfig, PictureError, PictureRun, Pictures, Pixels};
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::thread;
use std::time::Duration;
use thiserror::Error;

/// The session thread's name.
const THREAD: &str = "kx-pictures";

/// How to start the worker: its command line after the program, and the most memory it may take.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Launch {
    /// The arguments, in order.
    pub args: Vec<String>,
    /// The memory cap, in MB.
    pub memory_mb: u32,
}

impl Launch {
    /// The media job as `cfg` says: `media --max-pixels <n>`, with n at least 1.
    #[must_use]
    pub fn media(cfg: &PictureConfig) -> Self {
        let max = cfg.max_pixels.max(1).to_string();
        Self {
            args: vec![MEDIA_JOB.to_owned(), MAX_PIXELS_FLAG.to_owned(), max],
            memory_mb: cfg.memory_mb,
        }
    }
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

/// What a wait for the worker's reply heard.
#[derive(Debug, PartialEq, Eq)]
pub enum Heard {
    /// A frame, or why none was read; `Closed` once the worker ended.
    Frame(Result<Reply, IpcError>),
    /// Nothing within the wait.
    Late,
}

/// One started worker, as its host drives it.
pub trait Worker: Send {
    /// Hands `ask` to the worker without waiting for it to be read.
    fn send(&mut self, ask: Ask<'static>);

    /// The worker's next reply, waiting `within` at most.
    fn reply(&mut self, within: Duration) -> Heard;

    /// Ends the worker now; harmless once it has ended.
    fn end(&mut self);

    /// Its exit code, waiting `within` at most for it to end; `None` while it runs.
    fn code(&mut self, within: Duration) -> Option<u32>;
}

/// What an OS adapter gives the host: the worker program, started with reduced rights.
pub trait Spawner: Send + Sync + 'static {
    /// A started worker; dropping it ends the worker.
    type Worker: Worker + 'static;

    /// Starts the worker program as `launch` says.
    ///
    /// # Errors
    /// `Missing` without the program, `Refused` or `Rights` when it cannot start safely, `Thread` without its threads.
    fn spawn(&self, launch: &Launch) -> Result<Self::Worker, SpawnError>;
}

/// Pictures drawn by the worker `S` starts, one `kx-pictures` thread per session.
pub struct WorkerPictures<S> {
    spawner: Arc<S>,
    clock: Arc<dyn Clock>,
}

impl<S: Spawner> WorkerPictures<S> {
    /// Pictures over `spawner`, whose sessions read the time from `clock`.
    #[must_use]
    pub fn new(spawner: S, clock: Arc<dyn Clock>) -> Self {
        Self {
            spawner: Arc::new(spawner),
            clock,
        }
    }
}

impl<S: Spawner> Pictures for WorkerPictures<S> {
    fn open(&self, cfg: PictureConfig) -> Box<dyn PictureRun> {
        let host = Host::new(Arc::clone(&self.spawner), Arc::clone(&self.clock), cfg);
        let (jobs, inbox) = mpsc::channel();
        let started = thread::Builder::new()
            .name(THREAD.to_owned())
            .spawn(move || serve(host, &inbox));
        let jobs = started
            .inspect_err(
                |e| tracing::warn!(error = %e, "no pictures: the session thread did not start"),
            )
            .ok()
            .map(|_| jobs);
        Box::new(Session { jobs })
    }
}

/// One draw for the session thread, and where its answer goes.
struct Job {
    ask: PictureAsk,
    answer: Sender<Result<Pixels, PictureError>>,
}

/// A session; dropping it ends its thread, which stops the worker.
struct Session {
    /// `None` when the thread did not start; every draw is then `Gone`.
    jobs: Option<Sender<Job>>,
}

impl PictureRun for Session {
    fn draw(&mut self, ask: &PictureAsk) -> Result<Pixels, PictureError> {
        let jobs = self.jobs.as_ref().ok_or(PictureError::Gone)?;
        let (answer, heard) = mpsc::channel();
        let job = Job {
            ask: ask.clone(),
            answer,
        };
        jobs.send(job).map_err(|_| PictureError::Gone)?;
        heard.recv().unwrap_or(Err(PictureError::Gone))
    }
}

/// The session thread: it draws each job, stops an idle worker, and ends with the session.
fn serve<S: Spawner>(mut host: Host<S>, inbox: &Receiver<Job>) {
    loop {
        let next = match host.wake_in() {
            Some(wait) => inbox.recv_timeout(wait),
            None => inbox.recv().map_err(|_| RecvTimeoutError::Disconnected),
        };
        match next {
            Ok(job) => {
                let _ = job.answer.send(host.draw(job.ask));
            }
            Err(RecvTimeoutError::Timeout) => host.idle(),
            Err(RecvTimeoutError::Disconnected) => return,
        }
    }
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
mod fake;
#[cfg(test)]
mod tests;
