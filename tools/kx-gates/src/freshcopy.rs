//! A gate's copy of the test build, started from the plain kx-gates while no other product copy runs, on the data
//! folder beside it that the gate made and marks as G20 does; the folder goes at the end, after a panic too once no
//! copy runs.
#![cfg(windows)]

use std::path::{Path, PathBuf};

use kx_platform_windows::uiaccess;

use crate::apps::Ctx;
use crate::copyend::{self, Ended};
use crate::g8start::{self, Started};
use crate::product::{self, NAME};
use crate::testbuild::warned;
use crate::{clipcopy, clipdata, owncopy};

/// The test build and the data folder a gate made for it.
pub struct Fresh<'a> {
    ctx: &'a Ctx<'a>,
    /// The test build's program.
    pub program: PathBuf,
    data: PathBuf,
    /// Set by `close`, so the drop guard leaves the folder to it.
    closed: bool,
}

/// Refuses gate `gate` under uiAccess, which the copy would inherit, or beside a running product, whose keys it could
/// click instead of its own copy's.
fn refuse(gate: &str, ui_access: Result<bool, String>, running: bool) -> Result<(), String> {
    match ui_access {
        Err(e) => Err(format!("kx-gates cannot tell whether it has uiAccess: {e}")),
        Ok(true) => Err(
            "run the plain kx-gates (cargo run -p kx-gates): the test build would inherit uiAccess"
                .into(),
        ),
        Ok(false) if running => Err(format!(
            "close {NAME} first: {gate} drives only its own copy"
        )),
        Ok(false) => Ok(()),
    }
}

/// Refuses data folder `data` for gate `gate` unless the gate made it, `marked`, as it goes at the end.
fn ours(gate: &str, data: &Path, marked: bool) -> Result<(), String> {
    if marked {
        return Ok(());
    }
    Err(format!(
        "{} was not made by kx-gates, and {gate} removes it at the end: remove it first",
        data.display()
    ))
}

/// Refuses a copy whose log, read by `log` only when a `warn` line is due, holds no such line stamped since `at_us`.
fn heard(
    warn: Option<&str>,
    log: impl FnOnce() -> Result<String, String>,
    at_us: i64,
) -> Result<(), String> {
    let Some(line) = warn else {
        return Ok(());
    };
    if !warned(&log()?, at_us, line) {
        return Err(
            "the gate build logged no gate warning, so it may not run as the gate asks".into(),
        );
    }
    Ok(())
}

/// The data folder's marker, which the gates share with G20.
fn marker<'c>(ctx: &'c Ctx) -> &'c str {
    &ctx.cfg.g20_product.marker
}

impl<'a> Fresh<'a> {
    /// Readies the test build's data folder for gate `gate`, made and marked by the gate; refused under uiAccess,
    /// beside a running copy, or on a folder the gate did not make.
    pub fn open(ctx: &'a Ctx<'a>, gate: &str) -> Result<Self, String> {
        Self::open_at(ctx, gate, clipcopy::program(ctx)?)
    }

    /// Like `open`, for the gate build at `program`.
    pub fn open_at(ctx: &'a Ctx<'a>, gate: &str, program: PathBuf) -> Result<Self, String> {
        let ui = uiaccess::check().map_err(|e| e.to_string());
        refuse(gate, ui, owncopy::runs())?;
        let data = clipdata::own(&program, marker(ctx))?;
        ours(gate, &data, clipdata::marked(&data, marker(ctx)))?;
        Ok(Self {
            ctx,
            program,
            data,
            closed: false,
        })
    }

    /// Refuses a copy that logged no `warn` line, when one is due, so it may not run as the gate asks.
    fn warns(&self, s: &Started, warn: Option<&str>) -> Result<(), String> {
        heard(warn, || self.log(), s.at_us)
    }

    /// The copy's log as it stands; empty before it has one.
    pub fn log(&self) -> Result<String, String> {
        let file = product::files(&self.program)?.log;
        Ok(std::fs::read_to_string(file).unwrap_or_default())
    }

