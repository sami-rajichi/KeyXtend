//! `close`: the product gets Quit on its command window, as its own `--quit` sends it, and the gate waits for its
//! process to end; nothing is started. The frozen spike face only gets `WM_CLOSE`, which leaves its process running.
#![cfg(windows)]

use kx_platform::ui::{Command, Instance, UiError};
use kx_platform_windows::WindowsPlatform;
use serde::Serialize;
use serde_json::{Value, json};

use crate::apps::Ctx;
use crate::proc::Process;
use crate::product::{self, NAME};
use crate::{mouse, win, winfind};

/// Why the running copy was not asked to quit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unasked {
    /// Why, in words.
    pub why: String,
    /// Windows refused the Quit, as it does a plain kx-gates' Quit to a uiAccess copy.
    pub refused: bool,
}

impl From<String> for Unasked {
    fn from(why: String) -> Self {
        Self {
            why,
            refused: false,
        }
    }
}

impl From<UiError> for Unasked {
    fn from(e: UiError) -> Self {
        let refused = matches!(e, UiError::Refused(_));
        Self {
            why: why_not(e),
            refused,
        }
    }
}

/// Runs `close` on `name`: the product or the frozen face.
pub fn run(ctx: &Ctx, name: &str) -> Result<Value, String> {
    if name == NAME {
        let (exe, ended) = quit_running(ctx).map_err(|e| e.why)?;
        Ok(result(&exe, ended))
    } else {
        close_face(ctx, name)
    }
}

/// The result line of `close` on the product, run from `program`.
fn result(program: &str, ended: bool) -> Value {
    json!({ "gate": "close", "keyboard": NAME, "program": program, "pass": ended })
}

/// Why the running copy did not take Quit.
fn why_not(e: UiError) -> String {
    match e {
        UiError::NoWindow => {
            format!("{NAME} runs but has no command window yet; try again once it shows")
        }
        UiError::Refused(_) => {
            format!("{NAME} did not take Quit: {e}; run the installed uiAccess kx-gates")
        }
    }
}

/// Posts Quit to the copy that holds the session in `instance`.
fn ask_quit(instance: &dyn Instance) -> Result<(), Unasked> {
    instance.send(Command::Quit).map_err(Unasked::from)
}

/// Posts Quit to `instance` up to `tries` times, calling `wait` between them, as a copy just started may not listen
/// yet; it stops once `gone` says the copy ended. The tries it took; a refusal is final.
fn ask_quit_tries(
    instance: &dyn Instance,
    tries: u32,
    gone: impl Fn() -> bool,
    mut wait: impl FnMut(),
) -> Result<u32, String> {
    let mut n = 1;
    loop {
        if gone() {
            return Ok(n - 1);
        }
        match instance.send(Command::Quit) {
            Ok(()) => return Ok(n),
            Err(UiError::NoWindow) if n < tries => wait(),
            Err(e) => return Err(why_not(e)),
        }
        n += 1;
    }
}

/// True when `process` ended within `[timing] quit_wait_ms`.
pub fn ended_within(ctx: &Ctx, process: &Process) -> bool {
    let t = &ctx.cfg.timing;
    win::poll_until(t.quit_wait_ms, t.poll_ms, || process.ended().then_some(())).is_some()
}

/// Posts Quit to a copy a gate just started, trying `[g8] quit_tries` times as its command window may not exist yet;
/// whether `process` ended within `[timing] quit_wait_ms`.
pub fn quit_started(ctx: &Ctx, process: &Process) -> Result<bool, String> {
    let g = &ctx.cfg.g8;
    let platform = WindowsPlatform::new().map_err(|e| e.to_string())?;
    let gone = || process.ended();
    ask_quit_tries(&*platform.instance(), g.quit_tries, gone, || {
        win::sleep_ms(g.quit_try_ms);
    })?;
    Ok(ended_within(ctx, process))
}

/// How a copy a gate started went away.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Gone {
    /// It took Quit and ended in time.
    pub quit: bool,
    /// It has ended, by Quit or through its handle after.
    pub ended: bool,
}

impl Gone {
    /// True only when it quit by itself; a copy that had to be ended is a failure.
    pub fn ok(self) -> bool {
        self.quit && self.ended
    }
}

/// How a copy went once Quit reached it, when `asked`: `within` waits for its end, and when it did not end, `end` ends
/// it and `within` waits again.
pub(crate) fn finish(
    asked: bool,
    within: impl Fn() -> bool,
    end: impl FnOnce() -> Result<(), String>,
) -> Gone {
    if asked && within() {
        return Gone {
            quit: true,
            ended: true,
        };
    }
    Gone {
        quit: false,
        ended: end().is_ok() && within(),
    }
}

/// Quits a copy a gate started, held as `process`, as `quit_started` does; when it does not end in time, ends it
/// through its handle, so no copy is left running.
pub fn end_started(ctx: &Ctx, process: &Process) -> Gone {
    let g = &ctx.cfg.g8;
    let platform = WindowsPlatform::new().map_err(|e| e.to_string());
    let asked = platform.and_then(|p| {
        let wait = || win::sleep_ms(g.quit_try_ms);
        ask_quit_tries(&*p.instance(), g.quit_tries, || process.ended(), wait)
    });
    finish(
        asked.is_ok(),
        || ended_within(ctx, process),
        || process.end(),
    )
}

/// Posts Quit to the running product, which starts nothing; its program, and whether it ended within
/// `[timing] quit_wait_ms`.
pub fn quit_running(ctx: &Ctx) -> Result<(String, bool), Unasked> {
    let not_running = || format!("{NAME} is not running");
    let (pid, exe) = product::running().ok_or_else(not_running)?;
    // Opened before the quit, so the id cannot name another process by the time we look.
    let process = Process::to_wait(pid)?;
    let platform = WindowsPlatform::new().map_err(|e| e.to_string())?;
    ask_quit(&*platform.instance())?;
    Ok((exe, ended_within(ctx, &process)))
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
mod tests;
