//! G8 budgets (spec §10): it starts the product and sets cold start, idle memory and CPU, folder size, key highlight,
//! click to character, a bus publish and `PresentMon`'s dropped frames against them, and times resizes.
//! `g8 weak` caps it as the spike's weak mode does (`spike.toml [voice.weak]`); it clicks, so only with the owner's yes.
#![cfg(windows)]

use std::path::Path;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde_json::{Value, json};
use spike_core::weak::{Caps, Job};
use windows::Win32::System::Threading::GetCurrentProcess;

use crate::apps::Ctx;
use crate::budget;
use crate::busbench::{self, BusTimes};
use crate::g8parts::{Measured, Outside, parts};
use crate::g8ui;
use crate::out::say;
use crate::presentmon;
use crate::proc::{self, Process};
use crate::product::{self, Keyboard};
use crate::win::{self, sleep_ms};
use crate::{admin, close, g2, out};

/// The name that runs G8 in weak mode.
pub const WEAK: &str = "weak";
/// Weak mode cannot force software drawing: the app clears every Qt variable before Qt starts (kx-ui `harden`).
const NO_SOFTWARE: &str =
    "not forced: the app clears Qt's variables, so weak mode caps cores, CPU and memory only";
/// The shell cannot start a program paused, so the caps reach it only once it runs.
const CAPS_LATE: &str = "from a few ms after start: the shell cannot start the product paused";

/// The product as G8 started it.
struct Started {
    /// Its process, opened to measure.
    process: Process,
    /// When it was started, in µs since 1970.
    at_us: i64,
    /// Start to its first shown window, in ms.
    shown_ms: Option<f64>,
}

/// Microseconds since 1970 now.
fn now_us() -> i64 {
    let since = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    i64::try_from(since.as_micros()).unwrap_or(i64::MAX)
}

/// The weak-mode job on a PC with `cpus` cores, and its cores and CPU share, from `spike.toml [voice.weak]`.
fn weak_job(ctx: &Ctx, cpus: usize) -> Result<(Job, (usize, u32)), String> {
    let w = &ctx.spike.voice.weak;
    let job = Job::new(Some(&Caps::from(w, cpus)?))?;
    Ok((
        job,
        (usize::try_from(w.cores).unwrap_or(usize::MAX), w.rate_pct),
    ))
}

/// Starts the product through the shell, in `job` when given, and waits for its first shown window.
/// A copy it cannot open or cap is asked to quit, so the next run still starts cold.
fn start(ctx: &Ctx, program: &Path, job: Option<&Job>) -> Result<Started, String> {
    let g = &ctx.cfg.g8;
    let (at_us, at) = (now_us(), Instant::now());
    let opened = admin::start_measured(program, &[]).and_then(|p| match job {
        Some(job) => job.assign_handle(p.handle()).map(|()| p),
        None => Ok(p),
    });
    let process = opened.inspect_err(|_| {
        let _ = close::quit_running(ctx);
    })?;
    let pid = process.pid();
    let shown = || {
        let ours = win::seen_windows()
            .into_iter()
            .any(|w| win::owner(w).0 == pid);
        ours.then(|| budget::ms(at.elapsed()))
    };
    let shown_ms = win::poll_until(g.start_wait_ms, g.look_ms, shown);
    Ok(Started {
        process,
        at_us,
        shown_ms,
    })
}

/// Start to the log's start line, and that line to the guard's, in ms; this run's lines only.
fn log_span(ctx: &Ctx, program: &Path, since_us: i64) -> Result<Option<(f64, f64)>, String> {
    let (g, file) = (&ctx.cfg.g8, product::files(program)?.log);
    let read = || {
        let text = std::fs::read_to_string(&file).ok()?;
        let (from, to) = budget::span_us(&text, &g.log_start, &g.log_guarded)?;
        (from >= since_us).then(|| {
            (
                budget::ms_between(since_us, from),
                budget::ms_between(from, to),
            )
        })
    };
    Ok(win::poll_until(
        g.start_wait_ms,
        ctx.cfg.timing.poll_ms,
        read,
    ))
}

/// Idle private memory in MiB, and idle CPU in percent of `cores` usable cores.
fn idle(ctx: &Ctx, p: &Process, cores: f64) -> Result<(f64, Option<f64>), String> {
    let g = &ctx.cfg.g8;
    sleep_ms(g.idle_settle_ms);
    let (cpu, at) = (p.cpu_ms()?, Instant::now());
    sleep_ms(g.idle_ms);
    let used = p.cpu_ms()? - cpu;
    let pct = budget::cpu_pct(used, budget::ms(at.elapsed()), cores);
    Ok((p.private_mb()?, pct))
}