    /// Writes the copy's settings file `table` into its data folder, before a copy starts.
    #[cfg_attr(
        not(feature = "voice"),
        allow(dead_code, reason = "Only the voice gates call it.")
    )]
    pub fn write(&self, table: &toml::Table) -> Result<(), String> {
        let file = product::files(&self.program)?.settings;
        std::fs::write(&file, table.to_string()).map_err(|e| format!("the copy's settings: {e}"))
    }

    /// A copy, with `[gate_input] flag` when `flagged`, ready `ready_ms` after its keyboard shows, `drive` run on it
    /// with its process id, then ended: what `drive` gave, and how the copy went, none when none started.
    pub fn with_copy<T>(
        &self,
        (flagged, ready_ms): (bool, u64),
        drive: impl FnOnce(u32) -> Result<T, String>,
    ) -> (Result<T, String>, Option<Ended>) {
        let g = &self.ctx.cfg.gate_input;
        let flag: Vec<String> = flagged.then(|| g.flag.clone()).into_iter().collect();
        let warn = flagged.then_some(g.warn_line.as_str());
        self.with_args(&flag, ready_ms, warn, drive)
    }

    /// A copy started with `args`, ready `ready_ms` after its keyboard shows and once it logged `warn` when given,
    /// `drive` run on it with its process id, then ended: what `drive` gave, and how the copy went.
    pub fn with_args<T>(
        &self,
        args: &[String],
        ready_ms: u64,
        warn: Option<&str>,
        drive: impl FnOnce(u32) -> Result<T, String>,
    ) -> (Result<T, String>, Option<Ended>) {
        let started = match g8start::start(self.ctx, &self.program, args, None) {
            Ok(s) => s,
            Err(f) => return (Err(f.why), f.ended),
        };
        let ready = clipcopy::ready(&started, ready_ms).and_then(|()| self.warns(&started, warn));
        let got = ready.and_then(|()| drive(started.process.pid()));
        let ended = copyend::end(self.ctx, &started.process);
        (got, Some(ended))
    }

    /// Removes the data folder once the copy that went as `ended` no longer runs.
    pub fn close(mut self, ended: Option<&Ended>) -> Result<(), String> {
        self.closed = true;
        if !copyend::may_follow(ended.map(|e| e.gone)) {
            return Err(format!(
                "{} stays, as the copy may still run: remove it once {NAME} has ended",
                self.data.display()
            ));
        }
        clipdata::finish(&self.data, marker(self.ctx))
    }
}

impl Drop for Fresh<'_> {
    /// After a panic or an early return, removes the gate's folder once no product copy runs.
    fn drop(&mut self) {
        if !self.closed && !owncopy::runs() {
            let _ = clipdata::finish(&self.data, marker(self.ctx));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_run_under_ui_access_or_beside_a_running_copy_is_refused() {
        assert_eq!(refuse("G19", Ok(false), false), Ok(()));
        let e = refuse("G19", Ok(true), false).expect_err("uiAccess");
        assert!(e.contains("uiAccess") && e.contains("cargo run"), "{e}");
        let e = refuse("G23", Ok(false), true).expect_err("a copy runs");
        assert!(
            e.contains(&format!("close {NAME}")) && e.contains("G23"),
            "{e}"
        );
        let e = refuse("G19", Err("no token".into()), false).expect_err("unknown");
        assert!(e.contains("no token"), "{e}");
    }

    #[test]
    fn only_a_copy_owing_a_warning_has_its_log_read_and_it_must_hold_it() {
        let unread = || -> Result<String, String> { panic!("the log was read") };
        assert_eq!(heard(None, unread, 0), Ok(()));
        let quiet = heard(Some("gate input"), || Ok(String::new()), 0);
        assert!(quiet.is_err_and(|e| e.contains("no gate warning")));
        let gone = heard(Some("gate input"), || Err("no folder".into()), 0);
        assert_eq!(gone, Err("no folder".into()));
    }

    #[test]
    fn only_a_data_folder_the_gate_made_is_used() {
        let data = Path::new(r"D:\build\data");
        assert_eq!(ours("G23", data, true), Ok(()));
        let e = ours("G19", data, false).expect_err("someone else's folder");
        assert!(
            e.contains(r"D:\build\data") && e.contains("G19") && e.contains("remove"),
            "{e}"
        );
    }
}
