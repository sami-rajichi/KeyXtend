//! A run's mouse guard: it holds the mouse from the gate's start, lets go for an admin prompt, refuses new input once
//! the owner or the time limit ended it, and always lets go when the gate ends, breaks off or panics.
#![cfg(windows)]

use std::sync::{Mutex, MutexGuard, PoisonError};
use std::time::Duration;

use kx_module_api::{Clock, Mono};
use kx_platform_windows::QpcClock;
use serde_json::{Value, json};

use super::hook::WinHook;
use super::{Counts, End, Filter, MouseGuard};
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
    /// Removes the hook; why it had let go by itself, if it had, and what it counted.
    fn stop(self) -> (Option<End>, Counts);
}

/// How a run's guard ended, and what all its hooks counted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Outcome {
    /// Why it let go.
    pub end: End,
    /// The hand's events dropped and the own events passed.
    pub counts: Counts,
}

/// One run's guard over `H`'s hooks, timed by `C`.
pub struct Session<H: Hook, C: Clock> {
    hook: H,
    clock: C,
    cfg: MouseGuard,
    /// How long earlier hooks held the mouse in this run; the time limit counts only held time.
    used: Duration,
    /// When the running hook started.
    since: Mono,
    live: Option<H::Live>,
    /// Why it let go early, which stops the gate.
    early: Option<End>,
    /// True once the gate's pointer work is over, so the guard never comes back.
    idle: bool,
    /// What the hooks that ended counted.
    seen: Counts,
}

impl<H: Hook, C: Clock> Session<H, C> {
    /// Starts holding the mouse.
    pub fn begin(hook: H, clock: C, cfg: &MouseGuard) -> Result<Self, String> {
        let since = clock.now();
        let live = hook.start(Filter::new(cfg, since, Duration::ZERO))?;
        say!("{}", cfg.line());
        Ok(Self {
            hook,
            clock,
            cfg: cfg.clone(),
            used: Duration::ZERO,
            since,
            live: Some(live),
            early: None,
            idle: false,
            seen: Counts::default(),
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
        self.unhook(End::Prompt);
    }

    /// Holds the mouse again after an admin prompt, unless it let go early or for good; the prompt's time does not
    /// count against the limit.
    pub fn resume(&mut self) {
        if self.live.is_some() || self.early.is_some() || self.idle {
            return;
        }
        self.since = self.clock.now();
        let mut filter = Filter::new(&self.cfg, self.since, self.used);
        if let Some(end) = filter.tick(self.since) {
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

    /// Lets go for the rest of the run, as the gate's pointer work is over.
    pub fn release(&mut self) {
        self.unhook(End::Idle);
        self.idle = true;
    }

    /// Lets go as the gate ends, `ok` or broken off; why the guard ended, an early end first, and what it counted.
    pub fn finish(mut self, ok: bool) -> Outcome {
        let why = if ok { End::Done } else { End::Error };
        self.unhook(why);
        Outcome {
            end: self.early.unwrap_or(why),
            counts: self.seen,
        }
    }

    /// Removes a running hook and adds its time to `used` and its counts to `seen`, logging `why` unless it had let
    /// go by itself (it logged that itself).
    fn unhook(&mut self, why: End) {
        if let Some(live) = self.live.take() {
            self.used += self.clock.now().saturating_since(self.since);
            let (own, counts) = live.stop();
            self.seen.add(counts);
            match own {
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
        self.unhook(why);
    }
}

/// A session as this process's slot holds it, whatever its hook and clock.
pub trait Held: Send {
    /// See `Session::refusal`.
    fn refusal(&mut self) -> Result<(), String>;
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

/// This process's guard, which `keys`, `admin` and the presses reach without a run context.
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

/// Starts the guard for a gate that `held` says holds the mouse, when `cfg` has the guard on.
pub fn begin(cfg: &MouseGuard, held: bool) -> Result<Scope, String> {
    if !(cfg.on && held) {
        return Ok(Scope { on: false });
    }
    let clock = QpcClock::new().map_err(|e| format!("mouse guard: {e}"))?;
    Ok(hold(Box::new(Session::begin(
        WinHook::new(clock),
        clock,
        cfg,
    )?)))
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

/// Refuses new input once the guard let go early: the owner, the time limit, or a guard that could not come back.
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
