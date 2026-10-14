//! G12 ring overlay (roadmap G12): with Right-click on in the product's test build, one press at each grid point of
//! target-window lasts long enough for the overlay's ring to play, yet stays a click. Every click must reach
//! target-window, and `PresentMon` must see the ring keep 60 Hz. It moves the pointer, so only with the owner's yes.
#![cfg(windows)]

use std::path::PathBuf;
use std::time::{Instant, SystemTime};

use serde_json::{Value, json};
use spike_core::clock::now_us;
use spike_core::hold::Pt;
use windows::Win32::Foundation::{HWND, POINT, RECT};

use crate::apps::{self, Ctx};
use crate::config::GatesConfig;
use crate::g5::{self, RCLICK};
use crate::g8frames::{self, Capture, Recorder};
use crate::g12frames::{self, Ring, Span};
use crate::g12pts::{Pace, grid, logged, overlay_at, pace};
use crate::out::say;
use crate::p4cfg::G12;
use crate::product::{self, Keyboard};
use crate::simin::{self, Act};
use crate::stats::US_PER_MS_INT;
use crate::testbuild::{Closed, TestBuild};
use crate::win::{self, sleep_ms};
use crate::{launch, mouse, mouseguard, probe, tlog};

/// G12's name in its result and in `PresentMon`'s how-to.
const GATE: &str = "G12";

/// Target-window's text box on screen: its client area, in physical pixels.
fn text_box(target: HWND) -> Result<RECT, String> {
    let f = mouse::face(target)?;
    let (o, (w, h)) = (f.origin, f.client);
    Ok(RECT {
        left: o.x,
        top: o.y,
        right: o.x + w,
        bottom: o.y + h,
    })
}

/// G12's capture: its CSV and length from `[g12]`, `PresentMon`'s program and waits from `[g8]`.
fn capture(cfg: &GatesConfig) -> Capture {
    Capture {
        gate: GATE,
        csv: cfg.g12.presentmon.clone(),
        secs: cfg.g12.presentmon_secs,
        ..Capture::of_g8(&cfg.g8)
    }
}

/// The test build's program, whose file name `PresentMon` records.
fn test_build(cfg: &GatesConfig) -> PathBuf {
    cfg.program(&cfg.gate_input.program)
}

/// The windows a press looks at: the test build's process and keyboard, and target-window.
struct Wins {
    pid: u32,
    keyboard: HWND,
    target: HWND,
}

impl Wins {
    /// The test build's, found by its shown keyboard, and `target`.
    fn find(target: HWND) -> Result<Self, String> {
        let keyboard = Keyboard::find()?.hwnd;
        let pid = win::owner(keyboard).0;
        Ok(Self {
            pid,
            keyboard,
            target,
        })
    }

    /// The overlay window over `p` now, if any.
    fn overlay(&self, p: POINT) -> Option<HWND> {
        let boxed = |w: HWND| win::rect(w).ok().map(|r| (w, r));
        let ours: Vec<(HWND, RECT)> = product::shown_by(self.pid)
            .into_iter()
            .filter_map(boxed)
            .collect();
        overlay_at(&win::top_windows(), &ours, self.keyboard, self.target, p)
    }
}

/// What one press found: its ring span, whether the overlay covered the point, and whether it took the click.
struct Pressed {
    span: Span,
    covered: bool,
    blocked: bool,
}

