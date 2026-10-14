//! The frame record G8 and G12 share: a gate starts `PresentMon` when `[g8] presentmon_exe` names it and reads its CSV
//! once the recording stopped; otherwise it reads the owner's recording, or says how to make one.
#![cfg(windows)]

use std::path::Path;
use std::time::{Instant, SystemTime};

use kx_module_api::US_PER_S;
use spike_core::clock::now_us;

use crate::admin;
use crate::apps::Ctx;
use crate::g8cfg::G8;
use crate::presentmon::{self, Frames};
use crate::proc::Process;
use crate::win::{self, sleep_ms};

/// G8's name in the how-to.
pub const GATE: &str = "G8";

/// One gate's `PresentMon` capture; one `PresentMon` serves the PC, so its program and waits are always `[g8]`'s.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Capture {
    /// The gate, as the how-to names it.
    pub gate: &'static str,
    /// The `PresentMon` program to start as administrator; empty when the owner starts it.
    pub exe: String,
    /// The gate's CSV, in the out folder.
    pub csv: String,
    /// How long it records, in seconds.
    pub secs: u64,
    /// Wait after its start, in ms.
    pub wait_ms: u64,
    /// Longest wait past `secs` for it to stop and write its last rows, in ms.
    pub stop_ms: u64,
}

impl Capture {
    /// G8's capture, all from `[g8]`.
    pub fn of_g8(g: &G8) -> Self {
        Self {
            gate: GATE,
            exe: g.presentmon_exe.clone(),
            csv: g.presentmon.clone(),
            secs: g.presentmon_secs,
            wait_ms: g.presentmon_wait_ms,
            stop_ms: g.presentmon_stop_ms,
        }
    }
}

/// `PresentMon` as a gate started it: its process, and when it started, also on the gate's QPC clock in µs.
pub struct Recorder {
    process: Process,
    at: Instant,
    at_us: i64,
}

impl Recorder {
    /// The process id of the `PresentMon` the gate started.
    pub fn pid(&self) -> u32 {
        self.process.pid()
    }
}

/// The file name of `program`, which `PresentMon` matches its process by.
fn exe_name(program: &Path) -> String {
    program
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// Starts `cap`'s `PresentMon`, when named, as administrator to record `program`'s frames; none when not named.
pub fn record(ctx: &Ctx, cap: &Capture, program: &Path) -> Result<Option<Recorder>, String> {
    if cap.exe.is_empty() {
        return Ok(None);
    }
    let file = ctx.out.join(&cap.csv);
    let args = presentmon::args(&exe_name(program), &file, cap.secs);
    let process = admin::run_as_hidden(&ctx.cfg.program(&cap.exe), &args)?;
    let (at, at_us) = (Instant::now(), now_us());
    sleep_ms(cap.wait_ms);
    Ok(Some(Recorder { process, at, at_us }))
}

/// Waits for the `PresentMon` the gate started to stop, at most its recording time plus `cap.stop_ms`.
fn stopped(ctx: &Ctx, cap: &Capture, r: &Recorder) -> Result<(), String> {
    let left = presentmon::stop_wait(cap.secs, cap.stop_ms, r.at.elapsed());
    let left_ms = u64::try_from(left.as_millis()).unwrap_or(u64::MAX);
    let ended = win::poll_until(left_ms, ctx.cfg.timing.poll_ms, || {
        r.process.ended().then_some(())
    });
    ended.ok_or_else(|| format!("PresentMon (pid {}) did not stop in time", r.process.pid()))
}

/// Waits for the recording the owner started just before the gate, which ends at most this long after `since`.
fn owner_done(cap: &Capture, since: SystemTime) {
    let gone = SystemTime::now().duration_since(since).unwrap_or_default();
    let left = presentmon::stop_wait(cap.secs, cap.stop_ms, gone);
    if !left.is_zero() {
        std::thread::sleep(left);
    }
}

/// When a capture of `secs` s ends on the gate's QPC clock, in µs: counted from when the gate started `rec`, else
/// from the gate's start at `origin_us` for the owner's capture, started just before.
pub fn end_us(secs: u64, rec: Option<&Recorder>, origin_us: i64) -> i64 {
    let from = rec.map_or(origin_us, |r| r.at_us);
    let secs = i64::try_from(secs).unwrap_or(i64::MAX);
    from.saturating_add(secs.saturating_mul(i64::from(US_PER_S)))
}

/// `program`'s frames from `cap`'s CSV, written since `since`, read once the recording stopped; else why not.
pub fn frames(
    ctx: &Ctx,
    cap: &Capture,
    program: &Path,
    since: SystemTime,
    rec: Option<&Recorder>,
) -> Result<Frames, String> {
    let (file, name) = (ctx.out.join(&cap.csv), exe_name(program));
    if let Some(r) = rec {
        stopped(ctx, cap, r)?;
    }
    let fresh = std::fs::metadata(&file)
        .and_then(|m| m.modified())
        .is_ok_and(|t| t >= since);
    if !fresh && rec.is_some() {
        return Err(format!(
            "PresentMon ran but wrote no CSV to {}",
            file.display()
        ));
    }
    if !fresh {
        let how = presentmon::how(cap.gate, &name, &file, cap.secs);
        return Err(format!("no PresentMon CSV from this run; {how}"));
    }
    if rec.is_none() {
        owner_done(cap, since);
    }
    let csv = std::fs::read_to_string(&file).map_err(|e| format!("{}: {e}", file.display()))?;
    presentmon::parse(&csv, &name)
}

#[cfg(test)]
mod tests;
