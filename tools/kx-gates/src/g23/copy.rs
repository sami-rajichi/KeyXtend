//! G23's copy of the test build, started from the plain kx-gates while no other product copy runs, on the portable data
//! folder beside it. The gate must have made that folder, marked as G20 marks it, since the test entry's notebook lands
//! there; the folder goes at the end.
#![cfg(windows)]

use std::path::{Path, PathBuf};

use kx_platform_windows::uiaccess;

use crate::apps::Ctx;
use crate::copyend::{self, Ended};
use crate::product::NAME;
use crate::{clipcopy, clipdata, g8start, owncopy};

/// The test build and the data folder G23 made for it.
pub struct Fresh<'a> {
    ctx: &'a Ctx<'a>,
    /// The test build's program.
    pub program: PathBuf,
    data: PathBuf,
}

/// Refuses a run under uiAccess, which the test build would inherit, or beside a running product, whose keys and
/// panel G23 could click instead of its own copy's.
fn refuse(ui_access: Result<bool, String>, running: bool) -> Result<(), String> {
    match ui_access {
        Err(e) => Err(format!("kx-gates cannot tell whether it has uiAccess: {e}")),
        Ok(true) => Err(
            "run the plain kx-gates (cargo run -p kx-gates): the test build would inherit uiAccess"
                .into(),
        ),
        Ok(false) if running => Err(format!(
            "close {NAME} first: G23 clicks the keys and panel of its own copy"
        )),
        Ok(false) => Ok(()),
    }
}

/// Refuses data folder `data` unless the gate made it, `marked`: G23 saves its test entry there, so it must go.
fn ours(data: &Path, marked: bool) -> Result<(), String> {
    if marked {
        return Ok(());
    }
    Err(format!(
        "{} was not made by kx-gates, and G23 would leave its test entry there: remove it first",
        data.display()
    ))
}

/// The data folder's marker, which G20 and G23 share.
fn marker<'c>(ctx: &'c Ctx) -> &'c str {
    &ctx.cfg.g20_product.marker
}

impl<'a> Fresh<'a> {
    /// Readies the test build's data folder, made and marked by the gate; refused under uiAccess, beside a running
    /// copy, or on a folder the gate did not make.
    pub fn open(ctx: &'a Ctx<'a>) -> Result<Self, String> {
        refuse(
            uiaccess::check().map_err(|e| e.to_string()),
            owncopy::runs(),
        )?;
        let program = clipcopy::program(ctx)?;
        let data = clipdata::own(&program, marker(ctx))?;
        ours(&data, clipdata::marked(&data, marker(ctx)))?;
        Ok(Self { ctx, program, data })
    }

    /// A copy ready `[g23] ready_ms` after its keyboard shows, `drive` run on it, then ended: what `drive` gave, and
    /// how the copy went, none when none started.
    pub fn with_copy<T>(
        &self,
        drive: impl FnOnce() -> Result<T, String>,
    ) -> (Result<T, String>, Option<Ended>) {
        let started = match g8start::start(self.ctx, &self.program, &[], None) {
            Ok(s) => s,
            Err(f) => return (Err(f.why), f.ended),
        };
        let warm = self.ctx.cfg.g23.ready_ms;
        let got = clipcopy::ready(&started, warm).and_then(|()| drive());
        let ended = copyend::end(self.ctx, &started.process);
        (got, Some(ended))
    }

    /// Removes the data folder with the test entry in it, once the copy that went as `ended` no longer runs.
    pub fn close(self, ended: Option<&Ended>) -> Result<(), String> {
        if !copyend::may_follow(ended.map(|e| e.gone)) {
            return Err(format!(
                "{} stays, as the copy may still run: remove it once {NAME} has ended",
                self.data.display()
            ));
        }
        clipdata::finish(&self.data, marker(self.ctx))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_run_under_ui_access_or_beside_a_running_copy_is_refused() {
        assert_eq!(refuse(Ok(false), false), Ok(()));
        let e = refuse(Ok(true), false).expect_err("uiAccess");
        assert!(e.contains("uiAccess") && e.contains("cargo run"), "{e}");
        let e = refuse(Ok(false), true).expect_err("a copy runs");
        assert!(e.contains(&format!("close {NAME}")), "{e}");
        let e = refuse(Err("no token".into()), false).expect_err("unknown");
        assert!(e.contains("no token"), "{e}");
    }

    #[test]
    fn only_a_data_folder_the_gate_made_takes_the_test_entry() {
        let data = Path::new(r"D:\build\data");
        assert_eq!(ours(data, true), Ok(()));
        let e = ours(data, false).expect_err("someone else's folder");
        assert!(e.contains(r"D:\build\data") && e.contains("remove"), "{e}");
    }
}
