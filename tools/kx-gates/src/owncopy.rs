//! The product copies a gate starts on settings it writes over the owner's (G21's product half, G27, `g8 predict`).
//! The owner's bytes wait in a sidecar file before the first write, and every copy is ended before they come back.
#![cfg(windows)]

use std::cell::Cell;
use std::path::PathBuf;
use std::time::Instant;

use kx_module_api::ModuleId;
use kx_platform::ui::Instance;
use kx_platform_windows::WindowsPlatform;

use crate::apps::Ctx;
use crate::copyend::{self, Ended, Gone};
use crate::g8start::{self, Failed, Started};
use crate::out::say;
use crate::ownset::{self, Kept};
use crate::p7cfg;
use crate::product::{self, NAME};

/// The owner's settings file, kept while a gate runs its own copies of `[g8] program`.
pub struct Session<'a> {
    ctx: &'a Ctx<'a>,
    program: PathBuf,
    kept: Kept,
    /// Set once a copy could not be ended, so the backup stays.
    stray: Cell<bool>,
}

/// A copy a session started; it borrows the session, so the session cannot close while it lives. Dropping it ends
/// it, so even a panic leaves none running.
pub struct Copy<'s> {
    session: &'s Session<'s>,
    started: Option<Started>,
}

/// The program these gates start: `[g8] program`.
fn program(ctx: &Ctx) -> PathBuf {
    ctx.cfg.program(&ctx.cfg.g8.program)
}

/// The backup of the owner's files: `[owner_settings] backup` in kx-gates' out folder; refused when target-window's
/// log or a gate file (`p7cfg::gate_files`) has its name, as each start of target-window deletes that log.
pub(crate) fn side(ctx: &Ctx) -> Result<PathBuf, String> {
    let backup = &ctx.cfg.owner_settings.backup;
    let log = ctx.target.log.file_name().map(|n| n.to_string_lossy());
    let taken: Vec<&str> = log
        .iter()
        .map(AsRef::as_ref)
        .chain(p7cfg::gate_files(ctx.cfg))
        .collect();
    p7cfg::backup_free(backup, &taken)?;
    Ok(ctx.out.join(backup))
}

/// Marks `stray` when a copy that went as `ended` may still run.
fn note(stray: &Cell<bool>, ended: Option<&Ended>) {
    if !copyend::may_follow(ended.map(|e| e.gone)) {
        stray.set(true);
    }
}

/// Restores the owner's files from a backup a stopped gate left, once no product copy runs: whether one was there.
/// `ended` is false when the copy `close` asked to quit did not end, so it counts as running.
pub fn recover(ctx: &Ctx, ended: bool) -> Result<bool, String> {
    let files = product::files(&program(ctx))?;
    ownset::recover(&files, &side(ctx)?, ended, runs)
}

impl<'a> Session<'a> {
    /// Keeps the owner's files in the backup, and first restores them from a backup a stopped run left, naming it.
    /// Refused while the product runs, as a copy reads its settings only at start.
    pub fn open(ctx: &'a Ctx<'a>) -> Result<Self, String> {
        if runs() {
            return Err(format!(
                "close {NAME} first: this gate starts its own copy on its own settings"
            ));
        }
        let (program, side) = (program(ctx), side(ctx)?);
        let (kept, recovered) = Kept::take(&product::files(&program)?, &side)?;
        if recovered {
            say!(
                "the owner's settings came back first from {}, which a stopped run left",
                side.display()
            );
        }
        Ok(Self {
            ctx,
            program,
            kept,
            stray: Cell::default(),
        })
    }

