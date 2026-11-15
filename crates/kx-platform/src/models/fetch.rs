//! One download in a `fetch` job of its own, capped at `job_mb`, after a check of the drive's room; its notes go to
//! the run's callback until the run drops, which pauses it. A cancel the job can no longer take deletes the part here.
//! A paused download keeps its idle job, which holds the open download until it resumes, cancels or drops.

use super::{Disk, fault};
use crate::ear::Ear;
use crate::worker::{FetchFrames, Launch, Spawner, Split, Stream, Times, Told, Unsent, Wait};
use kx_ipc::{FETCH_JOB, FetchAsk as Get, FetchNote as Note};
use kx_module_api::Clock;
use kx_module_api::ports::{FetchAsk, FetchNote, FetchRun, JobLimits, ModelError};
use std::borrow::Cow;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

/// A download job's stream thread.
const THREAD: &str = "kx-fetch";
/// Bytes in one MB of `min_free_mb`.
const MIB: u64 = 1 << 20;
/// A download's job never ends on its own when idle: it lives while its run does, reading or paused.
const NEVER_IDLE_S: u32 = u32::MAX;
/// The value `start` names for a file whose part has no name, which `FetchAsk::check` already refuses.
const FILE: &str = "file";

/// A download the worker `S` runs.
pub(super) struct WorkerFetch<S: Spawner<FetchFrames, Worker: Split<FetchFrames>>> {
    stream: Stream<S, FetchFrames>,
    id: u32,
    part: PathBuf,
    ear: Arc<Ear<FetchNote>>,
    /// The download told its end, its file named or a fault.
    ended: Arc<AtomicBool>,
    /// Paused until resumed, so progress still on its way is not told, as the fake tells none.
    paused: Arc<AtomicBool>,
}

impl<S: Spawner<FetchFrames, Worker: Split<FetchFrames>>> WorkerFetch<S> {
    /// Starts the checked `ask` in a job of its own once its drive has room for what is still to come.
    pub(super) fn start(
        spawner: Arc<S>,
        clock: Arc<dyn Clock>,
        disk: &dyn Disk,
        ask: &FetchAsk,
        tell: Box<dyn Fn(FetchNote) + Send>,
    ) -> Result<Self, ModelError> {
        let part = ask.part().ok_or(ModelError::Range(FILE))?;
        room(disk, ask, &part)?;
        let get = get(ask)?;
        let (ear, ended) = (Arc::new(Ear::new(tell)), Arc::new(AtomicBool::new(false)));
        let paused = Arc::new(AtomicBool::new(false));
        let hear = {
            let (ear, ended, paused) = (Arc::clone(&ear), Arc::clone(&ended), Arc::clone(&paused));
            move |told| heard(&ear, (&ended, &paused), &told)
        };
        let (times, launch) = (times(&ask.job), job(&ask.job));
        let stream = Stream::open(THREAD, spawner, clock, times, launch, hear);
        let wait = Wait::Once(Duration::from_millis(u64::from(ask.job.ask_ms)));
        let id = stream.begin(get, wait).map_err(unsent)?;
        Ok(Self {
            stream,
            id,
            part,
            ear,
            ended,
            paused,
        })
    }

    /// Sends `ask` unwatched while the download lives, as the job tells nothing back for it.
    fn send(&self, ask: Get<'static>) {
        if !self.ended.load(Ordering::Acquire) {
            let _ = self.stream.ask(ask, Wait::Off);
        }
    }
}

impl<S: Spawner<FetchFrames, Worker: Split<FetchFrames>>> FetchRun for WorkerFetch<S> {
    fn pause(&mut self) {
        self.paused.store(true, Ordering::Release);
        self.send(Get::Pause { id: self.id });
    }

    fn resume(&mut self) {
        self.paused.store(false, Ordering::Release);
        self.send(Get::Resume { id: self.id });
    }

