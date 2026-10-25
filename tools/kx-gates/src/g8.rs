//! G8 budgets (spec §10): it starts the product and sets cold start, idle memory and CPU, key highlight, click to
//! character, a bus publish, `PresentMon`'s dropped frames, memory growth under pop-up churn and memory after
//! use and after a scroll (`g8scroll`) against them, times resizes and logs the machine's memory (`g8host`).
//! `g8 weak` caps it as the spike's weak mode does (`spike.toml [voice.weak]`), and `g8 arabic` runs it with an Arabic
//! layout in front (`g8lang`); it clicks, so only with the owner's yes.
//! `g8 hook` and `g8 hook-weak` judge the hook's p99 on the test build instead (`g8hook`), and `g8 predict` the
//! prediction part's memory and typing churn (`g8predict`).
#![cfg(windows)]

use std::path::Path;
use std::time::{Instant, SystemTime};

use kx_platform_windows::WinLayouts;
use serde_json::{Value, json};
use spike_core::clock::now_us;
use spike_core::weak::{Caps, Job};
use windows::Win32::System::Threading::GetCurrentProcess;

use crate::apps::{Ctx, Opened};
use crate::budget;
use crate::busbench::{self, BusTimes};
use crate::g8frames::{self, Capture, Recorder};
use crate::g8lang::{self, Free, Holds, WinTarget};
use crate::g8loss::{self, Losses, Span};
use crate::g8parts::{Measured, Outside, met, parts, verdict};
use crate::g8rounds::{self, WinSteps};
use crate::g8start::{self, Started, broke, log_span};
use crate::g8type::Expect;
use crate::g8ui::Spot;

use crate::out::{say, shown};
use crate::presentmon::Frames;
use crate::proc::{Memory, Process};
use crate::product::{self, Keyboard};
use crate::win::sleep_ms;
use crate::{close, g2, g2raw, g8hook, g8host, mouseguard, out};
use crate::{g8lit, g8predict, g8size, g8type, g8ui, g8use};

/// The name that runs G8 in weak mode.
pub const WEAK: &str = "weak";
/// Weak mode cannot force software drawing: the app clears every Qt variable before Qt starts (kx-ui `harden`).
const NO_SOFTWARE: &str = "not forced, as the app clears Qt's variables: weak mode caps cores, CPU time and memory \
     only, so its best verdict is partial. The owner accepted that for P3, and software drawing moved to P14";
/// The shell cannot start a program paused, so the caps reach it only once it runs.
const CAPS_LATE: &str = "from a few ms after start: the shell cannot start the product paused";

/// In `weak` mode, the job from `spike.toml [voice.weak]` and the CPU time it allows in whole cores; else neither.
pub(crate) fn weak_job(ctx: &Ctx, weak: bool) -> Result<(Option<Job>, Option<f64>), String> {
    if !weak {
        return Ok((None, None));
    }
    let cpus = std::thread::available_parallelism().map_or(1, usize::from);
    let caps = Caps::from(&ctx.spike.voice.weak, cpus)?;
    Ok((Some(Job::new(Some(&caps))?), Some(caps.cpu_cores(cpus))))
}

