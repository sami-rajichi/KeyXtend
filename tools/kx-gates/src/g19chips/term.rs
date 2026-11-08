//! The gate's own terminals: Windows Terminal as `[apps.terminal]` starts it, and the classic console as
//! `[g19_product.console]` names it, each on a script of the gate's in its out folder; closing one ends what it started
//! and deletes its files.
#![cfg(windows)]

use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

use kx_platform_windows::window::foreground;
use windows::Win32::Foundation::HWND;
use windows::Win32::System::Threading::{AttachThreadInput, GetCurrentThreadId};
use windows::Win32::UI::Input::KeyboardAndMouse::{GetKeyState, VK_NUMLOCK};

use crate::apps::{AppKind, Ctx, INPUT, Opened};
use crate::launch::launch;
use crate::winfind::{self, Match};
use crate::{out, win};

/// The toggle bit of a key state.
const TOGGLED: i16 = 1;

/// Which terminal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// Windows Terminal.
    Terminal,
    /// The classic console.
    Console,
}

impl Kind {
    /// Its name in the result line and in its files' names.
    pub fn name(self) -> &'static str {
        match self {
            Self::Terminal => "terminal",
            Self::Console => "console",
        }
    }
}

/// A terminal the gate started: its window, its program, and its script's files.
pub struct Term {
    /// Its window.
    pub hwnd: HWND,
    child: Child,
    files: [PathBuf; 3],
}

impl Term {
    /// The lines the script wrote to its out file so far.
    pub fn lines(&self) -> Vec<String> {
        let text = std::fs::read_to_string(&self.files[2]).unwrap_or_default();
        text.lines().map(String::from).collect()
    }

    /// The terminal as the gate's acts take an app: its window, nothing for the clean-up to close.
    pub fn app(&self) -> Opened {
        Opened {
            kind: AppKind::Terminal,
            hwnd: self.hwnd,
            child: None,
            file: None,
            title_has: None,
        }
    }
}

/// The script, ready and out files of terminal `kind` running script `tag`.
fn files(ctx: &Ctx, kind: Kind, tag: &str) -> [PathBuf; 3] {
    let named = |part: &str| ctx.file(&format!("-{}-{tag}{part}", kind.name()));
    [named(".ps1"), named("-ready.txt"), named("-out.txt")]
}

/// The classic console on `script`, and its window once it shows.
fn console(ctx: &Ctx, script: &str) -> Result<(HWND, Child), String> {
    let (c, wait) = (
        &ctx.cfg.g19_product.console,
        ctx.cfg.app(AppKind::Terminal.name())?.wait_ms,
    );
    let m = Match {
        class: Some(c.class.clone()),
        skip: win::snapshot(),
        ..Match::default()
    };
    let args = c.args.iter().map(|a| a.replace(INPUT, script));
    let mut child = (Command::new(&c.exe).args(args))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("{}: {e}", c.exe))?;
    let Some(hwnd) = winfind::wait_for(&m, wait, ctx.cfg.timing.poll_ms) else {
        let _ = child.kill();
        return Err(format!("no console window after {wait} ms"));
    };
    Ok((hwnd, child))
}

/// Terminal `kind` on the script `text` writes for its ready and out files, which the gate names `tag`, once the
/// script marked itself ready.
pub fn open(
    ctx: &Ctx,
    kind: Kind,
    tag: &str,
    text: impl FnOnce(&Path, &Path) -> String,
) -> Result<Term, String> {
    let files = files(ctx, kind, tag);
    out::write(&files[0], &text(&files[1], &files[2]))?;
    let script = files[0].to_string_lossy().into_owned();
    let started = match kind {
        Kind::Terminal => launch(ctx, AppKind::Terminal.name(), &script, None),
        Kind::Console => console(ctx, &script),
    };
    let (hwnd, child) = match started {
        Ok(s) => s,
        Err(e) => return Err(format!("{e}; clean-up: {:?}", remove(&files))),
    };
    let term = Term { hwnd, child, files };
    let wait = ctx.cfg.app(AppKind::Terminal.name())?.wait_ms;
    match out::wait_read(&term.files[1], wait, ctx.cfg.timing.poll_ms) {
        Ok(_) => Ok(term),
        Err(e) => Err(format!(
            "{} not ready: {e}; clean-up: {:?}",
            kind.name(),
            close(ctx, term)
        )),
    }
}

/// Closes `term`'s window, ends its program and deletes its files; what was left.
pub fn close(ctx: &Ctx, mut term: Term) -> Vec<String> {
    let t = &ctx.cfg.timing;
    let mut left = Vec::new();
    let _ = win::close(term.hwnd);
    if !winfind::wait_gone(term.hwnd, t.close_wait_ms, t.poll_ms) {
        left.push(format!("left open: {}", win::describe(term.hwnd)));
    }
    let _ = term.child.kill();
    left.extend(remove(&term.files));
    left
}

/// Deletes those of `files` that exist; what could not go.
fn remove(files: &[PathBuf]) -> Vec<String> {
    let gone = |f: &PathBuf| {
        std::fs::remove_file(f)
            .err()
            .map(|e| format!("{}: {e}", f.display()))
    };
    files
        .iter()
        .filter(|f| f.exists())
        .filter_map(gone)
        .collect()
}

/// True when key state `state` holds its toggle bit.
pub fn toggled(state: i16) -> bool {
    state & TOGGLED != 0
}

/// True while Num Lock is on, read through the input state of the window in front, which has taken every key sent;
/// the gate's own thread reads no keys, so its own state would be stale.
#[allow(unsafe_code, reason = "Plain thread and key-state queries.")]
pub fn numlock_on() -> bool {
    let front = win::owner(foreground()).1;
    // SAFETY: plain queries; the shared input state is let go before returning.
    unsafe {
        let me = GetCurrentThreadId();
        let joined = front != 0 && front != me && AttachThreadInput(me, front, true).as_bool();
        let state = GetKeyState(i32::from(VK_NUMLOCK.0));
        if joined {
            let _ = AttachThreadInput(me, front, false);
        }
        toggled(state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_low_bit_is_the_toggle() {
        assert!(toggled(1) && toggled(i16::MIN | 1));
        assert!(!toggled(0) && !toggled(i16::MIN), "held down, not on");
    }
}
