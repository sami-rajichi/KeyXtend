//! The P12 live checks: the freeze and a GDI grab leave out a shielded window; the worker's `record` job, under its job
//! and token, records a part with sound into a playable MP4 and re-times it, here and in weak mode, and the recorder
//! host keeps, copies and cancels through it; the `pick` job's dialogs open above our topmost window and answer checked
//! paths; and the media job's largest encode tells its peak memory. They show windows, read and record the screen,
//! the microphone and the PC's sound and open dialogs, so they run only when the build saw `KX_LIVE_SNIP=1`.
#![allow(unsafe_code, reason = "The probes read their own worker's process id.")]

use super::*;
use serde::Deserialize;
use std::fs;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Mutex, MutexGuard, PoisonError};
use windows::Win32::System::Threading::GetProcessId;

mod encode;
mod freeze;
mod host;
mod mp4;
mod pick;
mod record;
mod screen;
mod session;
mod shield;

/// The gate tool's config, whose `[weak]` section is the weak PC every weak-mode run copies.
const GATES: &str = include_str!("../../../../../tools/kx-gates/kx-gates.toml");
/// The start of each probe's own temp folder's name.
const TEMP_TAG: &str = "kx-probe-snip";

/// Keeps the probes apart: each shows windows on the one screen.
static LIVE: Mutex<()> = Mutex::new(());

/// Holds the other probes off until dropped.
fn hold() -> MutexGuard<'static, ()> {
    LIVE.lock().unwrap_or_else(PoisonError::into_inner)
}

/// The gate tool's config, read for its weak PC alone.
#[derive(Deserialize)]
struct Gates {
    weak: Weak,
}

/// The weak PC's cores and each core's speed in percent of one here.
#[derive(Deserialize)]
struct Weak {
    cores: usize,
    core_pct: u8,
}

/// The weak PC's processor, as the gate tool's weak mode caps a worker.
fn weak() -> CpuCap {
    let gates: Gates = toml::from_str(GATES).unwrap();
    CpuCap {
        cores: gates.weak.cores,
        pct: gates.weak.core_pct,
    }
}

/// A folder of the probe's own under the temp folder, deleted with all it holds when dropped, a failure too.
struct Temp(PathBuf);

impl Temp {
    fn new() -> Self {
        static NEXT: AtomicU32 = AtomicU32::new(0);
        let n = NEXT.fetch_add(1, Ordering::Relaxed);
        let dir = env::temp_dir().join(format!("{TEMP_TAG}-{}-{n}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        Self(dir)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// How a probe's worker ended: its exit code and the most memory it took.
struct Ran {
    code: Option<u32>,
    peak_mb: Option<u32>,
}

/// A probe's worker, started as the host starts one; dropping it ends the worker and waits for it.
struct Probing {
    input: Option<PipeWriter>,
    output: Option<PipeReader>,
    child: WinChild,
}

impl Probing {
    /// Starts the worker as `launch` says, under its job's own limits.
    fn launch(spawner: &WinSpawner, launch: &Launch) -> Self {
        let (input, output, child) = spawner.start(launch).unwrap();
        Self {
            input: Some(input),
            output: Some(output),
            child,
        }
    }

    /// The worker's stdin and stdout, taken once, for a job that talks in frames.
    fn pipes(&mut self) -> (PipeWriter, PipeReader) {
        (self.input.take().unwrap(), self.output.take().unwrap())
    }

    /// The worker's process id.
    fn pid(&self) -> u32 {
        // SAFETY: our live process handle.
        unsafe { GetProcessId(raw(&self.child.process)) }
    }

    /// Waits at most `within` for the worker's end.
    fn finish(mut self, within: Duration) -> Ran {
        Ran {
            code: self.child.code(within),
            peak_mb: self.child.peak_mb(),
        }
    }
}

impl Drop for Probing {
    /// Ends the worker and waits at most `WAIT` for it, so its files are closed before the probe's temp folder goes.
    fn drop(&mut self) {
        self.child.end();
        let _ = self.child.code(WAIT);
    }
}

#[test]
fn the_weak_pc_comes_from_the_gate_tools_config() {
    let cap = weak();
    assert!(
        cap.cores > 0 && cap.pct > 0 && cap.pct <= CpuCap::FULL_PCT,
        "{cap:?}"
    );
}
