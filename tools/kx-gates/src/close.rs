//! `close`: the product gets Quit on its command window, as `--quit` sends it, and the gate waits for its end, then
//! restores a settings backup a stopped gate left; nothing is started. The frozen spike face only gets `WM_CLOSE`,
//! which leaves its process running.
#![cfg(windows)]

use std::path::Path;

use kx_platform::ui::{Command, Instance, UiError};
use kx_platform_windows::WindowsPlatform;
use serde_json::{Value, json};

use crate::apps::Ctx;
use crate::proc::Process;
use crate::product::{self, NAME};
use crate::{mouse, owncopy, win, winfind};

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
        close_product(ctx)
    } else {
        close_face(ctx, name)
    }
}

/// Posts Quit to the running product, if any, then restores the owner's files from a backup a stopped gate left,
/// once no copy runs (`owncopy::recover`).
fn close_product(ctx: &Ctx) -> Result<Value, String> {
    let quit = || product::running().map(|c| quit_running(ctx, c)).transpose();
    closed(
        quit,
        |ended| owncopy::recover(ctx, ended),
        &owncopy::side(ctx)?,
    )
}

/// `close` on the product: `quit` asks the running copy, if any, to quit, then `recover` restores the backup at `side`,
/// told whether that copy ended, as one still exiting may yet save settings.
fn closed(
    quit: impl FnOnce() -> Result<Option<(String, bool)>, Unasked>,
    recover: impl FnOnce(bool) -> Result<bool, String>,
    side: &Path,
) -> Result<Value, String> {
    let quit = quit().map_err(|e| e.why)?;
    let back = recover(ended(quit.as_ref()));
    if quit.is_none() && back == Ok(false) {
        let side = side.display();
        return Err(format!(
            "{NAME} is not running, and no backup waits at {side}"
        ));
    }
    Ok(result(quit.as_ref(), &back))
}

/// True unless the copy `quit` asked did not end; true when none ran.
fn ended(quit: Option<&(String, bool)>) -> bool {
    quit.is_none_or(|(_, e)| *e)
}

/// The result line of `close` on the product: the copy it asked to quit, its program and whether it ended, if one
/// ran, and whether the owner's files came back from a backup a stopped gate left.
fn result(quit: Option<&(String, bool)>, back: &Result<bool, String>) -> Value {
    json!({
        "gate": "close", "keyboard": NAME, "program": quit.map(|(p, _)| p), "settings_back": back.as_ref().ok(),
        "settings_error": back.as_ref().err(), "pass": ended(quit) && back.is_ok(),
    })
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

/// What the Quit tries came to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Asked {
    /// Quit reached the copy at this try.
    Sent(u32),
    /// The copy had ended before Quit reached it, after this many failed tries.
    Gone(u32),
}

/// Posts Quit to `instance` up to `tries` times, calling `wait` between them, as a copy just started may not listen
/// yet; it stops once `gone` says the copy ended. A refusal is final.
pub(crate) fn ask_quit_tries(
    instance: &dyn Instance,
    tries: u32,
    gone: impl Fn() -> bool,
    mut wait: impl FnMut(),
) -> Result<Asked, String> {
    let mut n = 1;
    loop {
        if gone() {
            return Ok(Asked::Gone(n - 1));
        }
        match instance.send(Command::Quit) {
            Ok(()) => return Ok(Asked::Sent(n)),
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

/// Posts Quit to `process`, a copy a gate just started, up to `[g8] quit_tries` times `quit_try_ms` apart, as its
/// command window may not exist yet.
pub(crate) fn ask_started(ctx: &Ctx, process: &Process) -> Result<Asked, String> {
    let g = &ctx.cfg.g8;
    let platform = WindowsPlatform::new().map_err(|e| e.to_string())?;
    let wait = || win::sleep_ms(g.quit_try_ms);
    ask_quit_tries(
        &*platform.instance(),
        g.quit_tries,
        || process.ended(),
        wait,
    )
}

/// Posts Quit to a copy a gate just started (`ask_started`); whether `process` ended within `[timing] quit_wait_ms`.
pub fn quit_started(ctx: &Ctx, process: &Process) -> Result<bool, String> {
    ask_started(ctx, process)?;
    Ok(ended_within(ctx, process))
}

/// Posts Quit to `running`, the copy `product::running` found, and starts nothing; its program, and whether it ended
/// within `[timing] quit_wait_ms`. The caller's one look serves, so a copy gone since then is not looked for again.
pub fn quit_running(ctx: &Ctx, running: (u32, String)) -> Result<(String, bool), Unasked> {
    let (pid, exe) = running;
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
