//! A run's mouse guard: it holds the mouse from the gate's start, lets go for an admin prompt, refuses new input once
//! the owner or the time limit ended it, and always lets go when the gate ends, breaks off or panics.
#![cfg(windows)]

use std::sync::{Mutex, MutexGuard, PoisonError};

use kx_module_api::{Clock, Mono};
use kx_platform_windows::QpcClock;
use serde_json::{Value, json};

use crate::guardhook::WinHook;
use crate::mouseguard::{End, Filter, MouseGuard};
use crate::out::say;

/// The result line's field that says how the guard ended.
const FIELD: &str = "mouse_guard";
/// The result line's verdict, which a stopped run fails.
const PASS: &str = "pass";

/// Starts hooks that apply a filter.
pub trait Hook {
    /// A running hook.
    type Live: Live;
    /// Installs a hook that applies `filter`.
    fn start(&self, filter: Filter) -> Result<Self::Live, String>;
}

/// A running hook.
pub trait Live {
    /// Why it let go by itself, once it has: the owner or the time limit.
    fn ended(&self) -> Option<End>;
    /// Removes the hook; why it had let go by itself, if it had.
    fn stop(self) -> Option<End>;
}

/// One run's guard over `H`'s hooks, timed by `C`.
pub struct Session<H: Hook, C: Clock> {
    hook: H,
    clock: C,
    cfg: MouseGuard,
    /// When the guard first started; the time limit counts from here.
    start: Mono,
    live: Option<H::Live>,
    /// Why it let go early, which stops the gate.
    early: Option<End>,
}

impl<H: Hook, C: Clock> Session<H, C> {
    /// Starts holding the mouse.
    pub fn begin(hook: H, clock: C, cfg: &MouseGuard) -> Result<Self, String> {
        let start = clock.now();
        let live = hook.start(Filter::new(cfg, start))?;
        say!("{}", cfg.line());
        Ok(Self {
            hook,
            clock,
            cfg: cfg.clone(),
            start,
            live: Some(live),
            early: None,
        })
    }

    /// Refuses new input once the guard let go early, saying why.
    pub fn refusal(&mut self) -> Result<(), String> {
        if self.early.is_none() {
            self.early = self.live.as_ref().and_then(Live::ended);
        }
        self.early.map_or(Ok(()), |e| Err(e.stopped()))
    }

    /// Lets go while an admin prompt is up.
    pub fn pause(&mut self) {
        self.let_go(End::Prompt);
    }

    /// Holds the mouse again after an admin prompt, unless it let go early; the time limit counts from the first start.
    pub fn resume(&mut self) {
        if self.live.is_some() || self.early.is_some() {
            return;
        }
        let mut filter = Filter::new(&self.cfg, self.start);
        if let Some(end) = filter.tick(self.clock.now()) {
            say!("{}", end.line());
            self.early = Some(end);
            return;
        }
        match self.hook.start(filter) {
            Ok(live) => {
                self.live = Some(live);
                say!("{}", self.cfg.line());
            }
            Err(e) => {
                say!("{}: {e}", End::Lost.line());
                self.early = Some(End::Lost);
            }
        }
    }

    /// Lets go as the gate ends, `ok` or broken off; why the guard ended, an early end first.
    pub fn finish(mut self, ok: bool) -> End {
        let why = if ok { End::Done } else { End::Error };
        self.let_go(why);
        self.early.unwrap_or(why)
    }

    /// Removes a running hook, logging `why` unless it had let go by itself (it logged that itself).
    fn let_go(&mut self, why: End) {
        if let Some(live) = self.live.take() {
            match live.stop() {
                Some(own) => self.early = self.early.or(Some(own)),
                None => say!("{}", why.line()),
            }
        }
    }
}

impl<H: Hook, C: Clock> Drop for Session<H, C> {
    fn drop(&mut self) {
        let why = if std::thread::panicking() {
            End::Panic
        } else {
            End::Error
        };
        self.let_go(why);
    }
}

/// This process's guard, which `keys` and `admin` reach without a run context.
static RUN: Mutex<Option<Session<WinHook, QpcClock>>> = Mutex::new(None);

/// This process's guard; a poisoned lock still holds it, so it can always let go.
fn slot() -> MutexGuard<'static, Option<Session<WinHook, QpcClock>>> {
    RUN.lock().unwrap_or_else(PoisonError::into_inner)
}

/// This run's hold on the mouse; dropping it lets go.
pub struct Scope {
    on: bool,
}

/// Starts the guard for `gate`, when `cfg` covers it.
pub fn begin(cfg: &MouseGuard, gate: &str) -> Result<Scope, String> {
    if !cfg.covers(gate) {
        return Ok(Scope { on: false });
    }
    let clock = QpcClock::new().map_err(|e| format!("mouse guard: {e}"))?;
    let session = Session::begin(WinHook::new(clock), clock, cfg)?;
    *slot() = Some(session);
    Ok(Scope { on: true })
}

impl Scope {
    /// Lets go as the gate ends, `ok` or broken off; how the guard ended, when it held the mouse.
    pub fn finish(mut self, ok: bool) -> Option<End> {
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

/// Refuses new input once the guard let go early: the owner, the time limit, or a guard that could not come back.
pub fn allowed() -> Result<(), String> {
    slot().as_mut().map_or(Ok(()), Session::refusal)
}

/// Runs `prompt`, which may show Windows' admin prompt, with the mouse let go; the guard comes back after it.
pub fn around_prompt<T>(prompt: impl FnOnce() -> T) -> T {
    if let Some(s) = slot().as_mut() {
        s.pause();
    }
    let out = prompt();
    if let Some(s) = slot().as_mut() {
        s.resume();
    }
    out
}

/// Adds how the guard ended to result line `v`; an end that stops the gate also fails it.
pub fn mark(v: &mut Value, end: Option<End>) {
    let (Some(end), Some(fields)) = (end, v.as_object_mut()) else {
        return;
    };
    fields.insert(
        FIELD.to_string(),
        json!({ "ended": end.name(), "why": end.why() }),
    );
    if end.stops() {
        fields.insert(PASS.to_string(), json!(false));
    }
}

#[cfg(test)]
mod tests;