/// Presses at `p` as the simulated user for `pace.press_ms`, looking every `poll_ms` for the overlay over `p`, then
/// lets go; any window at `p` but target-window or the overlay stops the run before the press.
fn press(ctx: &Ctx, w: &Wins, p: POINT, pace: Pace) -> Result<Pressed, String> {
    let (at, poll_ms) = (Pt { x: p.x, y: p.y }, ctx.cfg.timing.poll_ms);
    simin::as_user(&[Act::Move(at)])?;
    sleep_ms(ctx.cfg.sim.step_ms);
    let (top, over) = (win::root_at(p), w.overlay(p));
    if top != w.target && Some(top) != over {
        return Err(format!("{} covers the test point", win::describe(top)));
    }
    let mut blocked = Some(top) == over;
    simin::as_user(&[Act::Down(at)])?;
    let (start, start_us) = (Instant::now(), now_us());
    let mut covered = false;
    while win::ms_since(start) < pace.press_ms {
        if let Some(o) = w.overlay(p) {
            covered = true;
            blocked |= win::root_at(p) == o;
        }
        sleep_ms(
            pace.press_ms
                .saturating_sub(win::ms_since(start))
                .min(poll_ms),
        );
    }
    simin::as_user(&[Act::Up(at)])?;
    let ring_us = i64::try_from(pace.ring_after_ms).unwrap_or(i64::MAX);
    let from_us = start_us.saturating_add(ring_us.saturating_mul(US_PER_MS_INT));
    let span = Span {
        from_us,
        to_us: now_us(),
    };
    sleep_ms(ctx.cfg.g12.settle_ms);
    Ok(Pressed {
        span,
        covered,
        blocked,
    })
}

/// What the presses found.
#[derive(Debug, Default, PartialEq)]
struct Tally {
    /// Points pressed, in order.
    clicked: Vec<POINT>,
    /// Their ring spans.
    spans: Vec<Span>,
    /// Presses the overlay covered.
    covered: usize,
    /// Presses where Windows put the overlay, not target-window, at the point.
    blocked: usize,
}

/// Presses each of `points` in turn.
fn click_all(ctx: &Ctx, w: &Wins, points: &[POINT], pace: Pace) -> Result<Tally, String> {
    let mut t = Tally::default();
    for &p in points {
        let got = press(ctx, w, p, pace)?;
        t.clicked.push(p);
        t.spans.push(got.span);
        t.covered += usize::from(got.covered);
        t.blocked += usize::from(got.blocked);
    }
    Ok(t)
}

/// What the run gave before the test build closed.
#[derive(Debug, PartialEq)]
struct Driven {
    tally: Tally,
    /// Clicks target-window logged.
    reached: usize,
    /// Grid points where Windows put another window than target-window.
    left_out: usize,
}

/// Parks target-window in front and spreads the grid over its text box; the points where Windows puts target-window,
/// how many were left out, and the windows a press looks at.
fn aim(ctx: &Ctx, target: HWND) -> Result<(Vec<POINT>, usize, Wins), String> {
    launch::park_target(target)?;
    if !win::front(target, &ctx.cfg.timing, &ctx.cfg.keys) {
        return Err("could not bring target-window to the front".to_string());
    }
    let g = &ctx.cfg.g12;
    let all = grid(text_box(target)?, g.grid, g.inset_px);
    let points: Vec<POINT> = all
        .iter()
        .copied()
        .filter(|&p| win::root_at(p) == target)
        .collect();
    let left_out = all.len() - points.len();
    Ok((points, left_out, Wins::find(target)?))
}

/// Aims, turns Right-click on, starts `PresentMon`, presses every point and turns Right-click off, even after a failed
/// press, then counts the clicks target-window logged; with the `PresentMon` it started, also on an error.
fn drive(
    ctx: &Ctx,
    build: &TestBuild,
    target: HWND,
    pace: Pace,
) -> (Option<Recorder>, Result<Driven, String>) {
    let aimed = aim(ctx, target).and_then(|a| build.set(RCLICK, true).map(|()| a));
    let (points, left_out, wins) = match aimed {
        Ok(a) => a,
        Err(e) => return (None, Err(e)),
    };
    let (rec, since) = match g8frames::record(ctx, &capture(ctx.cfg), &test_build(ctx.cfg)) {
        Ok(rec) => (rec, now_us()),
        Err(e) => return (None, g5::both(Err(e), build.set(RCLICK, false))),
    };
    let pressed = click_all(ctx, &wins, &points, pace);
    let counted = g5::both(pressed, build.set(RCLICK, false)).and_then(|tally| {
        let got = tlog::mouse_since(ctx, since)?;
        let reached = logged(&got, &tally.clicked, ctx.cfg.g12.slack_px);
        Ok(Driven {
            tally,
            reached,
            left_out,
        })
    });
    (rec, counted)
}

