//! The product's test build, which P4's gates drive: the plain kx-gates starts it with the gate flag and a data folder
//! of its own, after the owner's running copy quit, and starts that copy again when the test build ends.
#![cfg(windows)]

mod parts;

use std::path::{Path, PathBuf};

use kx_mod_keyboard::{KeyboardSettings, Role};
use kx_mod_mouse::MouseSettings;
use kx_module_api::{ModuleId, Settings, SettingsSpec};
use kx_platform::PORTABLE_DIR;
use kx_platform_windows::uiaccess;
use kx_settings::{peek, peek_section};
use spike_core::weak::Job;

pub use parts::Closed;
use parts::{SLOW, START_MENU, holds, not_back, plain, to_restore, warned};

use crate::apps::Ctx;
use crate::cfgfile::P4_FILE;
use crate::hooklog::{self, HookLine};
use crate::keymap::Reader;
use crate::out::say;
use crate::proc::Process;
use crate::product::{self, Keyboard, NAME};
use crate::{admin, board, close, g8start, mouse, out, win};

/// The product's test build while it runs; dropping it quits it and starts the owner's copy again, best effort.
pub struct TestBuild<'a> {
    ctx: &'a Ctx<'a>,
    /// Its program, as a plain absolute path.
    program: PathBuf,
    /// Its process, until it is asked to quit.
    process: Option<Process>,
    /// When it was started, in µs since 1970; its log's lines from then on are its own.
    since_us: i64,
    /// The owner's copy it closed, to start again.
    owner: Option<PathBuf>,
}

impl<'a> TestBuild<'a> {
    /// Starts the test build, in `job` when given, once the running copy quit; it is ready when its keyboard shows and
    /// its log holds the gate warning. Refused under uiAccess, which the test build would inherit.
    pub fn open(ctx: &'a Ctx<'a>, job: Option<&Job>) -> Result<Self, String> {
        plain(uiaccess::check().map_err(|e| e.to_string()))?;
        let program = checked(ctx)?;
        let folder = program.parent().ok_or("the test build has no folder")?;
        let data = folder.join(PORTABLE_DIR);
        std::fs::create_dir_all(&data).map_err(|e| format!("{}: {e}", data.display()))?;
        let owner = quit_running(ctx, &program)?;
        let mut build = Self {
            ctx,
            program,
            process: None,
            since_us: 0,
            owner,
        };
        let flag = [ctx.cfg.gate_input.flag.clone()];
        let started = g8start::start(ctx, &build.program, &flag, job)?;
        build.since_us = started.at_us;
        build.process = Some(started.process);
        build.ready()?;
        Ok(build)
    }

    /// Waits up to `start_ms` for its gate warning and its keyboard. Without the warning it fails, as its engine might
    /// act on the hand, and the drop quits it.
    fn ready(&self) -> Result<(), String> {
        let (g, poll_ms) = (&self.ctx.cfg.gate_input, self.ctx.cfg.timing.poll_ms);
        let log = product::files(&self.program)?.log;
        let warns =
            || std::fs::read_to_string(&log).is_ok_and(|t| warned(&t, self.since_us, &g.warn_line));
        let shown = || (warns() && Keyboard::find().is_ok()).then_some(());
        match win::poll_until(g.start_ms, poll_ms, shown) {
            Some(()) => Ok(()),
            None if !warns() => Err(format!(
                "the test build logged no gate warning within {} ms, so its engine might act on the hand; it quits",
                g.start_ms
            )),
            None => Err(format!(
                "the test build showed no keyboard within {} ms",
                g.start_ms
            )),
        }
    }

    /// Its `[mouse]` settings, read from its data folder without changing them.
    pub fn mouse(&self) -> Result<MouseSettings, String> {
        self.section(kx_mod_mouse::ID, &kx_mod_mouse::SPEC)
    }

    /// Its `[keyboard]` settings, read from its data folder without changing them.
    pub fn keyboard(&self) -> Result<KeyboardSettings, String> {
        self.section(kx_mod_keyboard::ID, &kx_mod_keyboard::SPEC)
    }

