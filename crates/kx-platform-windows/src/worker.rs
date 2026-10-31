//! The media worker's start on Windows (ADR-0004, ADR-0034 decision 5): `keyxtend-worker.exe` beside us, from our own
//! token with uiAccess cleared and Medium integrity, inheriting only its two pipe ends, in a job that caps its memory
//! and ends it with us. The neutral host in `kx_platform::pictures` drives it.

mod job;
mod line;
mod start;
mod token;

use std::io::{PipeReader, PipeWriter};
use std::os::windows::io::{AsRawHandle, OwnedHandle};
use std::path::PathBuf;
use std::time::Duration;

use kx_platform::pictures::{Child, Launch, Piped, SpawnError, Spawner};
use windows::Win32::Foundation::{HANDLE, WAIT_OBJECT_0};
use windows::Win32::System::Threading::{GetExitCodeProcess, WaitForSingleObject};

use crate::PlatformError;
use job::Job;

/// The worker program's file, beside ours.
pub const WORKER_EXE: &str = "keyxtend-worker.exe";

/// Starts the worker program with reduced rights. It has no `Debug`, as its path may hold the user's name.
#[derive(Clone)]
pub struct WinSpawner {
    exe: PathBuf,
}

impl WinSpawner {
    /// The worker beside the running program.
    ///
    /// # Errors
    /// `ExePath` or `NoExeDir` when the program's own path is unknown.
    pub fn beside_us() -> Result<Self, PlatformError> {
        let exe = std::env::current_exe().map_err(PlatformError::ExePath)?;
        let dir = exe.parent().ok_or(PlatformError::NoExeDir)?;
        Ok(Self::at(dir.join(WORKER_EXE)))
    }

    /// The worker at `exe`; a relative path is refused at each start, so no search path is ever used.
    #[must_use]
    pub const fn at(exe: PathBuf) -> Self {
        Self { exe }
    }

    /// Starts the worker and hands back its stdin, its stdout and its process.
    fn start(&self, launch: &Launch) -> Result<(PipeWriter, PipeReader, WinChild), SpawnError> {
        if !self.exe.is_absolute() || !self.exe.is_file() {
            return Err(SpawnError::Missing);
        }
        start::start(&self.exe, launch)
    }
}

impl Spawner for WinSpawner {
    type Worker = Piped<WinChild>;

    fn spawn(&self, launch: &Launch) -> Result<Piped<WinChild>, SpawnError> {
        let (input, output, child) = self.start(launch)?;
        Piped::new(input, output, child)
    }
}

/// The worker's process and its job; dropping it closes the job, which ends the process.
pub struct WinChild {
    process: OwnedHandle,
    job: Job,
}

impl Child for WinChild {
    fn end(&mut self) {
        self.job.end();
    }

    #[allow(unsafe_code, reason = "Waits on and reads our own process handle.")]
    fn code(&mut self, within: Duration) -> Option<u32> {
        let ms = u32::try_from(within.as_millis()).unwrap_or(u32::MAX - 1);
        // SAFETY: our live process handle; the wait ends by `ms` at the latest.
        let ended = unsafe { WaitForSingleObject(raw(&self.process), ms) } == WAIT_OBJECT_0;
        let mut code = 0;
        // SAFETY: our live process handle and a local to write to.
        let read =
            ended && unsafe { GetExitCodeProcess(raw(&self.process), &raw mut code) }.is_ok();
        read.then_some(code)
    }
}

/// The raw handle of `owned`, which stays open while it is used.
fn raw(owned: &impl AsRawHandle) -> HANDLE {
    HANDLE(owned.as_raw_handle())
}

/// The size of `T` as Windows takes it; every struct passed is far below the cap, and Windows refuses a wrong size.
fn size32<T>() -> u32 {
    u32::try_from(size_of::<T>()).unwrap_or(u32::MAX)
}

/// A refusal of `step` that keeps Windows' error code.
fn refused(step: &'static str) -> impl Fn(windows::core::Error) -> SpawnError {
    move |e| SpawnError::Refused {
        step,
        code: e.code().0,
    }
}

#[cfg(test)]
mod tests;
