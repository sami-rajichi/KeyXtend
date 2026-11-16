//! The modules a worker has mapped, read from a system snapshot as a debugger reads them: which of our own children run
//! a program, each one's module paths, and a sampler that lists them while a check runs. Paths are only compared, never
//! printed, as they may hold the user's name.
#![cfg(windows)]

use std::collections::BTreeSet;
use std::path::{Component, Path, PathBuf, Prefix};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use windows::Win32::Foundation::{CloseHandle, ERROR_BAD_LENGTH};
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, MODULEENTRY32W, Module32FirstW, Module32NextW, TH32CS_SNAPMODULE,
};

use crate::apps::same_name;
use crate::g19chips::procs::{self, Proc};
use crate::win::{poll_until, size32};

/// Snapshots tried while Windows asks to try again, as it does while the process maps a module.
const TRIES: usize = 3;

/// The ids of `all` that are children of process `me` running program `exe`.
pub fn kids(all: &[Proc], me: u32, exe: &str) -> Vec<u32> {
    let ours = |p: &&Proc| p.parent == me && same_name(&p.exe, exe);
    all.iter().filter(ours).map(|p| p.pid).collect()
}

/// Waits up to `wait_ms` until none of our children runs `exe`, as a dropped host says goodbye without waiting for the
/// end; true once none runs.
pub fn ended(exe: &str, wait_ms: u64, poll_ms: u64) -> bool {
    let me = std::process::id();
    let none = || kids(&procs::now(), me, exe).is_empty().then_some(());
    poll_until(wait_ms, poll_ms, none).is_some()
}

/// A path's parts in lower case, a drive named by its letter alone, so `\\?\D:\a` and `d:\A` agree.
fn parts(path: &Path) -> Vec<String> {
    let part = |c: Component| match c {
        Component::Prefix(p) => match p.kind() {
            Prefix::Disk(l) | Prefix::VerbatimDisk(l) => format!("{}:", char::from(l)),
            _ => p.as_os_str().to_string_lossy().into_owned(),
        },
        other => other.as_os_str().to_string_lossy().into_owned(),
    };
    path.components().map(|c| part(c).to_lowercase()).collect()
}

/// True when `path` lies inside folder `dir`, by whole parts in any letter case; the folder itself is not inside.
pub fn inside(path: &Path, dir: &Path) -> bool {
    let (path, dir) = (parts(path), parts(dir));
    path.len() > dir.len() && path.starts_with(&dir)
}

/// What the sampler saw: snapshots read and refused, and every module path once.
#[derive(Debug, Default)]
pub struct Seen {
    /// Snapshots read.
    pub snaps: u32,
    /// Snapshots Windows refused.
    pub failed: u32,
    paths: BTreeSet<PathBuf>,
}

impl Seen {
    /// Adds one snapshot's modules, or counts its refusal.
    pub fn add(&mut self, read: Result<Vec<PathBuf>, String>) {
        match read {
            Ok(list) => {
                self.snaps += 1;
                self.paths.extend(list);
            }
            Err(_) => self.failed += 1,
        }
    }

    /// How many modules seen lie inside folder `dir`.
    pub fn under(&self, dir: &Path) -> usize {
        self.named_under(dir, |_| true)
    }

    /// How many modules seen lie inside folder `dir` with a file name `keep` takes.
    pub fn named_under(&self, dir: &Path, keep: impl Fn(&str) -> bool) -> usize {
        let named = |p: &&PathBuf| p.file_name().is_some_and(|n| keep(&n.to_string_lossy()));
        self.paths
            .iter()
            .filter(|p| inside(p, dir))
            .filter(named)
            .count()
    }

    /// True when module `file` was seen, in any letter case.
    pub fn has(&self, file: &Path) -> bool {
        let want = parts(file);
        self.paths.iter().any(|p| parts(p) == want)
    }
}

/// The module paths of process `pid`; an error when Windows refuses its snapshot.
#[allow(unsafe_code, reason = "Reads another process's module snapshot.")]
pub fn modules(pid: u32) -> Result<Vec<PathBuf>, String> {
    let mut tries = 0;
    let snap = loop {
        // SAFETY: a plain call; the snapshot it gives is walked and closed below.
        let made = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPMODULE, pid) };
        match made {
            Ok(snap) => break snap,
            Err(e) if e.code() == ERROR_BAD_LENGTH.to_hresult() && tries + 1 < TRIES => tries += 1,
            Err(e) => return Err(format!("the worker's module list: {e}")),
        }
    };
    let mut found = Vec::new();
    let mut e = MODULEENTRY32W {
        dwSize: size32::<MODULEENTRY32W>(),
        ..Default::default()
    };
    // SAFETY: our live snapshot and a local entry whose size it states; the snapshot closes once walked.
    unsafe {
        let mut more = Module32FirstW(snap, &raw mut e).is_ok();
        while more {
            let end = e
                .szExePath
                .iter()
                .position(|&c| c == 0)
                .unwrap_or(e.szExePath.len());
            found.push(PathBuf::from(String::from_utf16_lossy(&e.szExePath[..end])));
            more = Module32NextW(snap, &raw mut e).is_ok();
        }
        let _ = CloseHandle(snap);
    }
    Ok(found)
}

/// Lists the modules of our children running one program, every poll, until it finishes.
pub struct Sampler {
    stop: Arc<AtomicBool>,
    seen: Arc<Mutex<Seen>>,
    thread: Option<JoinHandle<()>>,
}

impl Sampler {
    /// Starts listing the modules of our children running `exe` every `poll`.
    pub fn start(exe: &'static str, poll: Duration) -> Self {
        let (stop, seen) = (
            Arc::new(AtomicBool::new(false)),
            Arc::new(Mutex::new(Seen::default())),
        );
        let (halt, into) = (Arc::clone(&stop), Arc::clone(&seen));
        let me = std::process::id();
        let thread = thread::spawn(move || {
            while !halt.load(Ordering::Acquire) {
                for pid in kids(&procs::now(), me, exe) {
                    lock(&into).add(modules(pid));
                }
                thread::sleep(poll);
            }
        });
        Self {
            stop,
            seen,
            thread: Some(thread),
        }
    }

    /// Stops listing: what it saw.
    pub fn finish(mut self) -> Seen {
        self.halt();
        std::mem::take(&mut *lock(&self.seen))
    }

    /// Stops the thread and waits for it.
    fn halt(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}

impl Drop for Sampler {
    fn drop(&mut self) {
        self.halt();
    }
}

/// Locks `m`, taking the value even when a panicking thread poisoned it.
fn lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(PoisonError::into_inner)
}

#[cfg(test)]
mod tests;
