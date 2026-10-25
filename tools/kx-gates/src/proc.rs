//! A process kx-gates watches, opened by id or given by the shell, to wait for its end or to measure its private
//! working set, private commit and CPU time.
#![cfg(windows)]

use serde::Serialize;
use windows::Win32::Foundation::{CloseHandle, FILETIME, HANDLE, WAIT_OBJECT_0};
use windows::Win32::System::ProcessStatus::{
    GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS, PROCESS_MEMORY_COUNTERS_EX2,
};
use windows::Win32::System::Threading::{
    GetExitCodeProcess, GetProcessId, GetProcessTimes, OpenProcess, PROCESS_ACCESS_RIGHTS,
    PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SET_QUOTA, PROCESS_SYNCHRONIZE, PROCESS_TERMINATE,
    PROCESS_VM_READ, WaitForSingleObject,
};

use crate::win::size32;

/// `FILETIME` ticks (100 ns) in one millisecond.
const TICKS_PER_MS: f64 = 10_000.0;
/// Bytes in one MB, as spec §10 counts memory.
pub const MB: f64 = 1_000_000.0;

/// A process's private memory, in MB.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Memory {
    /// The private working set: what Task Manager shows.
    pub working_mb: f64,
    /// The private commit: what the process holds in memory or paged out, which a trim never lowers.
    pub commit_mb: f64,
}

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

    /// Takes `handle`, an open process handle; it closes on drop.
    ///
    /// # Safety
    /// `handle` is an open process handle the caller owns and closes nowhere else.
    #[allow(unsafe_code, reason = "Takes a handle that `drop` closes.")]
    pub unsafe fn own(handle: HANDLE) -> Self {
        // SAFETY: the caller hands over an open process handle.
        let pid = unsafe { GetProcessId(handle) };
        Self { handle, pid }
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

    /// Its exit code once it has ended; none while it runs, or when Windows cannot say.
    #[allow(unsafe_code, reason = "Fills a local from our own handle.")]
    pub fn exit_code(&self) -> Option<u32> {
        if !self.ended() {
            return None;
        }
        let mut code = 0;
        // SAFETY: our live handle and a local the call fills.
        unsafe { GetExitCodeProcess(self.handle, &raw mut code) }.ok()?;
        Some(code)
    }

    /// Its private working set and private commit, read only; nothing here ever trims the working set.
    #[allow(unsafe_code, reason = "Fills a local of the size we pass.")]
    pub fn memory(&self) -> Result<Memory, String> {
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
        let mb = |bytes: usize| bytes as f64 / MB;
        Ok(Memory {
            working_mb: mb(c.PrivateWorkingSetSize),
            commit_mb: mb(c.PrivateUsage),
        })
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
    #[allow(unsafe_code, reason = "Closes the handle `open` or `own` took.")]
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

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::process::{Command, Stdio};

    use super::*;

    #[test]
    fn our_own_process_reads_its_working_set_commit_and_cpu_and_has_not_ended() {
        let me = Process::to_measure(std::process::id()).expect("our own process opens");
        assert!(!me.ended() && me.pid() == std::process::id());
        let m = me.memory().expect("memory");
        assert!(m.working_mb > 0.0 && m.commit_mb > 0.0, "{m:?}");
        assert!(me.cpu_ms().expect("CPU time") >= 0.0);
    }

    /// Every `.rs` file under `dir`.
    fn sources(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
        for entry in std::fs::read_dir(dir).expect("the folder reads") {
            let path = entry.expect("an entry").path();
            if path.is_dir() {
                sources(&path, out);
            } else if path.extension().is_some_and(|e| e == "rs") {
                out.push(path);
            }
        }
    }

    #[test]
    fn no_gate_trims_a_working_set() {
        // A trim drops the working set but not the commit, so it would only game G8 (ADR-0021). Split so this file
        // does not match itself.
        let trims = [
            ["Empty", "WorkingSet"].concat(),
            ["SetProcess", "WorkingSetSize"].concat(),
        ];
        let mut files = Vec::new();
        sources(
            &Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
            &mut files,
        );
        assert!(!files.is_empty());
        for file in files {
            let text = std::fs::read_to_string(&file).expect("reads");
            assert!(
                !trims.iter().any(|t| text.contains(t.as_str())),
                "{}",
                file.display()
            );
        }
    }

    #[test]
    #[allow(unsafe_code, reason = "Opens a handle that `own` then owns.")]
    fn an_owned_handle_names_its_process() {
        let rights = PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SYNCHRONIZE;
        // SAFETY: a plain call on our own id; the handle goes to `own`, which closes it.
        let h = unsafe { OpenProcess(rights, false, std::process::id()) }.expect("opens");
        // SAFETY: `h` is ours alone, and only `me` closes it.
        let me = unsafe { Process::own(h) };
        assert_eq!(me.pid(), std::process::id());
        assert!(!me.ended());
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
    fn an_ended_process_gives_its_exit_code_and_a_running_one_none() {
        let mut child = Command::new("cmd")
            .args(["/C", "exit", "7"])
            .spawn()
            .expect("starts");
        child.wait().expect("ends");
        assert_eq!(
            Process::to_wait(child.id()).expect("opens").exit_code(),
            Some(7)
        );
        assert_eq!(
            Process::to_wait(std::process::id())
                .expect("opens")
                .exit_code(),
            None
        );
    }
}