    /// Writes the gate's settings, predict `on` with learning paused and the modules `off` switched off, then starts a
    /// copy on them.
    pub fn start(&self, on: bool, off: &[ModuleId]) -> Result<Copy<'_>, Failed> {
        self.kept
            .write(&ownset::predict(&self.kept.table(), on, off)?)?;
        let started = g8start::start(self.ctx, &self.program, &[], None)
            .inspect_err(|f| note(&self.stray, f.ended.as_ref()))?;
        Ok(Copy {
            session: self,
            started: Some(started),
        })
    }

    /// A copy with predict on and the modules `off` switched off, ready after `[g27] ready_ms`, `drive` run on it, then
    /// ended: what `drive` gave, and how the copy went, none when no copy started.
    pub fn with_copy<T>(
        &self,
        off: &[ModuleId],
        drive: impl FnOnce() -> Result<T, String>,
    ) -> (Result<T, String>, Option<Ended>) {
        let copy = match self.start(true, off) {
            Ok(copy) => copy,
            Err(f) => return (Err(f.why), f.ended),
        };
        let got = copy.ready(self.ctx.cfg.g27.ready_ms).and_then(|_| drive());
        (got, Some(copy.end()))
    }

    /// Puts the owner's files back and deletes the backup once they read back as kept; the backup stays while a copy
    /// it started may still run.
    pub fn close(self) -> Result<(), String> {
        self.kept.put_back(self.stray.get())
    }
}

impl Copy<'_> {
    /// Waits until `warm_ms` past its keyboard showing: the copy, or an error when it never showed.
    pub fn ready(&self, warm_ms: u64) -> Result<&Started, String> {
        let s = self.started.as_ref().ok_or("the copy has ended")?;
        let shown = s.shown_at.ok_or("the keyboard did not show in time")?;
        std::thread::sleep(g8start::warm_left(shown, warm_ms, Instant::now()));
        Ok(s)
    }

    /// Ends it as `copyend` does.
    pub fn end(mut self) -> Ended {
        self.finish()
    }

    /// Ends it once, as `once` says.
    fn finish(&mut self) -> Ended {
        let session = self.session;
        once(&mut self.started, |s| {
            let ended = copyend::end(session.ctx, &s.process);
            note(&session.stray, Some(&ended));
            ended
        })
    }
}

/// Ends the copy in `started` with `end` and takes it; one already taken reads as quit.
fn once<S>(started: &mut Option<S>, end: impl FnOnce(S) -> Ended) -> Ended {
    started.take().map_or_else(
        || Ended {
            gone: Gone::Quit,
            note: None,
            released: Vec::new(),
        },
        end,
    )
}

/// True when a copy holds `instance`'s session claim; a claim this look takes goes at once.
fn held(instance: &dyn Instance) -> bool {
    instance.claim().is_err()
}

/// True while a product copy runs: one with a window, or one exiting windowless that still holds its session claim
/// and may yet save; true when the platform cannot be asked.
fn runs() -> bool {
    product::running().is_some() || WindowsPlatform::new().map_or(true, |p| held(&*p.instance()))
}

impl Drop for Copy<'_> {
    fn drop(&mut self) {
        let _ = self.finish();
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use kx_platform_fake::FakeInstance;

    use super::*;

    fn ended(gone: Gone) -> Ended {
        Ended {
            gone,
            note: None,
            released: Vec::new(),
        }
    }

    #[test]
    fn only_a_copy_that_may_still_run_keeps_the_backup() {
        let stray = Cell::new(false);
        note(&stray, None);
        for gone in [Gone::Quit, Gone::Crashed(None), Gone::Ended] {
            note(&stray, Some(&ended(gone)));
        }
        assert!(!stray.get(), "every copy is gone");
        note(&stray, Some(&ended(Gone::Running)));
        note(&stray, Some(&ended(Gone::Quit)));
        assert!(stray.get(), "one copy may still run, and that stays");
    }

    #[test]
    fn a_copy_is_ended_once_and_after_that_reads_as_quit() {
        let (mut started, calls) = (Some(7_u8), Cell::new(0));
        let end = |_: u8| {
            calls.set(calls.get() + 1);
            ended(Gone::Ended)
        };
        assert_eq!(once(&mut started, end).gone, Gone::Ended);
        assert_eq!(once(&mut started, end).gone, Gone::Quit);
        assert_eq!(calls.get(), 1);
    }

    #[test]
    fn the_session_reads_as_held_only_while_a_copy_claims_it() {
        let session = FakeInstance::new();
        assert!(!held(&session));
        assert!(session.claim().is_ok(), "the look let go of its claim");
        let claim = session.claim().expect("a copy runs");
        assert!(held(&session));
        drop(claim);
        assert!(!held(&session));
    }
}
