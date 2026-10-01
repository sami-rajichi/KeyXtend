//! `close`: the product quits through its own `--quit`, and the gate waits for its process to end. The frozen spike
//! face only gets `WM_CLOSE`, as before, which hides its window but leaves its process running.
#![cfg(windows)]

use std::path::Path;

use kx_platform::ui::Command;
use kx_platform_windows::instance;
use serde_json::{Value, json};

use crate::apps::Ctx;
use crate::proc::Process;
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

/// Runs the product's own program with `--quit`, then waits for the running copy to end.
fn quit(ctx: &Ctx) -> Result<Value, String> {
    let (exe, ended) = quit_running(ctx)?;
    Ok(json!({ "gate": "close", "name": product::NAME, "program": exe, "pass": ended }))
}

/// Asks the running product to quit with its own `--quit`, which returns at once; its program, and whether it ended
/// within `[timing] quit_wait_ms`.
pub fn quit_running(ctx: &Ctx) -> Result<(String, bool), String> {
    let not_running = || format!("{} is not running", product::NAME);
    let (pid, exe) = product::running().ok_or_else(not_running)?;
    // Opened before the quit, so the id cannot name another process by the time we look.
    let process = Process::to_wait(pid)?;
    admin::run_hidden(Path::new(&exe), &quit_args())?;
    let t = &ctx.cfg.timing;
    let ended = win::poll_until(t.quit_wait_ms, t.poll_ms, || process.ended().then_some(()));
    Ok((exe, ended.is_some()))
}

/// Closes the frozen face `name` and waits until its window is gone; needs the uiAccess kx-gates.
fn close_face(ctx: &Ctx, name: &str) -> Result<Value, String> {
    let hwnd = mouse::find_face(ctx.spike, name)?;
    win::close(hwnd)?;
    let t = &ctx.cfg.timing;
    let closed = winfind::wait_gone(hwnd, t.close_wait_ms, t.poll_ms);
    Ok(json!({ "gate": "close", "face": name, "pass": closed }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_product_is_asked_to_quit_with_its_own_flag() {
        assert_eq!(quit_args(), ["--quit"]);
    }
}
