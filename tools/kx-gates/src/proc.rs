//! A process kx-gates watches: opened by id, to wait for its end or to measure its private memory and CPU time; and
//! the size of a program's folder.
#![cfg(windows)]

use std::path::Path;

use windows::Win32::Foundation::{CloseHandle, FILETIME, HANDLE, WAIT_OBJECT_0};
use windows::Win32::System::ProcessStatus::{
    GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS, PROCESS_MEMORY_COUNTERS_EX2,
};
use windows::Win32::System::Threading::{
    GetProcessTimes, OpenProcess, PROCESS_ACCESS_RIGHTS, PROCESS_QUERY_LIMITED_INFORMATION,
    PROCESS_SET_QUOTA, PROCESS_SYNCHRONIZE, PROCESS_TERMINATE, PROCESS_VM_READ,
    WaitForSingleObject,
};

use crate::win::size32;

/// `FILETIME` ticks (100 ns) in one millisecond.
const TICKS_PER_MS: f64 = 10_000.0;
/// Bytes in one MB, as spec §10 counts memory and size.
const MB: f64 = 1_000_000.0;

/// A process handle and the process's id; the handle closes on drop.
pub struct Process {
    handle: HANDLE,
    pid: u32,
}

impl Process {
    /// Opens process `pid` only to wait for its end, which a less trusted caller may also do.
    pub fn to_wait(pid: u32) -> Result<Self, String> {
        Self::open(pid, PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SYNCHRONIZE)
    }

    /// Opens process `pid` to wait for it, read its memory and CPU time, and put it in a job.
    pub fn to_measure(pid: u32) -> Result<Self, String> {
        let job = PROCESS_SET_QUOTA | PROCESS_TERMINATE;
        let read = PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_VM_READ | PROCESS_SYNCHRONIZE;
        Self::open(pid, read | job)
    }

    #[allow(unsafe_code, reason = "Opens a handle that `drop` closes.")]
    fn open(pid: u32, rights: PROCESS_ACCESS_RIGHTS) -> Result<Self, String> {
        // SAFETY: a plain call; the handle it gives is closed in `drop`.
        let opened = unsafe { OpenProcess(rights, false, pid) };
        let handle = opened.map_err(|e| format!("OpenProcess {pid}: {e}"))?;
        Ok(Self { handle, pid })
    }

    /// The handle, for a job; it stays this value's.
    pub fn handle(&self) -> HANDLE {
        self.handle
    }

    /// The process's id.
    pub fn pid(&self) -> u32 {
        self.pid
    }

    /// True once the process has ended.
    #[allow(unsafe_code, reason = "A wait of zero ms on our own handle.")]
    pub fn ended(&self) -> bool {
        // SAFETY: our live handle; a wait of zero ms never blocks.
        unsafe { WaitForSingleObject(self.handle, 0) == WAIT_OBJECT_0 }
    }

    /// Its private working set, the memory Task Manager shows, in MB.
    #[allow(unsafe_code, reason = "Fills a local of the size we pass.")]
    pub fn private_mb(&self) -> Result<f64, String> {
        let mut c = PROCESS_MEMORY_COUNTERS_EX2 {
            cb: size32::<PROCESS_MEMORY_COUNTERS_EX2>(),
            ..Default::default()
        };
        let at = (&raw mut c).cast::<PROCESS_MEMORY_COUNTERS>();
        // SAFETY: `at` points to a live struct of the size passed as `cb`.
        unsafe { GetProcessMemoryInfo(self.handle, at, c.cb) }
            .map_err(|e| format!("memory: {e}"))?;
        #[allow(
            clippy::cast_precision_loss,
            reason = "Sizes are far below 2^53 bytes."
        )]
        Ok(c.PrivateWorkingSetSize as f64 / MB)
    }

    /// The CPU time it has used so far, in the kernel and in user mode, in ms.
    #[allow(unsafe_code, reason = "Fills four locals.")]
    pub fn cpu_ms(&self) -> Result<f64, String> {
        let [mut made, mut ended, mut kernel, mut user] = [FILETIME::default(); 4];
        // SAFETY: four live locals, as the call takes.
        unsafe {
            GetProcessTimes(
                self.handle,
                &raw mut made,
                &raw mut ended,
                &raw mut kernel,
                &raw mut user,
            )
        }
        .map_err(|e| format!("CPU time: {e}"))?;
        Ok((ticks(kernel) + ticks(user)) / TICKS_PER_MS)
    }
}

impl Drop for Process {
    #[allow(unsafe_code, reason = "Closes the handle `open` made.")]
    fn drop(&mut self) {
        // SAFETY: the handle is ours and closed only here.
        let _ = unsafe { CloseHandle(self.handle) };
    }
}

/// A `FILETIME` span as 100 ns ticks.
#[allow(
    clippy::cast_precision_loss,
    reason = "CPU times are far below 2^53 ticks."
)]
fn ticks(t: FILETIME) -> f64 {
    ((u64::from(t.dwHighDateTime) << u32::BITS) | u64::from(t.dwLowDateTime)) as f64
}

/// Every file's size under `dir`, in MB; links are not followed.
pub fn folder_mb(dir: &Path) -> Result<f64, String> {
    #[allow(
        clippy::cast_precision_loss,
        reason = "Sizes are far below 2^53 bytes."
    )]
    Ok(bytes(dir)? as f64 / MB)
}

/// Every file's size under `dir`, in bytes.
fn bytes(dir: &Path) -> Result<u64, String> {
    let err = |e: std::io::Error| format!("{}: {e}", dir.display());
    let mut sum = 0;
    for entry in std::fs::read_dir(dir).map_err(err)? {
        let entry = entry.map_err(err)?;
        let meta = std::fs::symlink_metadata(entry.path()).map_err(err)?;
        sum += if meta.is_dir() {
            bytes(&entry.path())?
        } else if meta.is_file() {
            meta.len()
        } else {
            0
        };
    }
    Ok(sum)
}

#[cfg(test)]
mod tests {
    use std::process::{Command, Stdio};

    use kx_test_support::tempdir::TempDir;

    use super::*;

    #[test]
    fn our_own_process_reads_memory_and_cpu_and_has_not_ended() {
        let me = Process::to_measure(std::process::id()).expect("our own process opens");
        assert!(!me.ended() && me.pid() == std::process::id());
        assert!(me.private_mb().expect("memory") > 0.0);
        assert!(me.cpu_ms().expect("CPU time") >= 0.0);
    }

    #[test]
    fn a_process_that_ended_reads_as_ended() {
        let exe = std::env::current_exe().expect("the test program");
        let mut child = Command::new(exe)
            .arg("--list")
            .stdout(Stdio::null())
            .spawn()
            .expect("starts");
        child.wait().expect("ends");
        // `child` keeps its handle until it drops, so its id still names it.
        assert!(Process::to_wait(child.id()).expect("opens").ended());
    }

    #[test]
    fn a_folder_counts_every_file_below_it_in_mb() {
        let dir = TempDir::new("gates-size").expect("temp folder");
        let half = vec![0u8; 500_000];
        std::fs::create_dir(dir.path().join("sub")).expect("sub folder");
        std::fs::write(dir.path().join("a"), &half).expect("file");
        std::fs::write(dir.path().join("sub").join("b"), &half).expect("file");
        assert!((folder_mb(dir.path()).expect("sized") - 1.0).abs() < 1e-9);
        assert!(folder_mb(&dir.path().join("none")).is_err());
    }
}
