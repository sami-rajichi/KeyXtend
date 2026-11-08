//! G19's copy of the test build, started from the plain kx-gates while no other product copy runs, on a data folder the
//! gate made and marks as G20 does, so Translate's consent starts unsaved; the folder goes at the end.
#![cfg(windows)]

use std::path::PathBuf;

use kx_platform_windows::uiaccess;

use crate::apps::Ctx;
use crate::copyend::{self, Ended};
use crate::g8start::{self, Started};
use crate::product::{self, NAME};
use crate::testbuild::warned;
use crate::{clipcopy, clipdata, owncopy};

/// The test build and the data folder G19 made for it.
pub struct Fresh<'a> {
    ctx: &'a Ctx<'a>,
    /// The test build's program.
    pub program: PathBuf,
    data: PathBuf,
}

/// Why G19 will not start: uiAccess the copy would inherit, a running copy, or a data folder the gate did not make.
fn refused(ui_access: Result<bool, String>, running: bool, ours: bool) -> Option<String> {
    match ui_access {
        Err(e) => Some(format!("kx-gates cannot tell whether it has uiAccess: {e}")),
        Ok(true) => Some(
            "run the plain kx-gates (cargo run -p kx-gates): the copy would inherit uiAccess"
                .into(),
        ),
        Ok(false) if running => Some(format!(
            "close {NAME} first: G19 reads the strip of its own copy"
        )),
        Ok(false) if !ours => {
            Some("the test build's data folder was not made by kx-gates: remove it first".into())
        }
        Ok(false) => None,
    }
}

impl<'a> Fresh<'a> {
    /// Readies the test build's data folder, made and marked by the gate.
    pub fn open(ctx: &'a Ctx<'a>) -> Result<Self, String> {
        let program = clipcopy::program(ctx)?;
        let marker = &ctx.cfg.g20_product.marker;
        let running = owncopy::runs();
        let ui = uiaccess::check().map_err(|e| e.to_string());
        if let Some(why) = refused(ui, running, true) {
            return Err(why);
        }
        let data = clipdata::own(&program, marker)?;
        if let Some(why) = refused(Ok(false), false, clipdata::marked(&data, marker)) {
            return Err(why);
        }
        Ok(Self { ctx, program, data })
    }

    /// Refuses a copy started with `[gate_input] flag` that logged no gate warning, so its engine might hear the hand.
    fn warns(&self, s: &Started, flagged: bool) -> Result<(), String> {
        let g = &self.ctx.cfg.gate_input;
        let log = product::files(&self.program)?.log;
        let text = std::fs::read_to_string(&log).unwrap_or_default();
        if flagged && !warned(&text, s.at_us, &g.warn_line) {
            return Err(
                "the test build logged no gate warning, so its engine might hear the hand".into(),
            );
        }
        Ok(())
    }

    /// A copy, with `[gate_input] flag` when `flagged`, ready `[g19_product] ready_ms` after its keyboard shows,
    /// `drive` run on it with its process id, then ended: what `drive` gave, and how the copy went.
    pub fn with_copy<T>(
        &self,
        flagged: bool,
        drive: impl FnOnce(u32) -> Result<T, String>,
    ) -> (Result<T, String>, Option<Ended>) {
        let flag = if flagged {
            vec![self.ctx.cfg.gate_input.flag.clone()]
        } else {
            Vec::new()
        };
        let started = match g8start::start(self.ctx, &self.program, &flag, None) {
            Ok(s) => s,
            Err(f) => return (Err(f.why), f.ended),
        };
        let warm = self.ctx.cfg.g19_product.ready_ms;
        let ready = clipcopy::ready(&started, warm).and_then(|()| self.warns(&started, flagged));
        let got = ready.and_then(|()| drive(started.process.pid()));
        let ended = copyend::end(self.ctx, &started.process);
        (got, Some(ended))
    }

    /// Removes the data folder once the copy that went as `ended` no longer runs.
    pub fn close(self, ended: Option<&Ended>) -> Result<(), String> {
        if !copyend::may_follow(ended.map(|e| e.gone)) {
            return Err(format!(
                "{} stays, as the copy may still run: remove it once {NAME} has ended",
                self.data.display()
            ));
        }
        clipdata::finish(&self.data, &self.ctx.cfg.g20_product.marker)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uiaccess_a_running_copy_or_someone_elses_folder_is_refused() {
        assert_eq!(refused(Ok(false), false, true), None);
        let ui = refused(Ok(true), false, true).expect("uiAccess");
        assert!(ui.contains("cargo run"), "{ui}");
        let running = refused(Ok(false), true, true).expect("a copy runs");
        assert!(running.contains(&format!("close {NAME}")), "{running}");
        let theirs = refused(Ok(false), false, false).expect("not ours");
        assert!(theirs.contains("remove it"), "{theirs}");
        let unknown = refused(Err("no token".into()), false, true).expect("unknown");
        assert!(unknown.contains("no token"), "{unknown}");
    }
}
