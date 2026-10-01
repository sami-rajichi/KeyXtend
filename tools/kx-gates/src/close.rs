//! `close`: the product quits through its own `--quit`, and the gate waits for its process to end. The frozen spike
//! face only gets `WM_CLOSE`, as before, which hides its window but leaves its process running.
#![cfg(windows)]

use std::path::Path;

use kx_platform::ui::Command;
use kx_platform_windows::instance;
use serde_json::{Value, json};
use windows::Win32::Foundation::{CloseHandle, HANDLE, WAIT_OBJECT_0};
use windows::Win32::System::Threading::{
    OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SYNCHRONIZE, WaitForSingleObject,
};

use crate::apps::Ctx;
use crate::{admin, mouse, product, win, winfind};

/// Runs `close` on `name`: the product or the frozen face.
pub fn run(ctx: &Ctx, name: &str) -> Result<Value, String> {
    if name == product::NAME {
        quit(ctx)
    } else {
        close_face(ctx, name)
    }
}

/// The arguments that ask the running product to quit.
fn quit_args() -> Vec<String> {
    vec![instance::flag(Command::Quit)]
}

/// Runs the product's own program with `--quit`, which returns at once, then waits for the running copy to end.
fn quit(ctx: &Ctx) -> Result<Value, String> {
    let not_running = || format!("{} is not running", product::NAME);
    let (pid, exe) = product::running().ok_or_else(not_running)?;
    // Opened before the quit, so the id cannot name another process by the time we look.
    let process = Process::open(pid)?;
    admin::run_hidden(Path::new(&exe), &quit_args())?;
    let t = &ctx.cfg.timing;
    let ended = win::poll_until(t.quit_wait_ms, t.poll_ms, || process.ended().then_some(()));
    Ok(json!({ "gate": "close", "name": product::NAME, "program": exe, "pass": ended.is_some() }))
}

/// Closes the frozen face `name` and waits until its window is gone; needs the uiAccess kx-gates.
fn close_face(ctx: &Ctx, name: &str) -> Result<Value, String> {
    let hwnd = mouse::find_face(ctx.spike, name)?;
    win::close(hwnd)?;
    let t = &ctx.cfg.timing;
    let closed = winfind::wait_gone(hwnd, t.close_wait_ms, t.poll_ms);
    Ok(json!({ "gate": "close", "face": name, "pass": closed }))
}

/// A process handle that tells when the process has ended; it closes on drop.
struct Process(HANDLE);

impl Process {
    /// Opens process `pid` to wait for it.
    #[allow(unsafe_code, reason = "Opens a handle that `drop` closes.")]
    fn open(pid: u32) -> Result<Self, String> {
        let access = PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SYNCHRONIZE;
        // SAFETY: a plain call; the handle it gives is closed in `drop`.
        let opened = unsafe { OpenProcess(access, false, pid) };
        opened
            .map(Self)
            .map_err(|e| format!("OpenProcess {pid}: {e}"))
    }

    /// True once the process has ended.
    #[allow(unsafe_code, reason = "A wait of zero ms on our own handle.")]
    fn ended(&self) -> bool {
        // SAFETY: our live handle; a wait of zero ms never blocks.
        unsafe { WaitForSingleObject(self.0, 0) == WAIT_OBJECT_0 }
    }
}

impl Drop for Process {
    #[allow(unsafe_code, reason = "Closes the handle `open` made.")]
    fn drop(&mut self) {
        // SAFETY: the handle is ours and closed only here.
        let _ = unsafe { CloseHandle(self.0) };
    }
}

#[cfg(test)]
mod tests {
    use std::process::{Command, Stdio};

    use super::*;

    #[test]
    fn the_product_is_asked_to_quit_with_its_own_flag() {
        assert_eq!(quit_args(), ["--quit"]);
    }

    #[test]
    fn a_process_that_ended_reads_as_ended_and_a_running_one_does_not() {
        let me = Process::open(std::process::id()).expect("our own process opens");
        assert!(!me.ended());
        let exe = std::env::current_exe().expect("the test program");
        let mut child = Command::new(exe)
            .arg("--list")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("starts");
        child.wait().expect("ends");
        // `child` keeps its handle until it drops, so its id still names it.
        let done = Process::open(child.id()).expect("opens");
        assert!(done.ended());
    }
}
