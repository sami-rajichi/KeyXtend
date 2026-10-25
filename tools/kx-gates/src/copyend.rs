//! How a gate ends a product copy it started: Quit first, then through the copy's handle when it did not end in time,
//! then any button or key it left down is let go. Only a clean quit passes; a crash, an end or a running copy fails.
#![cfg(windows)]

use kx_platform_windows::WindowsPlatform;
use serde_json::{Value, json};

use crate::apps::Ctx;
use crate::close::{Asked, ask_quit_tries, ended_within};
use crate::probe;
use crate::proc::Process;
use crate::win;

/// The exit code of a clean quit: std's `ExitCode::SUCCESS`, which the product returns.
const CLEAN: u32 = 0;
/// The most waits for a copy's end `finish` makes: after Quit, and after an end through its handle.
pub(crate) const END_WAITS: usize = 2;

/// How a copy a gate started went away.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gone {
    /// It took Quit and ended in time, cleanly.
    Quit,
    /// It ended by itself before Quit reached it, or with another exit code, or one Windows could not read.
    Crashed(Option<u32>),
    /// It had to be ended through its handle.
    Ended,
    /// It still runs: Quit and the end both failed.
    Running,
}

impl Gone {
    /// True only for a clean quit.
    pub fn ok(self) -> bool {
        self == Self::Quit
    }

    /// True once it no longer runs.
    pub fn over(self) -> bool {
        self != Self::Running
    }

    /// Its words in a result line.
    pub(crate) fn words(self) -> String {
        match self {
            Self::Quit => "quit".to_string(),
            Self::Crashed(Some(code)) => format!("crashed with exit code {code}"),
            Self::Crashed(None) => "crashed".to_string(),
            Self::Ended => "was ended by the gate".to_string(),
            Self::Running => "still runs; close it".to_string(),
        }
    }
}

/// How a copy went, a note when ending it failed, and what was let go after.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ended {
    /// How it went.
    pub gone: Gone,
    /// Why the end failed, when it did.
    pub note: Option<String>,
    /// Buttons or keys let go after a copy that did not quit cleanly.
    pub released: Vec<String>,
}

impl Ended {
    /// True only for a clean quit.
    pub fn ok(&self) -> bool {
        self.gone.ok()
    }

    /// Its result line.
    pub fn line(&self) -> Value {
        json!({ "gone": self.gone.words(), "ok": self.ok(), "note": self.note, "released": self.released })
    }
}

/// How a copy went after the Quit tries, `asked`: `within` waits for its end, `code` reads its exit code, and when it
/// did not end, `end` ends it; an end that failed on a copy already gone is only a note.
pub(crate) fn finish(
    asked: &Result<Asked, String>,
    within: impl Fn() -> bool,
    end: impl FnOnce() -> Result<(), String>,
    code: impl Fn() -> Option<u32>,
) -> (Gone, Option<String>) {
    let sent = matches!(asked, Ok(Asked::Sent(_)));
    if asked.is_ok() && within() {
        return (by_itself(sent, code()), None);
    }
    let failed = end().err();
    if !within() {
        return (Gone::Running, failed);
    }
    match failed {
        None => (Gone::Ended, None),
        Some(e) => (by_itself(sent, code()), Some(e)),
    }
}

/// A copy that ended by itself: quit only when Quit reached it, `sent`, and its exit code is clean, crashed otherwise.
/// A crash that exits with a clean code after Quit reached it reads as quit, as Windows gives no other sign of it.
fn by_itself(sent: bool, code: Option<u32>) -> Gone {
    if sent && code == Some(CLEAN) {
        Gone::Quit
    } else {
        Gone::Crashed(code)
    }
}

/// Ends `process`, a copy a gate started, as the module says; no copy is left running unless Windows refuses both.
pub fn end(ctx: &Ctx, process: &Process) -> Ended {
    let g = &ctx.cfg.g8;
    let platform = WindowsPlatform::new().map_err(|e| e.to_string());
    let asked = platform.and_then(|p| {
        let wait = || win::sleep_ms(g.quit_try_ms);
        ask_quit_tries(&*p.instance(), g.quit_tries, || process.ended(), wait)
    });
    let within = || ended_within(ctx, process);
    let (gone, note) = finish(&asked, within, || process.end(), || process.exit_code());
    let released = if gone.ok() {
        Vec::new()
    } else {
        probe::released()
    };
    Ended {
        gone,
        note,
        released,
    }
}

/// True when a next copy may start after the last went `gone`: none started, or it no longer runs.
pub fn may_follow(gone: Option<Gone>) -> bool {
    gone.is_none_or(Gone::over)
}

#[cfg(test)]
mod tests;
