//! The gate worker: the product's own spawner (`WinSpawner`), its token and job, capped from its start in weak mode,
//! with the clip after the voice job's words. Each start and end is counted, so the bench measures one worker at a time.
#![cfg(windows)]

use std::path::Path;
use std::sync::{Arc, Condvar, Mutex, MutexGuard, PoisonError};
use std::time::Duration;

use kx_module_api::ports::Memory;
use kx_platform::speech::Hardware;
use kx_platform::worker::{Child, Launch, Piped, SpawnError, Spawner, VoiceFrames};
use kx_platform_windows::{WinChild, WinSpawner};

use crate::g22cfg::is_part;

/// The workers started and not yet ended.
#[derive(Default)]
pub(super) struct Alive {
    count: Mutex<usize>,
    gone: Condvar,
}

impl Alive {
    /// Counts a worker that started.
    pub(super) fn up(&self) {
        *lock(&self.count) += 1;
    }

    /// Counts a worker that ended.
    pub(super) fn down(&self) {
        let mut count = lock(&self.count);
        *count = count.saturating_sub(1);
        self.gone.notify_all();
    }

    /// True once no worker runs, waiting `within` at most.
    pub(super) fn none_within(&self, within: Duration) -> bool {
        let count = lock(&self.count);
        let left = self.gone.wait_timeout_while(count, within, |n| *n > 0);
        let (count, _) = left.unwrap_or_else(PoisonError::into_inner);
        *count == 0
    }
}

/// Refuses a weak worker whose folder holds a processor part beyond `weak`, or none of them, as the engine loads the
/// best part it finds beside the worker.
pub(super) fn plain_parts(worker: &Path, prefix: &str, weak: &[String]) -> Result<(), String> {
    let dir = worker.parent().ok_or("the weak worker has no folder")?;
    let list = std::fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let names: Vec<String> = list
        .filter_map(Result::ok)
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    only(&names, prefix, weak).map_err(|e| format!("{} holds {e}", dir.display()))
}

/// Refuses `names` unless their processor parts, those that start with `prefix`, are some of `weak` alone.
pub(super) fn only(names: &[String], prefix: &str, weak: &[String]) -> Result<(), String> {
    let parts: Vec<&String> = names.iter().filter(|n| is_part(n, prefix)).collect();
    if parts.is_empty() {
        return Err(format!("none of {weak:?}"));
    }
    match parts
        .iter()
        .find(|p| !weak.iter().any(|w| w.eq_ignore_ascii_case(p)))
    {
        Some(p) => Err(format!("{p}, beyond {weak:?}")),
        None => Ok(()),
    }
}

/// `launch` with `extra` after its own words, as the gate worker reads its sound file.
pub(super) fn args(launch: &Launch, extra: &[String]) -> Launch {
    let mut out = launch.clone();
    out.args.extend_from_slice(extra);
    out
}

/// The spawner the speech host starts the gate worker with.
pub(super) struct GateWorker {
    /// The gate worker's spawner, capped in weak mode.
    worker: WinSpawner,
    /// The sound flag and the clip, after the voice job's words.
    extra: Vec<String>,
    /// The cap read back from the newest worker's job.
    cap: Arc<Mutex<Option<u32>>>,
    alive: Arc<Alive>,
    /// How long a worker let go may take to end.
    wait: Duration,
}

impl GateWorker {
    /// The gate worker `worker` starts, playing what `extra` names; a worker let go may take `wait` to end.
    pub(super) fn new(worker: WinSpawner, extra: Vec<String>, wait: Duration) -> Self {
        let (cap, alive) = (Arc::default(), Arc::default());
        Self {
            worker,
            extra,
            cap,
            alive,
            wait,
        }
    }

    /// The cap read back from the newest worker's job; `None` before a start or without one.
    pub(super) fn cap(&self) -> Arc<Mutex<Option<u32>>> {
        Arc::clone(&self.cap)
    }

    /// The count of its workers still running.
    pub(super) fn alive(&self) -> Arc<Alive> {
        Arc::clone(&self.alive)
    }
}

impl Spawner<VoiceFrames> for GateWorker {
    type Worker = Piped<Gated, VoiceFrames>;

    fn spawn(&self, launch: &Launch) -> Result<Self::Worker, SpawnError> {
        let (input, output, child) = self.worker.start(&args(launch, &self.extra))?;
        *lock(&self.cap) = child.cpu_cap();
        self.alive.up();
        let alive = Arc::clone(&self.alive);
        let wait = self.wait;
        Piped::new(input, output, Gated { child, alive, wait })
    }
}

/// A started gate worker; dropping it ends the worker, waits for it and counts it gone.
pub(super) struct Gated {
    child: WinChild,
    alive: Arc<Alive>,
    wait: Duration,
}

impl Child for Gated {
    fn end(&mut self) {
        self.child.end();
    }

    fn code(&mut self, within: Duration) -> Option<u32> {
        self.child.code(within)
    }

    fn peak_mb(&mut self) -> Option<u32> {
        self.child.peak_mb()
    }
}

impl Drop for Gated {
    fn drop(&mut self) {
        self.child.end();
        let _ = self.child.code(self.wait);
        self.alive.down();
    }
}

/// The cores the host counts for its threads: the real ones, or weak mode's. The bench runs on the processor only and
/// reads no device list, so it names no memory and no card's driver.
pub(super) struct Cores(pub(super) usize);

impl Hardware for Cores {
    fn cores(&self) -> usize {
        self.0.max(1)
    }

    fn memory(&self) -> Option<Memory> {
        None
    }

    fn driver(&self, _name: &str) -> Option<String> {
        None
    }
}

/// Locks `m`, taking the value even when a panicking thread poisoned it.
pub(super) fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(PoisonError::into_inner)
}
