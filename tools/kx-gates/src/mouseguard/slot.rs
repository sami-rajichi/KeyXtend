//! This process's guard, which `keys`, `admin` and the presses reach without a run context: one session at most, held
//! from the gate's start until it ends.

use std::sync::{Mutex, MutexGuard, PoisonError};

use kx_module_api::Clock;
use kx_platform_windows::QpcClock;
use serde_json::{Value, json};

use super::hook::WinHook;
use super::run::{Hook, Outcome, Session};
use super::{Mouse, MouseGuard};

/// The result line's field that says how the guard ended.
const FIELD: &str = "mouse_guard";
/// The result line's verdict, which a stopped run fails.
const PASS: &str = "pass";

/// A session as this process's slot holds it, whatever its hook and clock.
pub trait Held: Send {
    /// See `Session::refusal`.
    fn refusal(&mut self) -> Result<(), String>;
    /// See `Session::own`.
    fn own(&self) -> u64;
    /// See `Session::sent`.
    fn sent(&mut self, base: u64);
    /// See `Session::pause`.
    fn pause(&mut self);
    /// See `Session::resume`.
    fn resume(&mut self);
    /// See `Session::release`.
    fn release(&mut self);
    /// See `Session::finish`.
    fn finish(self: Box<Self>, ok: bool) -> Outcome;
}

impl<H, C> Held for Session<H, C>
where
    H: Hook + Send,
    H::Live: Send,
    C: Clock,
{
    fn refusal(&mut self) -> Result<(), String> {
        Session::refusal(self)
    }
    fn own(&self) -> u64 {
        Session::own(self)
    }
    fn sent(&mut self, base: u64) {
        Session::sent(self, base);
    }
    fn pause(&mut self) {
        Session::pause(self);
    }
    fn resume(&mut self) {
        Session::resume(self);
    }
    fn release(&mut self) {
        Session::release(self);
    }
    fn finish(self: Box<Self>, ok: bool) -> Outcome {
        Session::finish(*self, ok)
    }
}

/// This process's guard.
static RUN: Mutex<Option<Box<dyn Held>>> = Mutex::new(None);

/// This process's guard; a poisoned lock still holds it, so it can always let go.
fn slot() -> MutexGuard<'static, Option<Box<dyn Held>>> {
    RUN.lock().unwrap_or_else(PoisonError::into_inner)
}

/// This run's hold on the mouse; dropping it lets go.
pub struct Scope {
    on: bool,
}

/// Puts `session` in this process's slot until the returned scope ends.
pub(super) fn hold(session: Box<dyn Held>) -> Scope {
    *slot() = Some(session);
    Scope { on: true }
}

/// Starts the guard for a gate whose run holds the mouse as `mouse` says, when `cfg` has the guard on.
pub fn begin(cfg: &MouseGuard, mouse: Mouse) -> Result<Scope, String> {
    if !cfg.on || mouse == Mouse::Free {
        return Ok(Scope { on: false });
    }
    let clock = QpcClock::new().map_err(|e| format!("mouse guard: {e}"))?;
    let replay = mouse == Mouse::Drives;
    let session = Session::begin(WinHook::new(clock), clock, cfg, replay)?;
    Ok(hold(Box::new(session)))
}

impl Scope {
    /// Lets go as the gate ends, `ok` or broken off; how the guard ended and what it counted, when it held the mouse.
    pub fn finish(mut self, ok: bool) -> Option<Outcome> {
        if !std::mem::replace(&mut self.on, false) {
            return None;
        }
        let session = slot().take();
        session.map(|s| s.finish(ok))
    }
}

impl Drop for Scope {
    fn drop(&mut self) {
        if self.on {
            let session = slot().take();
            drop(session);
        }
    }
}

/// Refuses new input once the guard let go early: a press, the time limit, or a lost hook.
pub fn allowed() -> Result<(), String> {
    slot().as_mut().map_or(Ok(()), |s| s.refusal())
}

/// Lets go of the mouse for the rest of the run, once the gate's pointer work is over.
pub fn let_go() {
    if let Some(s) = slot().as_mut() {
        s.release();
    }
}

/// Runs `prompt`, which may show Windows' admin prompt, with the mouse let go; the guard comes back after it. A gate
/// the guard stopped shows no prompt.
pub fn around_prompt<T>(prompt: impl FnOnce() -> Result<T, String>) -> Result<T, String> {
    allowed()?;
    if let Some(s) = slot().as_mut() {
        s.pause();
    }
    let out = prompt();
    if let Some(s) = slot().as_mut() {
        s.resume();
    }
    out
}

/// Sends kx-gates' own clicks or moves through `send`; while the guard holds the mouse, its hook must count them within
/// `alive_ms`, or the gate stops as lost.
pub fn own_input<T>(send: impl FnOnce() -> Result<T, String>) -> Result<T, String> {
    let base = slot().as_ref().map(|s| s.own());
    let out = send()?;
    if let (Some(base), Some(s)) = (base, slot().as_mut()) {
        s.sent(base);
    }
    Ok(out)
}

/// Adds how the guard ended and what it counted to result line `v`; an end that stops the gate also fails it.
pub fn mark(v: &mut Value, outcome: Option<Outcome>) {
    let (Some(Outcome { end, counts }), Some(fields)) = (outcome, v.as_object_mut()) else {
        return;
    };
    let guard = json!({
        "ended": end.name(), "why": end.why(), "dropped": counts.dropped, "passed": counts.passed,
    });
    fields.insert(FIELD.to_string(), guard);
    if end.stops() {
        fields.insert(PASS.to_string(), json!(false));
    }
}

#[cfg(test)]
mod tests;
