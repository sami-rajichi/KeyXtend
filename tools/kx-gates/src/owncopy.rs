//! The product copies a gate starts on settings it writes over the owner's (G21's product half, G27, `g8 predict`).
//! The owner's file is kept, and every copy is ended before it comes back: by Quit, else through its handle.
#![cfg(windows)]

use std::path::PathBuf;
use std::time::Instant;

use kx_module_api::ModuleId;
use serde_json::{Value, json};

use crate::apps::Ctx;
use crate::close::{self, Gone};
use crate::g8start::{self, Started};
use crate::ownset::{self, Kept};
use crate::probe;
use crate::product::{self, NAME};

/// The owner's settings file, kept while a gate runs its own copies of `[g8] program`.
pub struct Session<'a> {
    ctx: &'a Ctx<'a>,
    program: PathBuf,
    kept: Kept,
}

/// A copy a session started; dropping it ends it, so even a panic leaves none running.
pub struct Copy<'a> {
    ctx: &'a Ctx<'a>,
    started: Option<Started>,
}

/// How a copy went away, and what was let go after it had to be ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ended {
    /// Quit by itself, or ended.
    pub gone: Gone,
    /// Buttons or keys let go after an end.
    pub released: Vec<String>,
}

impl Ended {
    /// True only when the copy quit by itself.
    pub fn ok(&self) -> bool {
        self.gone.ok()
    }

    /// Its result line.
    pub fn line(&self) -> Value {
        json!({ "quit": self.gone.quit, "ended": self.gone.ended, "released": self.released })
    }
}

impl<'a> Session<'a> {
    /// Keeps the owner's file; refused while the product runs, as a copy reads its settings only at start.
    pub fn open(ctx: &'a Ctx<'a>) -> Result<Self, String> {
        if product::running().is_some() {
            return Err(format!(
                "close {NAME} first: this gate starts its own copy on its own settings"
            ));
        }
        let program = ctx.cfg.program(&ctx.cfg.g8.program);
        let kept = Kept::take(&product::files(&program)?)?;
        Ok(Self { ctx, program, kept })
    }

    /// Writes the gate's settings, predict `on` with learning paused and the modules `off` switched off, then starts a
    /// copy on them.
    pub fn start(&self, on: bool, off: &[ModuleId]) -> Result<Copy<'a>, String> {
        self.kept
            .write(&ownset::predict(&self.kept.table(), on, off)?)?;
        let started = g8start::start(self.ctx, &self.program, &[], None)?;
        Ok(Copy {
            ctx: self.ctx,
            started: Some(started),
        })
    }

    /// A copy with predict on and the modules `off` switched off, ready after `[g27] ready_ms`, `drive` run on it, then
    /// ended: what `drive` gave, and how the copy went, none when it never started.
    pub fn with_copy<T>(
        &self,
        off: &[ModuleId],
        drive: impl FnOnce() -> Result<T, String>,
    ) -> (Result<T, String>, Option<Ended>) {
        let copy = match self.start(true, off) {
            Ok(copy) => copy,
            Err(e) => return (Err(e), None),
        };
        let got = copy.ready(self.ctx.cfg.g27.ready_ms).and_then(|_| drive());
        (got, Some(copy.end()))
    }

    /// Puts the owner's file back; called once every copy has ended.
    pub fn close(self) -> Result<(), String> {
        self.kept.put_back()
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

    /// Ends it: Quit, else through its handle, then lets go of any button or key it left down.
    pub fn end(mut self) -> Ended {
        self.finish()
    }

    /// Ends it once; a copy already ended reads as quit.
    fn finish(&mut self) -> Ended {
        let Some(s) = self.started.take() else {
            let gone = Gone {
                quit: true,
                ended: true,
            };
            return Ended {
                gone,
                released: Vec::new(),
            };
        };
        let gone = close::end_started(self.ctx, &s.process);
        let released = if gone.quit {
            Vec::new()
        } else {
            probe::released()
        };
        Ended { gone, released }
    }
}

impl Drop for Copy<'_> {
    fn drop(&mut self) {
        let _ = self.finish();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_copy_that_had_to_be_ended_fails_and_its_line_says_what_was_let_go() {
        let gone = |quit, ended| Gone { quit, ended };
        let quit = Ended {
            gone: gone(true, true),
            released: Vec::new(),
        };
        assert!(quit.ok());
        let forced = Ended {
            gone: gone(false, true),
            released: vec!["released a button or key left down".into()],
        };
        assert!(!forced.ok(), "ended by force is a failure");
        let line = forced.line();
        assert_eq!(
            (line["quit"].as_bool(), line["ended"].as_bool()),
            (Some(false), Some(true))
        );
        assert_eq!(line["released"].as_array().map(Vec::len), Some(1));
    }
}
