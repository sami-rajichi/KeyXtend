//! A run's mouse guard: it holds the mouse from the gate's start, lets go for an admin prompt, refuses new input once
//! a press, the time limit or a lost hook ended it, and always lets go when the gate ends, breaks off or panics.

use std::sync::Arc;
use std::time::Duration;

use kx_module_api::{Clock, Mono};

use super::{Counts, End, Filter, Loss, MouseGuard, Tally, Watch};
use crate::out::say;

/// Starts hooks that apply a filter.
pub trait Hook {
    /// A running hook.
    type Live: Live;
    /// Installs a hook that applies `filter`.
    fn start(&self, filter: Filter) -> Result<Self::Live, String>;
}

/// A running hook.
pub trait Live {
    /// Why it let go by itself, once it has: a press or the time limit.
    fn ended(&self) -> Option<End>;
    /// Removes the hook; why it had let go by itself, if it had.
    fn stop(self) -> Option<End>;
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
    /// What every hook of this run counted, read live.
    tally: Arc<Tally>,
    /// Whether the running hook still counts kx-gates' own clicks and moves.
    watch: Watch,
}

impl<H: Hook, C: Clock> Session<H, C> {
    /// Starts holding the mouse.
    pub fn begin(hook: H, clock: C, cfg: &MouseGuard) -> Result<Self, String> {
        let (since, tally) = (clock.now(), Arc::new(Tally::default()));
        let filter = Filter::new(cfg, since, Duration::ZERO, Arc::clone(&tally));
        let live = hook.start(filter)?;
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
            tally,
            watch: Watch::new(cfg),
        })
    }

    /// Refuses new input once the guard let go early, saying why; the tally knows as soon as the hook's callback does.
    pub fn refusal(&mut self) -> Result<(), String> {
        if self.early.is_none() {
            let own = || self.live.as_ref().and_then(Live::ended);
            self.early = self.tally.ended().or_else(own);
        }
        self.check_alive();
        self.early.map_or(Ok(()), |e| Err(e.stopped()))
    }

    /// kx-gates' own events its hooks counted so far; read before sending clicks or moves, for `sent`.
    pub fn own(&self) -> u64 {
        self.tally.passed()
    }

    /// Notes clicks or moves kx-gates sent after its hooks had counted `base` own events; while a hook holds the mouse, it
    /// must count one within `alive_ms`.
    pub fn sent(&mut self, base: u64) {
        if self.live.is_some() && self.early.is_none() {
            self.watch.sent(self.clock.now(), base);
        }
    }

    /// Lets go while an admin prompt is up.
    pub fn pause(&mut self) {
        self.check_alive();
        self.unhook(End::Prompt);
    }

    /// Holds the mouse again after an admin prompt, unless it let go early or for good; the prompt's time does not
    /// count against the limit.
    pub fn resume(&mut self) {
        if self.live.is_some() || self.early.is_some() || self.idle {
            return;
        }
        self.since = self.clock.now();
        let mut filter = Filter::new(&self.cfg, self.since, self.used, Arc::clone(&self.tally));
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
                let lost = End::Lost(Loss::Prompt);
                say!("{}: {e}", lost.line());
                self.early = Some(lost);
            }
        }
    }

    /// Lets go for the rest of the run, as the gate's pointer work is over.
    pub fn release(&mut self) {
        self.check_alive();
        self.unhook(End::Idle);
        self.idle = true;
    }

    /// Lets go as the gate ends, `ok` or broken off; why the guard ended, an early end first, and what it counted.
    pub fn finish(mut self, ok: bool) -> Outcome {
        self.check_alive();
        let why = if ok { End::Done } else { End::Error };
        self.unhook(why);
        Outcome {
            end: self.early.unwrap_or(why),
            counts: self.tally.counts(),
        }
    }

    /// Lets go as lost when the running hook counted none of kx-gates' own clicks or moves within `alive_ms`, as Windows
    /// switched it off.
    fn check_alive(&mut self) {
        let held = self.live.is_some() && self.early.is_none();
        if held && self.watch.lost(self.clock.now(), self.tally.passed()) {
            let lost = End::Lost(Loss::Off);
            self.unhook(lost);
            self.early = self.early.or(Some(lost));
        }
    }

    /// Removes a running hook and adds its time to `used`, logging `why` unless it had let go by itself (it logged
    /// that itself). Clicks it has not counted yet are no longer judged.
    fn unhook(&mut self, why: End) {
        if let Some(live) = self.live.take() {
            self.used += self.clock.now().saturating_since(self.since);
            self.watch.clear();
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
        self.unhook(why);
    }
}

#[cfg(test)]
mod tests;
