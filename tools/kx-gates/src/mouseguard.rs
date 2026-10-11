//! The mouse guard's rules: while a gate runs, physical mouse input drops and injected input passes, until the gate
//! ends, the time limit runs out or the owner holds a button down. `guardhook` applies them; `guardrun` runs them.
#![cfg(windows)]

use std::time::Duration;

use kx_module_api::{Mono, US_PER_MS};
use serde::Deserialize;
use windows::Win32::UI::WindowsAndMessaging::{
    WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MBUTTONDOWN, WM_MBUTTONUP, WM_RBUTTONDOWN, WM_RBUTTONUP,
    WM_XBUTTONDOWN, WM_XBUTTONUP, XBUTTON1,
};

/// Gates the guard never covers: the hand try, G13 and G23 wait on the owner's own mouse, G26 runs on the CI runner,
/// and `close` moves nothing.
const UNGUARDED: [&str; 5] = ["assist", "g13", "g23", "g26", "close"];

/// The left button's bit; one bit per button, so the guard knows which presses it dropped.
pub const LEFT: u8 = 1;
/// The right button's bit.
pub const RIGHT: u8 = 1 << 1;
/// The middle button's bit.
pub const MIDDLE: u8 = 1 << 2;
/// The first X (side) button's bit.
pub const X1: u8 = 1 << 3;
/// The second X (side) button's bit.
pub const X2: u8 = 1 << 4;
/// The X button's number sits in the high word of a hook's mouse data.
const X_SHIFT: u32 = 16;

/// `[mouse_guard]` in `kx-gates.toml`, shared by every gate; times in ms.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MouseGuard {
    /// Guard the mouse at all.
    pub on: bool,
    /// Longest the guard holds the mouse in one run.
    pub limit_ms: u64,
    /// A physical button held this long gives the mouse back and stops the gate.
    pub escape_ms: u64,
}

impl MouseGuard {
    /// Refuses a zero limit or hold, or a hold the limit always cuts short.
    pub fn check(&self) -> Result<(), String> {
        if self.limit_ms == 0 || self.escape_ms == 0 || self.escape_ms >= self.limit_ms {
            return Err(
                "mouse_guard.limit_ms and escape_ms must be above 0, and escape_ms below limit_ms"
                    .to_string(),
            );
        }
        Ok(())
    }

    /// True when the guard holds the mouse while `gate` runs.
    pub fn covers(&self, gate: &str) -> bool {
        self.on && !UNGUARDED.contains(&gate)
    }

    /// The log line when the guard starts; it names no position or key.
    pub fn line(&self) -> String {
        format!(
            "mouse guard on: physical mouse input is ignored for up to {} ms; hold a mouse button {} ms to get it \
             back, which stops the gate",
            self.limit_ms, self.escape_ms
        )
    }
}

/// Why the guard let go of the mouse.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum End {
    /// The gate ended.
    Done,
    /// The gate broke off with an error.
    Error,
    /// The gate panicked.
    Panic,
    /// An admin prompt is up; the guard comes back after it.
    Prompt,
    /// The owner held a physical button for the escape time.
    Owner,
    /// The time limit ran out.
    Limit,
    /// The guard lost its hook: it could not come back after an admin prompt, or its thread could not wait.
    Lost,
}

impl End {
    /// Its name in the result line.
    pub fn name(self) -> &'static str {
        match self {
            Self::Done => "done",
            Self::Error => "error",
            Self::Panic => "panic",
            Self::Prompt => "prompt",
            Self::Owner => "owner",
            Self::Limit => "limit",
            Self::Lost => "lost",
        }
    }

    /// Why it let go, for the log line and the result.
    pub fn why(self) -> &'static str {
        match self {
            Self::Done => "the gate ended",
            Self::Error => "the gate broke off",
            Self::Panic => "the gate panicked",
            Self::Prompt => "an admin prompt is up",
            Self::Owner => "the owner held a mouse button, so the gate stops and does not pass",
            Self::Limit => "its time limit ran out, so the gate stops and does not pass",
            Self::Lost => "it lost its hook, so the gate stops and does not pass",
        }
    }

    /// True when the gate must stop: the owner, the limit, or a guard that could not come back.
    pub fn stops(self) -> bool {
        matches!(self, Self::Owner | Self::Limit | Self::Lost)
    }

    /// The log line when the guard ends; it names no position or key.
    pub fn line(self) -> String {
        format!("mouse guard off: {}", self.why())
    }

    /// The error that refuses the gate's next input once the guard let go early.
    pub fn stopped(self) -> String {
        format!("stopped by the mouse guard: {}", self.why())
    }
}

