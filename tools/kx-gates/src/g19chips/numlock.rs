//! Num Lock in G19's terminals: put back to the state the run found first, only through the gate's own terminal in
//! front and also on a panic; every error tells the owner which state to restore.
#![cfg(windows)]

use serde_json::{Value, json};

use crate::apps::Opened;
use crate::g19chips::GATE;
use crate::g19chips::gate::Gate;
use crate::g19chips::term::{Term, numlock_on};
use crate::out::say;
use crate::ownkeys::{self, Desk, Real};

/// What the owner does when Num Lock may stay changed: set it back to `was`, its state before the run.
pub fn restore(was: bool) -> String {
    let state = if was { "on" } else { "off" };
    format!("Num Lock was {state} before the run; turn it back {state} with the on-screen keyboard")
}

/// The state Num Lock had before the run: the one `carried` from the first terminal, else what `read` finds now.
pub fn first(
    carried: Option<bool>,
    read: impl FnOnce() -> Result<bool, String>,
) -> Result<bool, String> {
    carried.map_or_else(read, Ok)
}

/// Runs `back` when dropped while armed, as when the drive panics, and prints its error, since no result is left.
struct Guard<F: Fn() -> Result<(), String>> {
    back: F,
    armed: bool,
}

impl<F: Fn() -> Result<(), String>> Drop for Guard<F> {
    fn drop(&mut self) {
        if !self.armed {
            return;
        }
        if let Err(e) = (self.back)() {
            say!("{GATE}: {e}");
        }
    }
}

/// Runs `drive`, then `back` once, even when `drive` panics: what each gave.
pub fn then_back<T>(
    drive: impl FnOnce() -> T,
    back: impl Fn() -> Result<(), String>,
) -> (T, Result<(), String>) {
    let mut guard = Guard { back, armed: true };
    let got = drive();
    guard.armed = false;
    (got, (guard.back)())
}

/// Num Lock put back to `was` on the gate's own terminal `app`, read by `on` and tapped by `tap` only while `app` is
/// in front on `desk`, then `settle_ms` given; an error says why it is not back and what to do.
pub fn back_on(
    desk: &dyn Desk,
    app: &Opened,
    (was, settle_ms): (bool, u64),
    on: &dyn Fn() -> Result<bool, String>,
    tap: &dyn Fn() -> Result<(), String>,
) -> Result<(), String> {
    let not = |e: String| format!("Num Lock not put back: {e}. {}", restore(was));
    if on().map_err(not)? != was {
        ownkeys::click(desk, app, tap).map_err(not)?;
        desk.settle(settle_ms);
    }
    if on().map_err(not)? != was {
        return Err(not("it stayed changed".into()));
    }
    Ok(())
}

/// Num Lock put back to `was` in terminal `t`, never brought forward, as that would tap into the window in front.
pub fn put_back(gate: &Gate, t: &Term, was: bool) -> Result<(), String> {
    let g = gate.g();
    let tap = || crate::keys::restore_combo(&g.numlock);
    let on = || numlock_on(t.hwnd);
    back_on(&Real, &t.app(), (was, g.settle_ms), &on, &tap)
}

/// One terminal's result with Num Lock put back as `back` says: it fails, naming why, when Num Lock is not back.
pub fn finished(mut v: Value, back: Result<(), String>) -> Value {
    v["ok"] = json!(v["ok"] == true && back.is_ok());
    v["numlock_back"] = json!(back.is_ok());
    if let Err(e) = back {
        v["numlock_error"] = json!(e);
    }
    v
}

/// One terminal's outcome: the drive's `got` with Num Lock's `back`, whose error also goes into `left`, so a later
/// failed step cannot drop it.
pub fn settled(
    got: Result<Value, String>,
    back: Result<(), String>,
    left: &mut Vec<String>,
) -> Result<Value, String> {
    if let Err(e) = &back {
        left.push(e.clone());
    }
    got.map(|v| finished(v, back))
}

#[cfg(test)]
mod tests;
