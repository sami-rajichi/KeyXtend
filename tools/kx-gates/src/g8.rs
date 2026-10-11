//! G8 budgets (spec §10): it starts the product and sets cold start, idle memory and CPU, folder size, key highlight,
//! click to character, a bus publish, `PresentMon`'s dropped frames and memory growth under pop-up churn against
//! them, and times resizes.
//! `g8 weak` caps it as the spike's weak mode does (`spike.toml [voice.weak]`); it clicks, so only with the owner's yes.
#![cfg(windows)]

use std::path::Path;
use std::time::{Instant, SystemTime};

use serde_json::{Value, json};
use spike_core::weak::{Caps, Job};
use windows::Win32::System::Threading::GetCurrentProcess;

use crate::apps::Ctx;
use crate::budget;
use crate::busbench::{self, BusTimes};
use crate::g8churn;
use crate::g8frames::{self, Recorder};
use crate::g8parts::{Measured, Outside, parts, verdict};
use crate::g8start::{self, Started, log_span};
use crate::out::say;
use crate::presentmon::Frames;
use crate::proc::{self, Memory, Process};
use crate::product::{self, Keyboard};
use crate::win::sleep_ms;
use crate::{close, g2, mouseguard, out};
use crate::{g8size, g8ui};

/// The name that runs G8 in weak mode.
pub const WEAK: &str = "weak";
/// Weak mode cannot force software drawing: the app clears every Qt variable before Qt starts (kx-ui `harden`).
const NO_SOFTWARE: &str = "not forced, as the app clears Qt's variables: weak mode caps cores, CPU time and memory \
     only, so its best verdict is partial. The owner accepted that for P3, and software drawing moved to P14";
/// The shell cannot start a program paused, so the caps reach it only once it runs.
const CAPS_LATE: &str = "from a few ms after start: the shell cannot start the product paused";

/// The weak-mode job from `spike.toml [voice.weak]`, and the CPU time it allows in whole cores.
fn weak_job(ctx: &Ctx) -> Result<(Job, f64), String> {
    let cpus = std::thread::available_parallelism().map_or(1, usize::from);
    let caps = Caps::from(&ctx.spike.voice.weak, cpus)?;
    Ok((Job::new(Some(&caps))?, caps.cpu_cores(cpus)))
}

/// Private memory once `[g8] warm_ms` has passed since the keyboard showed, and the ms from show to the read.
fn warm(ctx: &Ctx, s: &Started) -> Result<(Memory, f64), String> {
    let from = s.shown_at.unwrap_or(s.at);
    std::thread::sleep(g8start::warm_left(from, ctx.cfg.g8.warm_ms, Instant::now()));
    let memory = s.process.memory()?;
    Ok((memory, budget::ms(from.elapsed())))
}

/// Idle CPU in percent of one core, after a settle.
fn idle(ctx: &Ctx, p: &Process) -> Result<Option<f64>, String> {
    let g = &ctx.cfg.g8;
    sleep_ms(g.idle_settle_ms);
    let (cpu, at) = (p.cpu_ms()?, Instant::now());
    sleep_ms(g.idle_ms);
    let used = p.cpu_ms()? - cpu;
    Ok(budget::core_pct(used, budget::ms(at.elapsed())))
}

/// Everything G8 measures on the started product, in order: start-up log, memory, idle, folder, presses, resizes and
/// G2; then `PresentMon`'s frames once its capture ends, and only then the churn, whose theme switches never count.
fn measure(
    ctx: &Ctx,
    s: &Started,
    program: &Path,
    (since, rec): (SystemTime, Option<&Recorder>),
) -> Result<(Measured, Result<Frames, String>), String> {
    let g = &ctx.cfg.g8;
    let log = log_span(ctx, program, s.at_us)?;
    let kb = Keyboard::find()?;
    let (memory, memory_at_ms) = warm(ctx, s)?;
    let core_pct = idle(ctx, &s.process)?;
    let folder = program.parent().ok_or("the program has no folder")?;
    let size_mb = proc::folder_mb(folder)?;
    let presses = g8ui::presses(ctx, &kb)?;
    let resizes = g8size::resizes(ctx, kb.hwnd)?;
    let g2 = g2::run(ctx, product::NAME, g.clicks, out::clock_seed())?;
    let capture_cut = g8frames::cut(ctx, since, rec);
    let frames = g8frames::frames(ctx, program, since, rec);
    let churn = g8churn::run(ctx, &s.process)?;
    let m = Measured {
        log,
        memory,
        memory_at_ms,
        core_pct,
        size_mb,
        presses,
        resizes,
        g2,
        capture_cut,
        churn,
    };
    Ok((m, frames))
}

/// Times the bus in kx-gates, after joining the weak job when given; the job then lives until kx-gates ends, as
/// closing it would end kx-gates with it.
#[allow(unsafe_code, reason = "Our own process's pseudo handle.")]
fn bench(ctx: &Ctx, job: Option<Job>) -> Result<BusTimes, String> {
    if let Some(job) = job {
        // SAFETY: a pseudo handle to our own process, which needs no closing.
        job.assign_handle(unsafe { GetCurrentProcess() })?;
        std::mem::forget(job);
    }
    busbench::time(ctx.cfg.g8.subscribers, ctx.cfg.g8.publishes)
}