/// Everything G8 measures on the started product, in order: start-up log, idle, folder, presses, resizes, G2.
fn measure(ctx: &Ctx, s: &Started, program: &Path, cores: f64) -> Result<Measured, String> {
    let g = &ctx.cfg.g8;
    let log = log_span(ctx, program, s.at_us)?;
    let kb = Keyboard::find()?;
    let (memory_mb, cpu_pct) = idle(ctx, &s.process, cores)?;
    let folder = program.parent().ok_or("the program has no folder")?;
    let size_mb = proc::folder_mb(folder)?;
    let presses = g8ui::presses(ctx, &kb)?;
    let resizes = g8ui::resizes(ctx, kb.hwnd)?;
    let g2 = g2::run(ctx, product::NAME, g.clicks, out::clock_seed())?;
    Ok(Measured {
        log,
        memory_mb,
        cpu_pct,
        size_mb,
        presses,
        resizes,
        g2,
    })
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

/// `PresentMon`'s frames for `program`, from a CSV written since `since`; else how to get one.
fn frames(ctx: &Ctx, program: &Path, since: SystemTime) -> Result<presentmon::Frames, String> {
    let (g, file) = (&ctx.cfg.g8, ctx.out.join(&ctx.cfg.g8.presentmon));
    let name = program
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let fresh = std::fs::metadata(&file)
        .and_then(|m| m.modified())
        .is_ok_and(|t| t >= since);
    if !fresh {
        return Err(format!(
            "no PresentMon CSV from this run; {}",
            presentmon::how(&name, &file, g.presentmon_secs)
        ));
    }
    // PresentMon, started just before G8, records `presentmon_secs`; read its CSV only once it has stopped.
    let end = since + Duration::from_secs(g.presentmon_secs);
    if let Ok(left) = end.duration_since(SystemTime::now()) {
        std::thread::sleep(left);
    }
    let csv = std::fs::read_to_string(&file).map_err(|e| format!("{}: {e}", file.display()))?;
    presentmon::parse(&csv, &name)
}

/// `r` for the result line: its value, or its error.
fn shown<T: serde::Serialize>(r: &Result<T, String>) -> Value {
    r.as_ref()
        .map_or_else(|e| json!({ "error": e }), |v| json!(v))
}

/// Times the bus, then reads `PresentMon`'s CSV and the refresh rate; with the bus and frames for the result line.
fn outside(
    ctx: &Ctx,
    job: Option<Job>,
    program: &Path,
    since: SystemTime,
) -> (Outside, [Value; 2]) {
    let (bus, frames) = (bench(ctx, job), frames(ctx, program, since));
    #[allow(
        clippy::cast_precision_loss,
        reason = "Frame counts are far below 2^53."
    )]
    let dropped = frames
        .as_ref()
        .ok()
        .and_then(presentmon::Frames::drops)
        .map(|d| d as f64);
    let bus_p99_us = bus.as_ref().ok().and_then(|b| b.p99_us);
    let o = Outside {
        hz: g8ui::refresh_hz(),
        bus_p99_us,
        dropped,
    };
    (o, [shown(&bus), shown(&frames)])
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
    let cores = std::thread::available_parallelism().map_or(1, usize::from);
    let (job, caps) = if weak {
        weak_job(ctx, cores).map(|(j, c)| (Some(j), Some(c)))?
    } else {
        (None, None)
    };
    let since = SystemTime::now();
    let s = start(ctx, &program, job.as_ref())?;
    say!(
        "G8 {name}: started {} (pid {}); shown after {:?} ms",
        program.display(),
        s.process.pid(),
        s.shown_ms
    );
    let measured = measure(ctx, &s, &program, budget::usable_cores(cores, caps));
    let quit = close::quit_running(ctx).map(|(_, ended)| ended);
    let m = measured?;
    let (o, [bus, frames]) = outside(ctx, job, &program, since);
    let parts = parts(&ctx.cfg.g8, s.shown_ms, &m, weak, &o);
    let ended = quit.unwrap_or(false);
    let (to_log, log_span) = m.log.unzip();
    Ok(json!({
        "gate": "G8", "keyboard": product::NAME, "weak": weak, "program": program,
        "renderer": weak.then_some(NO_SOFTWARE), "caps": weak.then_some(CAPS_LATE), "start_to_log_ms": to_log, "log_to_guarded_ms": log_span,
        "resizes": m.resizes, "presses": m.presses, "g2": m.g2, "bus": bus, "frames": frames,
        "parts": parts, "quit": ended, "pass": ended && budget::all_pass(&parts),
    }))
}