    /// The job deletes the part, a live download's or one a fault left; once its job is gone, the part goes here.
    fn cancel(&mut self) {
        let cancel = Get::Cancel { id: self.id };
        if self.stream.ask(cancel, Wait::Off).is_err() {
            let _ = fs::remove_file(&self.part);
        }
    }
}

impl<S: Spawner<FetchFrames, Worker: Split<FetchFrames>>> Drop for WorkerFetch<S> {
    /// Pauses the download and tells nothing more; the job then says goodbye and keeps the part.
    fn drop(&mut self) {
        self.ear.close();
        self.send(Get::Pause { id: self.id });
    }
}

/// Refuses `ask` when its drive has less room than `min_free_mb` and the bytes its part still lacks.
fn room(disk: &dyn Disk, ask: &FetchAsk, part: &Path) -> Result<(), ModelError> {
    let folder = ask.file.expose().parent().ok_or(ModelError::Missing)?;
    let free = disk.free(folder).map_err(|e| fault(&e))?;
    let have = fs::metadata(part).map_or(0, |m| m.len());
    let keep = u64::from(ask.min_free_mb).saturating_mul(MIB);
    let need = keep.saturating_add(ask.bytes.saturating_sub(have));
    if free < need {
        Err(ModelError::Full)
    } else {
        Ok(())
    }
}

/// The Get frame `ask` is sent as, for the id the stream gives it.
fn get(ask: &FetchAsk) -> Result<impl FnOnce(u32) -> Get<'static> + Send + 'static, ModelError> {
    let file = ask.file.expose().to_str().ok_or(ModelError::Range(FILE))?;
    let urls = ask
        .links
        .iter()
        .map(|l| Cow::Owned(l.expose().to_owned()))
        .collect();
    let hosts = ask.hosts.iter().map(|h| Cow::Owned(h.clone())).collect();
    let (path, size, sha256, stall_s) = (
        Cow::Owned(file.to_owned()),
        ask.bytes,
        ask.print.bytes(),
        ask.stall_s,
    );
    Ok(move |id| Get::Get {
        id,
        urls,
        hosts,
        path,
        size,
        sha256,
        stall_s,
    })
}

/// Tells what the stream heard as the port's note; the end once, no progress while paused, and nothing of a job lost
/// after the end.
pub(super) fn heard(
    ear: &Ear<FetchNote>,
    (ended, paused): (&AtomicBool, &AtomicBool),
    told: &Told<Note>,
) {
    let note = match told {
        Told::Note(Note::Progress { .. }) if paused.load(Ordering::Acquire) => return,
        &Told::Note(Note::Progress { got, total, .. }) => FetchNote::Progress { done: got, total },
        Told::Note(Note::Done { .. }) => FetchNote::Done,
        &Told::Note(Note::Fault { fault, .. }) => FetchNote::Fault(fault.into()),
        Told::Lost(_) | Told::Idle if ended.load(Ordering::Acquire) => return,
        Told::Lost(_) | Told::Idle => FetchNote::Fault(ModelError::Gone),
    };
    if !matches!(note, FetchNote::Progress { .. }) {
        ended.store(true, Ordering::Release);
    }
    ear.tell([note]);
}

/// Why a Get was not sent: `Link` and `check` already hold the frames' rules, so a refusal would be an address the
/// frames still refuse; any other is the job not starting.
const fn unsent(e: Unsent) -> ModelError {
    match e {
        Unsent::Refused(_) => ModelError::Host,
        Unsent::Lost(_) => ModelError::Gone,
    }
}

/// A download job's times: `job`'s, but it never idles out.
const fn times(job: &JobLimits) -> Times {
    Times {
        deadline_ms: job.ask_ms,
        idle_s: NEVER_IDLE_S,
        restart_ms: job.restart_ms,
        restart_max_ms: job.restart_max_ms,
    }
}

/// The fetch job capped at `job_mb`.
fn job(job: &JobLimits) -> Launch {
    Launch {
        args: vec![FETCH_JOB.to_owned()],
        memory_mb: job.job_mb,
    }
}
