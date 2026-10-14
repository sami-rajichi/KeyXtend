//! Tests of `g8frames`: G8's capture, its start, its wait for the stop, and the how-to for the gate's own CSV.

use std::process::{Child, Command, Stdio};
use std::time::Duration;

use kx_test_support::tempdir::TempDir;

use super::*;
use crate::config;

/// Two frames of the product in `PresentMon` 2.x columns with QPC times; the second never showed.
const CSV: &str = "Application,SwapChainAddress,CPUStartQPCTime,CPUBusy,FrameTime,DisplayedTime\n\
    keyxtend.exe,0x1,5000.0,1.0,6.06,6.06\nkeyxtend.exe,0x1,5006.06,1.0,6.06,NA\n";
/// The product's program, as G8 names it to `PresentMon`.
const PROGRAM: &str = r"C:\kx\keyxtend.exe";

/// G8's capture with no `PresentMon` to start and no recording time.
fn cap() -> Capture {
    let g8 = config::load().expect("settings load").g8;
    Capture {
        exe: String::new(),
        secs: 0,
        stop_ms: 0,
        ..Capture::of_g8(&g8)
    }
}

/// Runs `test` with the shipped settings and a fresh out folder.
fn with_ctx(test: impl FnOnce(&Ctx)) {
    let cfg = config::load().expect("settings load");
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
fn g8s_capture_is_all_g8s() {
    let g = config::load().expect("settings load").g8;
    let c = Capture::of_g8(&g);
    assert_eq!(
        (c.gate, c.csv.as_str(), c.exe.as_str()),
        (GATE, g.presentmon.as_str(), g.presentmon_exe.as_str())
    );
    let times = (c.secs, c.wait_ms, c.stop_ms);
    assert_eq!(
        times,
        (
            g.presentmon_secs,
            g.presentmon_wait_ms,
            g.presentmon_stop_ms
        )
    );
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
    with_ctx(|ctx| {
        let rec = record(ctx, &cap(), Path::new(PROGRAM)).expect("no error");
        assert!(rec.is_none());
    });
}

#[test]
fn a_recording_g8_started_that_wrote_no_csv_says_so() {
    with_ctx(|ctx| {
        let (_child, rec) = ended();
        let got = frames(ctx, &cap(), Path::new(PROGRAM), before_now(), Some(&rec));
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
        let got = frames(ctx, &cap(), Path::new(PROGRAM), before_now(), Some(&rec));
        assert!(
            got.expect_err("still runs")
                .contains("did not stop in time")
        );
    });
}

#[test]
fn with_no_recording_and_no_fresh_csv_it_says_how_to_make_the_gates_own_one() {
    with_ctx(|ctx| {
        let c = Capture {
            gate: "G12",
            csv: "g12-own.csv".into(),
            ..cap()
        };
        let e = frames(ctx, &c, Path::new(PROGRAM), before_now(), None).expect_err("no CSV");
        for part in ["presentmon_exe", "G12", "g12-own.csv"] {
            assert!(e.contains(part), "{part}: {e}");
        }
        std::fs::write(ctx.out.join(&c.csv), CSV).expect("an old CSV");
        let later = SystemTime::now() + Duration::from_secs(60);
        let got = frames(ctx, &c, Path::new(PROGRAM), later, None);
        assert!(
            got.expect_err("too old")
                .contains("no PresentMon CSV from this run")
        );
    });
}

#[test]
fn a_fresh_csv_is_read_once_the_recording_stopped() {
    with_ctx(|ctx| {
        let c = cap();
        std::fs::write(ctx.out.join(&c.csv), CSV).expect("a CSV");
        let (_child, rec) = ended();
        for r in [Some(&rec), None] {
            let f = frames(ctx, &c, Path::new(PROGRAM), before_now(), r).expect("read");
            assert_eq!((f.presented, f.lost), (2, 1));
        }
    });
}