/// `r` for the result line: its value, or its error.
fn shown<T: serde::Serialize>(r: &Result<T, String>) -> Value {
    r.as_ref()
        .map_or_else(|e| json!({ "error": e }), |v| json!(v))
}

/// G12's result line. It passes when every click reached target-window under the overlay with nothing blocked or left
/// out, the ring kept `min_fps` and `p99_ms` over a capture that covered every span, and the test build quit.
fn result(g: &G12, d: &Driven, ring: &Result<Ring, String>, closed: &Closed) -> Value {
    let (t, n) = (&d.tally, d.tally.clicked.len());
    let all_hit = d.left_out == 0 && d.reached == n && t.covered == n && t.blocked == 0;
    let fast = |r: &Ring| r.fps.is_some_and(|f| f >= g.min_fps);
    let even = |r: &Ring| r.p99_ms.is_some_and(|p| p <= g.p99_ms);
    let smooth = ring.as_ref().is_ok_and(|r| fast(r) && even(r));
    json!({
        "gate": GATE, "keyboard": product::NAME, "clicks": n, "reached": d.reached, "covered": t.covered,
        "blocked": t.blocked, "left_out": d.left_out, "frames": shown(ring), "test_build": closed.json(),
        "pass": n > 0 && all_hit && smooth && closed.quit,
    })
}

/// The note for an error after G12 started `PresentMon` as `rec`, which records for `secs` s from its start.
fn still_recording(rec: Option<&Recorder>, secs: u64) -> String {
    let note = |r: &Recorder| {
        format!(
            "; PresentMon (pid {}) records for up to {secs} s from its start",
            r.pid()
        )
    };
    rec.map(note).unwrap_or_default()
}

/// Runs G12 on the product's test build: target-window opens, every grid point is pressed under the overlay, both
/// close, and `PresentMon`'s frames are judged once its capture ends, with the owner's mouse already free.
pub fn run(ctx: &Ctx, name: &str) -> Result<Value, String> {
    let (since, origin_us, cap) = (SystemTime::now(), now_us(), capture(ctx.cfg));
    let build = TestBuild::open(ctx, None)?;
    let pace = pace(&build.mouse()?, &ctx.cfg.g12, ctx.cfg.mouse_guard.alive_ms)?;
    let target = launch::start_target(ctx)?;
    say!("G12 {name}: {}", win::describe(target.hwnd));
    let (rec, driven) = drive(ctx, &build, target.hwnd, pace);
    let mut left_open = probe::released();
    left_open.extend(apps::clean_up(ctx, target));
    let closed = build.close();
    mouseguard::let_go();
    let pm = still_recording(rec.as_ref(), cap.secs);
    let d = driven.map_err(|e| closed.failed(&format!("{e}{pm}"), &left_open))?;
    let frames = g8frames::frames(ctx, &cap, &test_build(ctx.cfg), since, rec.as_ref());
    let end_us = g8frames::end_us(cap.secs, rec.as_ref(), origin_us);
    let spans = &d.tally.spans;
    let ring =
        g12frames::cut(spans, end_us).and_then(|()| g12frames::judge(&frames?.presents, spans));
    let mut v = result(&ctx.cfg.g12, &d, &ring, &closed);
    v["press_ms"] = json!(pace.press_ms);
    v["left_open"] = json!(left_open);
    say!("G12 {name}: pass {}", v["pass"]);
    Ok(v)
}

#[cfg(test)]
mod tests;
