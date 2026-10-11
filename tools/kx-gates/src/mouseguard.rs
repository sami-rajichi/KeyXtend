//! The mouse guard's rules: while a gate runs, physical mouse input drops and injected input passes, until the gate
//! ends, the time limit runs out, a button is pressed or Windows switches its hook off. `filter` decides, `hook` applies
//! it, `run` runs it and `slot` holds this process's run.
#![cfg(windows)]

mod counts;
#[cfg(test)]
pub mod fake;
mod filter;
mod hook;
mod run;
mod slot;
mod watch;

pub use counts::{Counts, Tally};
pub use filter::Filter;
pub use slot::{allowed, around_prompt, begin, let_go, mark, own_input};
use watch::Watch;

use std::time::Duration;

use kx_module_api::US_PER_MS;
use kx_platform_windows::keys::TAG;
use serde::Deserialize;
use windows::Win32::System::Threading::INFINITE;
use windows::Win32::UI::WindowsAndMessaging::{
    LLMHF_INJECTED, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MBUTTONDOWN, WM_MBUTTONUP, WM_RBUTTONDOWN,
    WM_RBUTTONUP, WM_XBUTTONDOWN, WM_XBUTTONUP, XBUTTON1,
};

use crate::hookio::SIM_TAG;

/// The longest wait the hook thread asks for, in ms: one below `INFINITE`, so every wait ends.
const LONGEST_WAIT_MS: u32 = INFINITE - 1;

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
    /// Physical button presses that give the mouse back and stop the gate.
    pub escape_clicks: u8,
    /// How close together those presses come, when more than one is set.
    pub escape_window_ms: u64,
    /// How soon the hook must count kx-gates' own clicks and moves; past it, Windows switched the hook off.
    pub alive_ms: u64,
}

impl MouseGuard {
    /// Refuses a zero limit, no escape click, a zero window or a zero alive time.
    pub fn check(&self) -> Result<(), String> {
        let times = [self.limit_ms, self.escape_window_ms, self.alive_ms];
        if self.escape_clicks == 0 || times.contains(&0) {
            return Err(
                "mouse_guard.limit_ms, escape_clicks, escape_window_ms and alive_ms must be above 0"
                    .to_string(),
            );
        }
        Ok(())
    }

    /// The log line when the guard starts; it names no position or key.
    pub fn line(&self) -> String {
        format!(
            "mouse guard on: physical mouse input is ignored for up to {} ms; {} to get it back, which stops the \
             gate",
            self.limit_ms,
            self.escape()
        )
    }

    /// How the owner gets the mouse back, in words.
    fn escape(&self) -> String {
        match self.escape_clicks {
            1 => "press any mouse button".to_string(),
            n => format!(
                "press a mouse button {n} times within {} ms",
                self.escape_window_ms
            ),
        }
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
    /// The gate's pointer work is over, so the guard lets go for the rest of the run.
    Idle,
    /// A physical button was pressed the escape's number of times: the owner's, or another program's click.
    Owner,
    /// The time limit ran out.
    Limit,
    /// The guard lost its hook, for the reason given.
    Lost(Loss),
}

/// How the guard lost its hook.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Loss {
    /// Windows switched it off: it counted none of kx-gates' own clicks or moves in time.
    Off,
    /// It could not come back after an admin prompt.
    Prompt,
    /// Its thread could not wait or lost its state.
    Thread,
}

impl End {
    /// Every end, each loss included.
    #[cfg(test)]
    pub const ALL: [Self; 10] = [
        Self::Done,
        Self::Error,
        Self::Panic,
        Self::Prompt,
        Self::Idle,
        Self::Owner,
        Self::Limit,
        Self::Lost(Loss::Off),
        Self::Lost(Loss::Prompt),
        Self::Lost(Loss::Thread),
    ];

    /// Its name in the result line.
    pub fn name(self) -> &'static str {
        match self {
            Self::Done => "done",
            Self::Error => "error",
            Self::Panic => "panic",
            Self::Prompt => "prompt",
            Self::Idle => "idle",
            Self::Owner => "owner",
            Self::Limit => "limit",
            Self::Lost(_) => "lost",
        }
    }

    /// Why it let go, for the log line and the result.
    pub fn why(self) -> &'static str {
        match self {
            Self::Done => "the gate ended",
            Self::Error => "the gate broke off",
            Self::Panic => "the gate panicked",
            Self::Prompt => "an admin prompt is up",
            Self::Idle => "the gate's pointer work is over",
            Self::Owner => "a mouse button was pressed, so the gate stops and does not pass",
            Self::Limit => "its time limit ran out, so the gate stops and does not pass",
            Self::Lost(Loss::Off) => {
                "Windows switched the mouse guard off, so the gate stops and does not pass"
            }
            Self::Lost(Loss::Prompt) => {
                "the mouse guard could not come back after an admin prompt, so the gate stops and does not pass"
            }
            Self::Lost(Loss::Thread) => {
                "the mouse guard's thread failed, so the gate stops and does not pass"
            }
        }
    }

    /// True when the gate must stop: a press, the limit, or a lost hook.
    pub fn stops(self) -> bool {
        matches!(self, Self::Owner | Self::Limit | Self::Lost(_))
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

/// True for a hook event with `flags` and `extra` info that kx-gates injected itself; other programs' injected input
/// counts as the hand's.
pub fn ours(flags: u32, extra: usize) -> bool {
    flags & LLMHF_INJECTED != 0 && (extra == TAG || extra == SIM_TAG)
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

/// `d` in whole ms, rounded up, for a wait; never `INFINITE`, so the hook thread always checks its time again.
pub fn wait_ms(d: Duration) -> u32 {
    let ms = d.as_micros().div_ceil(u128::from(US_PER_MS));
    u32::try_from(ms).map_or(LONGEST_WAIT_MS, |ms| ms.min(LONGEST_WAIT_MS))
}

#[cfg(test)]
mod tests;
