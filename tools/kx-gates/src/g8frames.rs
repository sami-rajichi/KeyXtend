//! G8's frame record: it starts `PresentMon` when `[g8] presentmon_exe` names it and reads its CSV once the recording
//! stopped; otherwise it reads the owner's recording, or says how to make one.
#![cfg(windows)]

use std::path::Path;
use std::time::{Instant, SystemTime};

use kx_module_api::US_PER_S;
use spike_core::clock::now_us;

use crate::admin;
use crate::apps::Ctx;
use crate::presentmon::{self, Frames};
use crate::proc::Process;
use crate::win::{self, sleep_ms};

/// `PresentMon` as G8 started it: its process, and when it started, also on G8's QPC clock in µs.
pub struct Recorder {
    process: Process,
    at: Instant,
    at_us: i64,
}

impl Recorder {
    /// The process id of the `PresentMon` G8 started.
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

/// Starts `[g8] presentmon_exe`, when named, as administrator to record `program`'s frames; none when not named.
pub fn record(ctx: &Ctx, program: &Path) -> Result<Option<Recorder>, String> {
    let g = &ctx.cfg.g8;
    if g.presentmon_exe.is_empty() {
        return Ok(None);
    }
    let file = ctx.out.join(&g.presentmon);
    let args = presentmon::args(&exe_name(program), &file, g.presentmon_secs);
    let process = admin::run_as_hidden(&ctx.cfg.program(&g.presentmon_exe), &args)?;
    let (at, at_us) = (Instant::now(), now_us());
    sleep_ms(g.presentmon_wait_ms);
    Ok(Some(Recorder { process, at, at_us }))
}

/// Waits for the `PresentMon` G8 started to stop, at most its recording time plus `[g8] presentmon_stop_ms`.
fn stopped(ctx: &Ctx, r: &Recorder) -> Result<(), String> {
    let g = &ctx.cfg.g8;
    let left = presentmon::stop_wait(g.presentmon_secs, g.presentmon_stop_ms, r.at.elapsed());
    let left_ms = u64::try_from(left.as_millis()).unwrap_or(u64::MAX);
    let ended = win::poll_until(left_ms, ctx.cfg.timing.poll_ms, || {
        r.process.ended().then_some(())
    });
    ended.ok_or_else(|| format!("PresentMon (pid {}) did not stop in time", r.process.pid()))
}

/// Waits for the recording the owner started just before G8, which ends at most this long after `since`.
fn owner_done(ctx: &Ctx, since: SystemTime) {
    let g = &ctx.cfg.g8;
    let gone = SystemTime::now().duration_since(since).unwrap_or_default();
    let left = presentmon::stop_wait(g.presentmon_secs, g.presentmon_stop_ms, gone);
    if !left.is_zero() {
        std::thread::sleep(left);
    }
}

/// When a capture of `secs` s ends on G8's QPC clock, in µs: counted from when G8 started `rec`, else from G8's
/// start at `origin_us` for the owner's capture, started just before.
pub fn end_us(secs: u64, rec: Option<&Recorder>, origin_us: i64) -> i64 {
    let from = rec.map_or(origin_us, |r| r.at_us);
    let secs = i64::try_from(secs).unwrap_or(i64::MAX);
    from.saturating_add(secs.saturating_mul(i64::from(US_PER_S)))
}

/// `program`'s frames from this run's CSV, written since `since`, read once the recording stopped; else why not.
pub fn frames(
    ctx: &Ctx,
    program: &Path,
    since: SystemTime,
    rec: Option<&Recorder>,
) -> Result<Frames, String> {
    let (g, file) = (&ctx.cfg.g8, ctx.out.join(&ctx.cfg.g8.presentmon));
    let name = exe_name(program);
    if let Some(r) = rec {
        stopped(ctx, r)?;
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
        let how = presentmon::how(&name, &file, g.presentmon_secs);
        return Err(format!("no PresentMon CSV from this run; {how}"));
    }
    if rec.is_none() {
        owner_done(ctx, since);
    }
    let csv = std::fs::read_to_string(&file).map_err(|e| format!("{}: {e}", file.display()))?;
    presentmon::parse(&csv, &name)
}

#[cfg(test)]
mod tests {
    use std::process::{Child, Command, Stdio};
    use std::time::Duration;

    use kx_test_support::tempdir::TempDir;

    use super::*;
    use crate::config::{self, GatesConfig};