/// A mouse event as the guard sees it: a button's press or release, or anything else.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// The button with this bit went down.
    Down(u8),
    /// The button with this bit went up.
    Up(u8),
    /// A move, a wheel turn or anything else.
    Other,
}

/// The event of low-level mouse message `msg` with mouse data `data`.
pub fn kind_of(msg: u32, data: u32) -> Kind {
    let x = if data >> X_SHIFT == u32::from(XBUTTON1) {
        X1
    } else {
        X2
    };
    match msg {
        WM_LBUTTONDOWN => Kind::Down(LEFT),
        WM_LBUTTONUP => Kind::Up(LEFT),
        WM_RBUTTONDOWN => Kind::Down(RIGHT),
        WM_RBUTTONUP => Kind::Up(RIGHT),
        WM_MBUTTONDOWN => Kind::Down(MIDDLE),
        WM_MBUTTONUP => Kind::Up(MIDDLE),
        WM_XBUTTONDOWN => Kind::Down(x),
        WM_XBUTTONUP => Kind::Up(x),
        _ => Kind::Other,
    }
}

/// What happens to one event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// Windows gets the event.
    Pass,
    /// Nothing gets the event.
    Drop,
}

/// The guard's decisions over one hook's life: which events drop, and when it lets go.
#[derive(Debug, Clone)]
pub struct Filter {
    /// When the time limit runs out.
    deadline: Mono,
    /// How long a held button takes to let go.
    escape: Duration,
    /// Buttons whose physical press the guard dropped.
    held: u8,
    /// When the first of them went down.
    since: Option<Mono>,
    /// Why it let go, once it has; every event then passes.
    out: Option<End>,
}

impl Filter {
    /// A filter for `cfg` whose time limit counts from `start`.
    pub fn new(cfg: &MouseGuard, start: Mono) -> Self {
        Self {
            deadline: start.saturating_add(Duration::from_millis(cfg.limit_ms)),
            escape: Duration::from_millis(cfg.escape_ms),
            held: 0,
            since: None,
            out: None,
        }
    }

    /// Why it lets go at `now`, if it does: the time limit, or a button held for the escape time.
    pub fn tick(&mut self, now: Mono) -> Option<End> {
        if self.out.is_none() {
            let held = self.since.map(|t| now.saturating_since(t));
            if now >= self.deadline {
                self.out = Some(End::Limit);
            } else if held.is_some_and(|h| h >= self.escape) {
                self.out = Some(End::Owner);
            }
        }
        self.out
    }

    /// Whether an event at `now` passes: injected input does, and so does a physical release whose press came before
    /// the guard, so no button sticks.
    pub fn event(&mut self, injected: bool, kind: Kind, now: Mono) -> Verdict {
        if injected || self.tick(now).is_some() {
            return Verdict::Pass;
        }
        match kind {
            Kind::Down(b) => {
                self.since = self.since.or(Some(now));
                self.held |= b;
            }
            Kind::Up(b) if self.held & b == 0 => return Verdict::Pass,
            Kind::Up(b) => {
                self.held &= !b;
                self.since = self.since.filter(|_| self.held != 0);
            }
            Kind::Other => {}
        }
        Verdict::Drop
    }

    /// How long until `tick` could let go: the time limit, or a held button's escape, whichever is first.
    pub fn due_in(&self, now: Mono) -> Duration {
        let limit = self.deadline.saturating_since(now);
        let escape = self
            .since
            .map(|t| t.saturating_add(self.escape).saturating_since(now));
        escape.map_or(limit, |e| e.min(limit))
    }
}

/// `d` in whole ms, rounded up, for a wait; the largest wait past that.
pub fn wait_ms(d: Duration) -> u32 {
    let ms = d.as_micros().div_ceil(u128::from(US_PER_MS));
    u32::try_from(ms).unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests;
