//! `close`: the product gets Quit on its command window, as `--quit` sends it, and the gate waits for its end, then
//! restores a settings backup a stopped gate left; nothing is started. The frozen spike face only gets `WM_CLOSE`,
//! which leaves its process running.
#![cfg(windows)]

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
    let quit = match product::running() {
        Some(_) => Some(quit_running(ctx).map_err(|e| e.why)?),
        None => None,
    };
    let back = owncopy::recover(ctx);
    if quit.is_none() && back == Ok(false) {
        return Err(format!("{NAME} is not running"));
    }
    Ok(result(quit.as_ref(), &back))
}

/// The result line of `close` on the product: the copy it asked to quit, its program and whether it ended, if one
/// ran, and whether the owner's files came back from a backup a stopped gate left.
fn result(quit: Option<&(String, bool)>, back: &Result<bool, String>) -> Value {
    let (program, ended) = quit.map_or((None, true), |(p, e)| (Some(p), *e));
    json!({
        "gate": "close", "keyboard": NAME, "program": program, "settings_back": back.as_ref().ok(),
        "settings_error": back.as_ref().err(), "pass": ended && back.is_ok(),
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