/// Refuses a run while the product is running, as G8 times a cold start.
fn cold() -> Result<(), String> {
    match product::running() {
        Some(_) => Err(format!(
            "close {} first: G8 times a cold start",
            product::NAME
        )),
        None => Ok(()),
    }
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

/// Everything G8 measures on the started product once its holds fit the running repeat delay, in order: start-up log,
/// memory, idle, presses, resizes, G2, the memory after use and after a scroll, before any theme round (owner,
/// 2026-10-16); then `PresentMon`'s frames once its capture ends, and only then the churn, whose theme switches never
/// count (`g8rounds`). `hold` runs after the presses, after G2 and after the scroll, each of which took the front.
fn measure(
    ctx: &Ctx,
    s: &Started,
    (program, hold): (&Path, Hold),
    (since, rec): (SystemTime, Option<&Recorder>),
) -> Result<(Measured, Result<Frames, String>), String> {
    let g = &ctx.cfg.g8;
    g8type::check_running(ctx)?;
    let log = log_span(ctx, program, s.at_us)?;
    let kb = Keyboard::find()?;
    let (memory, memory_at_ms) = warm(ctx, s)?;
    let core_pct = idle(ctx, &s.process)?;
    let spot = Spot::of(&kb, &g.press_key)?;
    let presses = g8type::presses(ctx, &spot, Expect::Key(&g.press_key))?;
    hold.fresh()?;
    let resizes = g8size::resizes(ctx, kb.hwnd)?;
    let g2 = g2::run(ctx, product::NAME, g.clicks, out::clock_seed())?;
    let ran_us = now_us();
    hold.fresh()?;
    let used = g8use::run(ctx, &s.process);
    let frames = || g8frames::frames(ctx, &Capture::of_g8(g), program, since, rec);
    let r = g8rounds::run(&WinSteps::of(ctx, &s.process, hold), frames)?;
    let m = Measured {
        log,
        memory,
        memory_at_ms,
        core_pct,
        presses,
        resizes,
        g2,
        ran_us,
        churn: r.churn?,
        used,
        scrolled: r.scrolled,
        scroll_left: r.left,
    };
    Ok((m, r.between))
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

/// The losses in `PresentMon`'s `frames` against G8's `spans`, on a screen of `hz`, with times from `origin_us`; and
/// how the capture ending at `end_us` met a run whose G2 ended at `ran_us`, as a cut press window fails the losses.
fn judged(
    spans: Result<Vec<Span>, String>,
    ran_us: i64,
    frames: &Result<Frames, String>,
    hz: Option<f64>,
    (origin_us, end_us): (i64, i64),
) -> (Result<Losses, String>, Result<Option<&'static str>, String>) {
    let cut = spans
        .as_ref()
        .map_err(Clone::clone)
        .and_then(|s| g8loss::cut(s, end_us, ran_us));
    let losses = frames.as_ref().map_err(Clone::clone).and_then(|f| {
        cut.clone()?;
        g8loss::losses(&f.presents, &spans?, hz, origin_us)
    });
    (losses, cut)
}

/// Times the bus and reads the refresh rate; with `PresentMon`'s `frames` judged against the steps in `m` at `times`
/// (G8's start and the capture's end), what G8 measured outside the product, and the bus, frames, losses and the
/// capture's cut for the result line.
fn outside(
    ctx: &Ctx,
    job: Option<Job>,
    m: &Measured,
    frames: &Result<Frames, String>,
    times: (i64, i64),
) -> (Outside, [Value; 4]) {
    let (bus, hz) = (bench(ctx, job), g8ui::refresh_hz());
    let (losses, cut) = judged(spans(ctx, m), m.ran_us, frames, hz, times);
    #[allow(
        clippy::cast_precision_loss,
        reason = "Frame counts are far below 2^53."
    )]
    let dropped = losses.as_ref().ok().map(|l| l.counted.len() as f64);
    let bus_p99_us = bus.as_ref().ok().and_then(|b| b.p99_us);
    let o = Outside {
        hz,
        bus_p99_us,
        dropped,
    };
    (o, [shown(&bus), shown(frames), shown(&losses), shown(&cut)])
}

/// G8's presses, G2's clicks from the click file G2 wrote with `ctx`, and the resizes in `m`, for G8's frame rule.
fn spans(ctx: &Ctx, m: &Measured) -> Result<Vec<Span>, String> {
    let clicks = g2raw::read(&ctx.file(g2raw::CLICKS_FILE))?;
    let clicked = g8loss::clicks(&clicks, ctx.cfg.g8.rise_ms);
    let resized = m.resizes.iter().map(|r| r.span);
    let pressed = m.presses.spans.iter().copied();
    Ok(pressed.chain(clicked).chain(resized).collect())
}

/// Runs G8: `name` is the product, `weak` for weak mode or `arabic` with an Arabic layout in front, and the product
/// must not be running, as G8 times a cold start; `lit` and `osk` time only the key light (`g8lit`), `hook` and
/// `hook-weak` only the hook (`g8hook`), and `predict` only the prediction part (`g8predict`).
pub fn run(ctx: &Ctx, name: &str) -> Result<Value, String> {
    let parts = g8lit::run(ctx, name).or_else(|| g8hook::run(ctx, name));
    if let Some(part) = parts.or_else(|| g8predict::run(ctx, name)) {
        return part;
    }
    cold()?;
    match g8lang::wanted(&WinLayouts, &ctx.cfg.g8, name)? {
        Some(to) => g8lang::within(&WinTarget::of(ctx), to, |hold| started(ctx, name, hold)),
        None => started(ctx, name, &Free),
    }
}