    /// Module `id`'s section of its settings, as a later copy of the app reads it.
    fn section<T: Settings>(&self, id: ModuleId, spec: &'static SettingsSpec) -> Result<T, String> {
        let file = peek(&product::files(&self.program)?);
        peek_section(file.as_ref(), id, spec)
            .map_err(|e| format!("the test build's [{}]: {e}", id.as_str()))
    }

    /// Turns the key with `role` on or off with a `TAG` click, unless it already reads so, and waits up to `lit_ms` for
    /// it to read so; a mode key lights only once the hook's engine applied the toggle.
    pub fn set(&self, role: Role, on: bool) -> Result<(), String> {
        let (g, poll_ms) = (&self.ctx.cfg.gate_input, self.ctx.cfg.timing.poll_ms);
        let id = board::id_of(role)?;
        let (reader, keyboard) = (Reader::new()?, Keyboard::find()?);
        let state = || {
            let read = reader.toggled(keyboard.hwnd, &id)?;
            read.ok_or_else(|| format!("key {id} has no on or off state"))
        };
        if state()? == on {
            return Ok(());
        }
        mouse::click_at(keyboard.at(&id)?)?;
        let lit = || state().ok().filter(|now| *now == on).map(drop);
        let word = if on { "on" } else { "off" };
        win::poll_until(g.lit_ms, poll_ms, lit)
            .ok_or_else(|| format!("key {id} did not read {word} within {} ms", g.lit_ms))
    }

    /// Quits it, reads its stats lines logged since its start, then starts the owner's copy again.
    pub fn close(mut self) -> Closed {
        let quit = self.quit();
        let lines = self.lines();
        let owner_back = self.restore(quit);
        Closed {
            quit,
            owner_back,
            lines,
        }
    }

    /// Asks it to quit, once; whether it ended in time.
    fn quit(&mut self) -> bool {
        let process = self.process.take();
        process.is_some_and(|p| close::quit_started(self.ctx, &p).unwrap_or(false))
    }

    /// Its stats lines since its start.
    fn lines(&self) -> Result<Vec<HookLine>, String> {
        let log = product::files(&self.program)?.log;
        let text = std::fs::read_to_string(&log).map_err(|e| format!("{}: {e}", log.display()))?;
        hooklog::since(&text, self.since_us, &self.ctx.cfg.gate_input.hook_line)
    }

    /// Starts the owner's copy again through the shell, once, but only after the test build ended, else says to start it
    /// by hand; whether it started, none when it closed no copy.
    fn restore(&mut self, quit: bool) -> Option<bool> {
        let owner = self.owner.take()?;
        let back = quit && admin::start_shown(&owner, &[]).is_ok();
        if !back {
            say!("{}", not_back(quit));
        }
        Some(back)
    }
}

impl Drop for TestBuild<'_> {
    fn drop(&mut self) {
        let quit = self.process.is_none() || self.quit();
        let _ = self.restore(quit);
    }
}

/// The test build's program as a plain path, once its bytes show a gate-input build.
fn checked(ctx: &Ctx) -> Result<PathBuf, String> {
    let g = &ctx.cfg.gate_input;
    let program = ctx.cfg.program(&g.program);
    let not_built = |e| format!("{}: {e}; see [gate_input] in {P4_FILE}", program.display());
    let bytes = std::fs::read(&program).map_err(not_built)?;
    if !holds(&bytes, &g.flag) {
        let p = program.display();
        return Err(format!(
            "{p} is not a gate-input build; see [gate_input] in {P4_FILE}"
        ));
    }
    out::plain(&program)
}

/// Asks the running copy, if any, to quit; the copy to start again later, never the test build at `program`.
fn quit_running(ctx: &Ctx, program: &Path) -> Result<Option<PathBuf>, String> {
    if product::running().is_none() {
        return Ok(None);
    }
    let (exe, ended) = close::quit_running(ctx).map_err(|e| {
        format!(
            "{e}: close it with `kx-gates close {NAME}` from the installed kx-gates first, and start it again after \
             this gate"
        )
    })?;
    if !ended {
        return Err(format!(
            "the running {NAME} {SLOW} and is not started again; close it, try again, then {START_MENU}"
        ));
    }
    Ok(to_restore(Some(&exe), program))
}