    /// Two frames of the product in `PresentMon` 2.x columns with QPC times; the second never showed.
    const CSV: &str = "Application,SwapChainAddress,CPUStartQPCTime,CPUBusy,FrameTime,DisplayedTime\n\
        keyxtend.exe,0x1,5000.0,1.0,6.06,6.06\nkeyxtend.exe,0x1,5006.06,1.0,6.06,NA\n";
    /// The product's program, as G8 names it to `PresentMon`.
    const PROGRAM: &str = r"C:\kx\keyxtend.exe";

    /// Runs `test` with the shipped settings, no `PresentMon` to start, no recording time and a fresh out folder.
    fn with_ctx(test: impl FnOnce(&Ctx)) {
        let mut cfg: GatesConfig = config::load().expect("settings load");
        cfg.g8.presentmon_exe.clear();
        (cfg.g8.presentmon_secs, cfg.g8.presentmon_stop_ms) = (0, 0);
        let spike = spike_core::config::load().expect("spike settings");
        let target = kx_target_window::config::load().expect("target settings");
        let dir = TempDir::new("gates-frames").expect("temp folder");
        let out = dir.path().to_path_buf();
        let base = "t".to_string();
        let (cfg, spike, target) = (&cfg, &spike, &target);
        test(&Ctx {
            cfg,
            spike,
            target,
            out,
            base,
        });
    }

    /// A recorder whose process already ended, as a stopped `PresentMon`'s; the child keeps its id valid.
    fn ended() -> (Child, Recorder) {
        let exe = std::env::current_exe().expect("the test program");
        let mut child = Command::new(exe)
            .arg("--list")
            .stdout(Stdio::null())
            .spawn()
            .expect("starts");
        child.wait().expect("ends");
        let process = Process::to_wait(child.id()).expect("opens");
        (
            child,
            Recorder {
                process,
                at: Instant::now(),
                at_us: now_us(),
            },
        )
    }

    /// Ten seconds before now, so a CSV written now counts as this run's.
    fn before_now() -> SystemTime {
        SystemTime::now() - Duration::from_secs(10)
    }

    #[test]
    fn the_capture_ends_its_seconds_after_g8_started_presentmon_or_else_after_g8_began() {
        let (_child, rec) = ended();
        let secs_us = 90 * i64::from(US_PER_S);
        assert_eq!(end_us(90, Some(&rec), 0), rec.at_us + secs_us);
        assert_eq!(end_us(90, None, 5), 5 + secs_us, "the owner's capture");
    }

    #[test]
    fn with_no_presentmon_named_g8_records_nothing() {
        with_ctx(|ctx| assert!(record(ctx, Path::new(PROGRAM)).expect("no error").is_none()));
    }

    #[test]
    fn a_recording_g8_started_that_wrote_no_csv_says_so() {
        with_ctx(|ctx| {
            let (_child, rec) = ended();
            let got = frames(ctx, Path::new(PROGRAM), before_now(), Some(&rec));
            assert!(
                got.expect_err("no CSV")
                    .contains("PresentMon ran but wrote no CSV")
            );
        });
    }

    #[test]
    fn a_recording_that_never_stops_is_an_error() {
        with_ctx(|ctx| {
            let process = Process::to_wait(std::process::id()).expect("our own process");
            let rec = Recorder {
                process,
                at: Instant::now(),
                at_us: now_us(),
            };
            let got = frames(ctx, Path::new(PROGRAM), before_now(), Some(&rec));
            assert!(
                got.expect_err("still runs")
                    .contains("did not stop in time")
            );
        });
    }

    #[test]
    fn with_no_recording_and_no_fresh_csv_it_says_how_to_make_one() {
        with_ctx(|ctx| {
            let got = frames(ctx, Path::new(PROGRAM), before_now(), None);
            assert!(got.expect_err("no CSV").contains("presentmon_exe"));
            std::fs::write(ctx.out.join(&ctx.cfg.g8.presentmon), CSV).expect("an old CSV");
            let later = SystemTime::now() + Duration::from_secs(60);
            let got = frames(ctx, Path::new(PROGRAM), later, None);
            assert!(
                got.expect_err("too old")
                    .contains("no PresentMon CSV from this run")
            );
        });
    }

    #[test]
    fn a_fresh_csv_is_read_once_the_recording_stopped() {
        with_ctx(|ctx| {
            std::fs::write(ctx.out.join(&ctx.cfg.g8.presentmon), CSV).expect("a CSV");
            let (_child, rec) = ended();
            for r in [Some(&rec), None] {
                let f = frames(ctx, Path::new(PROGRAM), before_now(), r).expect("read");
                assert_eq!((f.presented, f.lost), (2, 1));
            }
        });
    }
}