/// What G8 runs after each step that took the front: `g8 arabic` brings its own window back there.
type Hold<'a> = &'a dyn Holds<Opened>;

/// G8 from its cold start to its result line, in weak mode for `weak`, running `hold` as `measure` says.
fn started(ctx: &Ctx, name: &str, hold: Hold) -> Result<Value, String> {
    let (weak, host) = (name == WEAK, g8host::read(ctx));
    say!("G8 {name}: the machine before: {}", json!(host));
    let program = ctx.cfg.program(&ctx.cfg.g8.program);
    let (job, cpu_cores) = weak_job(ctx, weak)?;
    let (since, origin_us) = (SystemTime::now(), now_us());
    let rec = g8frames::record(ctx, &Capture::of_g8(&ctx.cfg.g8), &program)?;
    let (pm, secs) = (rec.as_ref().map(Recorder::pid), ctx.cfg.g8.presentmon_secs);
    let s =
        g8start::start(ctx, &program, &[], job.as_ref()).map_err(|e| broke(&e, pm, secs, None))?;
    let shown_ms = s.shown_ms();
    say!(
        "G8 {name}: started {} (pid {}); shown after {shown_ms:?} ms",
        program.display(),
        s.process.pid(),
    );
    let measured = measure(ctx, &s, (&program, hold), (since, rec.as_ref()));
    let quit = close::quit_running(ctx)
        .map(|(_, ended)| ended)
        .map_err(|e| e.why);
    // The pointer work is over, so the bus bench runs with the owner's mouse free.
    mouseguard::let_go();
    let (m, frames) = measured.map_err(|e| broke(&e, pm, secs, Some(&quit)))?;
    let end_us = g8frames::end_us(secs, rec.as_ref(), origin_us);
    let (o, [bus, frames, losses, cut]) = outside(ctx, job, &m, &frames, (origin_us, end_us));
    let parts = parts(&ctx.cfg.g8, shown_ms, &m, weak, &o);
    let ended = quit.unwrap_or(false);
    let (to_log, log_span) = m.log.unzip();
    let mut v = json!({
        "gate": "G8", "keyboard": product::NAME, "weak": weak, "program": program, "cpu_cores": cpu_cores,
        "renderer": weak.then_some(NO_SOFTWARE), "caps": weak.then_some(CAPS_LATE), "start_to_log_ms": to_log,
        "log_to_guarded_ms": log_span, "memory_at_ms": m.memory_at_ms, "resizes": m.resizes, "presses": m.presses,
        "g2": m.g2, "churn": m.churn, "bus": bus, "frames": frames, "losses": losses, "capture_cut": cut,
        "scroll": shown(&m.scrolled), "scroll_left_open": m.scroll_left, "parts": parts, "quit": ended,
        "host": { "before": host, "after": g8host::read(ctx) },
        "pass": verdict(weak, &m.churn, met(ended, &m.presses, &parts)),
    });
    v[g8use::FIELD] = shown(&m.used);
    Ok(v)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::presentmon::Present;

    #[test]
    fn a_capture_cut_inside_a_press_window_fails_the_losses_and_one_after_it_only_notes() {
        let (at_us, rise_ms, ran_us) = (1_000_000, 200, 5_000_000);
        let spans = vec![Span::press(at_us, at_us, rise_ms)];
        let shown = Present {
            chain: "0x1".into(),
            at_us: at_us + 1,
            shown: true,
        };
        let frames = Ok(Frames {
            presented: 1,
            lost: 0,
            p99_ms: None,
            unread: 0,
            presents: vec![shown],
        });
        let press_end = at_us + 200_000;
        let hz = Some(60.0);
        let (losses, cut) = judged(Ok(spans.clone()), ran_us, &frames, hz, (0, press_end - 1));
        let e = losses.expect_err("it fails closed");
        assert!(e.contains("presentmon_secs") && cut.is_err(), "{e}");
        let (losses, cut) = judged(Ok(spans), ran_us, &frames, hz, (0, press_end));
        assert!(losses.is_ok_and(|l| l.counted.is_empty()));
        assert!(cut.expect("judged").is_some(), "a note only");
    }
}
