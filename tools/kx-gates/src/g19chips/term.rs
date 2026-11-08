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

/// A terminal the gate started: its window, its program, its script's files, and what its ready file says.
pub struct Term {
    /// Its window.
    pub hwnd: HWND,
    /// What the script wrote to mark itself ready.
    pub mark: String,
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

/// What starting a terminal gave: its window and program, or why no window showed, so its case did not run.
type Started = Result<(HWND, Child), String>;

/// The classic console on `script`, and its window once it shows; else why none showed, its program ended.
fn console(ctx: &Ctx, script: &str) -> Result<Started, String> {
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
        return Ok(Err(format!("no {} window after {wait} ms", c.class)));
    };
    Ok(Ok((hwnd, child)))
}

/// Terminal `kind` on the script `text` writes for its ready and out files, which the gate names `tag`, once the
/// script marked itself ready; else why the console showed no window, its files gone, so the script ends too.
pub fn open(
    ctx: &Ctx,
    kind: Kind,
    tag: &str,
    text: impl FnOnce(&Path, &Path) -> String,
) -> Result<Result<Term, String>, String> {
    let wait = ctx.cfg.app(AppKind::Terminal.name())?.wait_ms;
    let files = files(ctx, kind, tag);
    out::write(&files[0], &text(&files[1], &files[2]))?;
    let script = files[0].to_string_lossy().into_owned();
    let started = match kind {
        Kind::Terminal => launch(ctx, AppKind::Terminal.name(), &script, None).map(Ok),
        Kind::Console => console(ctx, &script),
    };
    let (hwnd, child) = match started {
        Ok(Ok(s)) => s,
        Ok(Err(why)) => return unshown(why, &remove(&files)),
        Err(e) => return Err(format!("{e}; clean-up: {:?}", remove(&files))),
    };
    let mut term = Term {
        hwnd,
        mark: String::new(),
        child,
        files,
    };
    match out::wait_read(&term.files[1], wait, ctx.cfg.timing.poll_ms) {
        Ok(mark) => {
            term.mark = mark;
            Ok(Ok(term))
        }
        Err(e) => Err(format!(
            "{} not ready: {e}; clean-up: {:?}",
            kind.name(),
            close(ctx, term)
        )),
    }
}

/// A terminal that showed no window, for `why`: it did not run once its files are gone, else what `left` says fails.
fn unshown<T>(why: String, left: &[String]) -> Result<Result<T, String>, String> {
    if left.is_empty() {
        Ok(Err(why))
    } else {
        Err(format!("{why}; clean-up: {left:?}"))
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

/// The thread whose input a Num Lock read shares: `own`'s `thread`, only while window `own` is the one in `front`
/// and runs on another thread than `me`.
pub fn attach_to(front: isize, own: isize, thread: u32, me: u32) -> Option<u32> {
    (front == own && thread != 0 && thread != me).then_some(thread)
}

/// True while Num Lock is on, read through the input state of the gate's own terminal `own` while it is in front, as
/// it has taken every key sent; an error, with nothing shared, while another window is in front.
#[allow(unsafe_code, reason = "Plain thread and key-state queries.")]
pub fn numlock_on(own: HWND) -> Result<bool, String> {
    let thread = win::owner(own).1;
    // SAFETY: a plain query of this thread's id.
    let me = unsafe { GetCurrentThreadId() };
    let front = foreground();
    let Some(to) = attach_to(win::key(front), win::key(own), thread, me) else {
        let e = format!("Num Lock not read: {} is in front", win::describe(front));
        return Err(e);
    };
    // SAFETY: plain queries on the terminal's thread; the shared input state is let go before returning.
    unsafe {
        if !AttachThreadInput(me, to, true).as_bool() {
            return Err("Num Lock not read: the terminal's input could not be shared".into());
        }
        let state = GetKeyState(i32::from(VK_NUMLOCK.0));
        let _ = AttachThreadInput(me, to, false);
        Ok(toggled(state))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn num_lock_is_read_only_through_our_own_terminal_in_front() {
        let (ours, other, me) = (7, 9, 40);
        assert_eq!(attach_to(ours, ours, 41, me), Some(41));
        assert_eq!(
            attach_to(other, ours, 41, me),
            None,
            "another window in front"
        );
        assert_eq!(attach_to(ours, ours, 0, me), None, "a closed terminal");
        assert_eq!(attach_to(ours, ours, me, me), None, "the gate's own thread");
    }

    #[test]
    fn the_shipped_console_stays_in_conhost_and_runs_the_gates_script() {
        let c = crate::config::load().expect("loads").g19_product.console;
        let first = c.args.first().map(String::as_str);
        assert_eq!(first, Some("-ForceNoHandoff"), "never the default terminal");
        assert!(c.args.last().is_some_and(|a| a == INPUT), "{:?}", c.args);
    }

    #[test]
    fn a_console_with_no_window_did_not_run_only_once_its_files_went() {
        let why = || "no ConsoleWindowClass window after 9 ms".to_string();
        assert_eq!(unshown::<()>(why(), &[]), Ok(Err(why())));
        let kept = unshown::<()>(why(), &["a.ps1: in use".into()]).expect_err("a file stayed");
        assert!(
            kept.contains("a.ps1: in use") && kept.contains("9 ms"),
            "{kept}"
        );
    }

    #[test]
    fn only_the_low_bit_is_the_toggle() {
        assert!(toggled(1) && toggled(i16::MIN | 1));
        assert!(!toggled(0) && !toggled(i16::MIN), "held down, not on");
    }
}
