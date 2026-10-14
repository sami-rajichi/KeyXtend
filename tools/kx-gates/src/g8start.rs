//! G8's cold start: it starts the product through the shell, times it to its first shown window and its log's
//! start-up lines, and says when the warm-up before the memory sample ends.
#![cfg(windows)]

use std::path::Path;
use std::time::{Duration, Instant};

use spike_core::weak::Job;

use crate::apps::Ctx;
use crate::proc::Process;
use crate::{admin, budget, close, product, win};

/// The product as G8 started it.
pub struct Started {
    /// Its process, opened to measure.
    pub process: Process,
    /// When it was started, in µs since 1970.
    pub at_us: i64,
    /// When it was started.
    pub at: Instant,
    /// When its first window showed; none when none did in time.
    pub shown_at: Option<Instant>,
}

impl Started {
    /// Start to its first shown window, in ms.
    pub fn shown_ms(&self) -> Option<f64> {
        self.shown_at
            .map(|t| budget::ms(t.saturating_duration_since(self.at)))
    }
}

/// Asks the copy G8 just started, held as `process`, to quit; what is left of it, for the error.
fn quit_left(ctx: &Ctx, process: &Process) -> String {
    match close::quit_started(ctx, process) {
        Ok(true) => "it quit".to_string(),
        Ok(false) => "it still runs; close it".to_string(),
        Err(q) => format!("it still runs ({q}); close it"),
    }
}

/// Puts `process`, just started, in `job`; a copy it cannot cap is asked to quit, so no uncapped copy stays.
fn cap(ctx: &Ctx, job: &Job, process: &Process) -> Result<(), String> {
    let uncapped = |e| {
        format!(
            "{e}: the copy runs uncapped, so {}",
            quit_left(ctx, process)
        )
    };
    job.assign_handle(process.handle()).map_err(uncapped)
}

/// Starts the product through the shell with `args`, in `job` when given, and waits for its first shown window.
/// A copy it cannot open gets Quit with the same tries, so the next run still starts cold.
pub fn start(
    ctx: &Ctx,
    program: &Path,
    args: &[String],
    job: Option<&Job>,
) -> Result<Started, String> {
    let g = &ctx.cfg.g8;
    let (at_us, at) = (budget::now_us(), Instant::now());
    let given = admin::start_shown(program, args)?;
    // The shell's handle lacks the rights to measure, so the process is opened again.
    let unmeasured = |e| {
        format!(
            "{e}: G8 cannot measure the copy, so {}",
            quit_left(ctx, &given)
        )
    };
    let process = Process::to_measure(given.pid()).map_err(unmeasured)?;
    if let Some(job) = job {
        cap(ctx, job, &process)?;
    }
    let pid = process.pid();
    let shown = || {
        let ours = win::seen_windows()
            .into_iter()
            .any(|w| win::owner(w).0 == pid);
        ours.then(Instant::now)
    };
    let shown_at = win::poll_until(g.start_wait_ms, g.look_ms, shown);
    Ok(Started {
        process,
        at_us,
        at,
        shown_at,
    })
}

/// Start to the log's start line, and that line to the guard's, in ms; this run's lines only.
pub fn log_span(ctx: &Ctx, program: &Path, since_us: i64) -> Result<Option<(f64, f64)>, String> {
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

/// The wait left at `now` until `warm_ms` past `from`; nothing once that has passed.
pub fn warm_left(from: Instant, warm_ms: u64, now: Instant) -> Duration {
    (from + Duration::from_millis(warm_ms)).saturating_duration_since(now)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A sample warm-up, in ms.
    const WARM_MS: u64 = 10_000;

    #[test]
    fn the_memory_sample_waits_out_the_warm_up_from_the_shown_keyboard() {
        let shown = Instant::now();
        let after = |ms| shown + Duration::from_millis(ms);
        assert_eq!(
            warm_left(shown, WARM_MS, shown),
            Duration::from_millis(WARM_MS)
        );
        assert_eq!(
            warm_left(shown, WARM_MS, after(4_000)),
            Duration::from_millis(6_000)
        );
        assert_eq!(
            warm_left(shown, WARM_MS, after(12_000)),
            Duration::ZERO,
            "a late look reads at once"
        );
    }
}