/// `r` for the result line: its value, or its error.
fn shown<T: serde::Serialize>(r: &Result<T, String>) -> Value {
    r.as_ref()
        .map_or_else(|e| json!({ "error": e }), |v| json!(v))
}

/// Times the bus and reads the refresh rate; with `PresentMon`'s `frames`, what G8 measured outside the product, and
/// the bus and frames for the result line.
fn outside(ctx: &Ctx, job: Option<Job>, frames: &Result<Frames, String>) -> (Outside, [Value; 2]) {
    let bus = bench(ctx, job);
    #[allow(
        clippy::cast_precision_loss,
        reason = "Frame counts are far below 2^53."
    )]
    let dropped = frames
        .as_ref()
        .ok()
        .and_then(Frames::drops)
        .map(|d| d as f64);
    let bus_p99_us = bus.as_ref().ok().and_then(|b| b.p99_us);
    let o = Outside {
        hz: g8ui::refresh_hz(),
        bus_p99_us,
        dropped,
    };
    (o, [shown(&bus), shown(frames)])
}

/// Error `e` with what G8 leaves behind: the `PresentMon` it started, which records for `secs` s from its start,
/// and whether the product quit after `quit`.
fn broke(e: &str, recorder: Option<u32>, secs: u64, quit: Option<&Result<bool, String>>) -> String {
    let left = recorder.map(|pid| {
        format!("; PresentMon (pid {pid}) records for up to {secs} s from its start, then stops")
    });
    let product = match quit {
        Some(Ok(true)) => "; the product quit".to_string(),
        Some(Ok(false)) => "; the product still runs; close it".to_string(),
        Some(Err(q)) => format!("; the product still runs ({q}); close it"),
        None => String::new(),
    };
    format!("{e}{}{product}", left.unwrap_or_default())
}

/// Runs G8: `name` is the product, or `weak` for weak mode. The product must not be running, as G8 times a cold start.
pub fn run(ctx: &Ctx, name: &str) -> Result<Value, String> {
    if product::running().is_some() {
        return Err(format!(
            "close {} first: G8 times a cold start",
            product::NAME
        ));
    }
    let weak = name == WEAK;
    let program = ctx.cfg.program(&ctx.cfg.g8.program);
    let (job, cpu_cores) = if weak {
        weak_job(ctx).map(|(j, c)| (Some(j), Some(c)))?
    } else {
        (None, None)
    };
    let since = SystemTime::now();
    let rec = g8frames::record(ctx, &program)?;
    let (pm, secs) = (rec.as_ref().map(Recorder::pid), ctx.cfg.g8.presentmon_secs);
    let s = g8start::start(ctx, &program, job.as_ref()).map_err(|e| broke(&e, pm, secs, None))?;
    let shown_ms = s.shown_ms();
    say!(
        "G8 {name}: started {} (pid {}); shown after {shown_ms:?} ms",
        program.display(),
        s.process.pid(),
    );
    let measured = measure(ctx, &s, &program, (since, rec.as_ref()));
    let quit = close::quit_running(ctx).map(|(_, ended)| ended);
    // The pointer work is over, so the bus bench runs with the owner's mouse free.
    mouseguard::let_go();
    let (m, frames) = measured.map_err(|e| broke(&e, pm, secs, Some(&quit)))?;
    let (o, [bus, frames]) = outside(ctx, job, &frames);
    let parts = parts(&ctx.cfg.g8, shown_ms, &m, weak, &o);
    let ended = quit.unwrap_or(false);
    let (to_log, log_span) = m.log.unzip();
    Ok(json!({
        "gate": "G8", "keyboard": product::NAME, "weak": weak, "program": program, "cpu_cores": cpu_cores,
        "renderer": weak.then_some(NO_SOFTWARE), "caps": weak.then_some(CAPS_LATE), "start_to_log_ms": to_log,
        "log_to_guarded_ms": log_span, "memory_at_ms": m.memory_at_ms, "resizes": m.resizes, "presses": m.presses,
        "g2": m.g2, "churn": m.churn, "bus": bus, "frames": frames, "capture_cut": m.capture_cut, "parts": parts,
        "quit": ended,
        "pass": verdict(weak, &m.churn, ended && budget::all_pass(&parts)),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_run_that_broke_off_names_the_presentmon_it_left_and_whether_the_product_quit() {
        let e = broke("no keyboard", Some(4242), 90, Some(&Ok(false)));
        assert!(e.starts_with("no keyboard"), "{e}");
        assert!(
            e.contains("PresentMon (pid 4242)") && e.contains("90 s"),
            "{e}"
        );
        assert!(e.contains("still runs"), "{e}");
        let quit = broke("no keyboard", None, 90, Some(&Ok(true)));
        assert!(
            !quit.contains("PresentMon") && quit.contains("quit"),
            "{quit}"
        );
        let refused = broke("x", None, 90, Some(&Err("no command window".into())));
        assert!(refused.contains("no command window"), "{refused}");
        assert_eq!(broke("x", None, 90, None), "x", "nothing left to name");
    }
}
