//! G5 hold engine on the product's test build: with Right-click on, short clicks and drags stay normal, a still hold
//! right-clicks, even into a hung target, and the hook never times out or loses its echoes. Plain targets only; admin
//! windows are manual.
#![cfg(windows)]

use kx_mod_keyboard::{Assist, Role};
use kx_mod_mouse::MouseSettings;
use kx_platform_windows::screen;
use serde_json::{Value, json};
use spike_core::geom;

use crate::apps::{self, Ctx, Opened};
use crate::g5play::{self, Hold, Played};
use crate::hooklog::{self, HookLine};
use crate::launch;
use crate::out::say;
use crate::p4cfg::G5;
use crate::probe;
use crate::testbuild::{Closed, TestBuild};
use crate::win;

/// G5 targets: target-window as is.
pub const PLAIN: &str = "plain";
/// The admin-window case, which `g5` only explains.
pub const ADMIN: &str = "admin";
/// Why the admin-window case is the owner's manual check: the test build lacks uiAccess, so Windows keeps its input out
/// of admin windows. `g5 admin` says it and every `plain` result carries it.
pub const ADMIN_NOTE: &str = "The admin-window case is the owner's manual check on the installed product: it needs \
                              uiAccess, and the test build is never staged, signed or installed, so Windows blocks \
                              its input into admin windows.";
/// The key that turns Right-click on and off.
pub(crate) const RCLICK: Role = Role::Assist(Assist::RClick);

/// Refuses every name but `plain`; `admin` gives the reason it is the owner's manual check.
fn target_of(name: &str) -> Result<(), String> {
    match name {
        PLAIN => Ok(()),
        ADMIN => Err(ADMIN_NOTE.to_string()),
        _ => Err(format!("G5 runs on {PLAIN}, not {name}")),
    }
}

/// Refuses a hold the mouse guard would take for a lost hook, a hang past target-window's `hang_max_ms`, a hang lead
/// at or past the hold time, and a drag that stays within the still radius and its margin.
fn refuse(hold: Hold, alive_ms: u64, g: &G5, hang_max_ms: u64) -> Result<(), String> {
    let hold_ms = hold.ms;
    if hold_ms >= alive_ms {
        return Err(format!(
            "the test build's hold_ms ({hold_ms} ms) must stay below [mouse_guard] alive_ms ({alive_ms} ms): its hook \
             holds the press back, so the guard sees nothing until the right-click"
        ));
    }
    if g.hang_ms > hang_max_ms {
        return Err(format!(
            "g5.hang_ms ({} ms) passes kx-target-window.toml's hang_max_ms ({hang_max_ms} ms)",
            g.hang_ms
        ));
    }
    if g.hang_lead_ms >= hold_ms {
        return Err(format!(
            "g5.hang_lead_ms ({} ms) must stay below the test build's hold_ms ({hold_ms} ms)",
            g.hang_lead_ms
        ));
    }
    let room = hold.radius_px.saturating_add(g.still_margin_px);
    if g.drag_px <= room {
        return Err(format!(
            "g5.drag_px ({} px) must pass the still radius plus still_margin_px ({room} px) at this DPI",
            g.drag_px
        ));
    }
    Ok(())
}

/// `first`'s value, or the errors of `first` and `then` joined, so a step that puts things back always reports.
pub(crate) fn both<T>(first: Result<T, String>, then: Result<(), String>) -> Result<T, String> {
    match (first, then) {
        (Ok(v), Ok(())) => Ok(v),
        (Err(e), Ok(())) | (Ok(_), Err(e)) => Err(e),
        (Err(a), Err(b)) => Err(format!("{a}; {b}")),
    }
}

/// True when the hook never timed out: at least one stats line, no reinstall, no send error, and at least `per_case`
/// events for each of the `played` cases.
fn hook_ok(lines: &[HookLine], played: usize, per_case: u64) -> bool {
    let sum = hooklog::sum(lines);
    let want = per_case.saturating_mul(u64::try_from(played).unwrap_or(u64::MAX));
    !lines.is_empty() && sum.reinstalls == 0 && sum.send_errors == 0 && sum.events >= want
}

/// What one G5 run gave, before the test build closed.
struct Run {
    hold: Hold,
    still_px: u32,
    played: Played,
    left_open: Vec<String>,
}

/// The result line: the verdict, which needs the test build to have quit, the cases, the summed stats line with its
/// p99, the lost-echo check with its count, and how the test build ended.
fn result(run: &Run, closed: &Closed, g: &G5) -> Value {
    let lines = closed.lines.as_deref().unwrap_or_default();
    let echoes = hooklog::lost(lines, g.max_lost_echoes);
    let hook = hook_ok(lines, run.played.count, g.min_events_per_case) && echoes.pass;
    let pass = closed.pass(run.played.matched() && hook);
    json!({
        "gate": "G5", "target": PLAIN, "hold_ms": run.hold.ms, "still_px": run.still_px,
        "radius_px": run.hold.radius_px, "pass": pass, "cases": run.played.lines, "hook": hooklog::summed(lines),
        "hook_lines": lines.len(), "echoes": echoes, "admin": ADMIN_NOTE, "test_build": closed.json(),
        "left_open": run.left_open,
    })
}

/// Parks target-window in front, refuses rules this screen breaks, turns Right-click on, plays every case at its
/// centre `rounds` times, then turns Right-click off, even after a failed case.
fn drive(
    ctx: &Ctx,
    build: &TestBuild,
    target: &Opened,
    mouse: &MouseSettings,
    rounds: usize,
) -> Result<(Hold, Played), String> {
    launch::park_target(target.hwnd)?;
    if !win::front(target.hwnd, &ctx.cfg.timing, &ctx.cfg.keys) {
        return Err("could not bring target-window to the front".to_string());
    }
    let p = geom::centre(&win::rect(target.hwnd)?);
    let hold = Hold {
        ms: u64::from(mouse.hold_ms),
        radius_px: g5play::radius(mouse.still_px, screen::dpi_of(&g5play::dot(p))),
    };
    let (alive_ms, hang_max_ms) = (ctx.cfg.mouse_guard.alive_ms, ctx.target.hang_max_ms);
    refuse(hold, alive_ms, &ctx.cfg.g5, hang_max_ms)?;
    build.set(RCLICK, true)?;
    let played = g5play::play_all(ctx, target.hwnd, p, hold, rounds);
    let played = both(played, build.set(RCLICK, false))?;
    Ok((hold, played))
}

/// Starts target-window and drives G5's cases on it `rounds` times under `build`, then closes it, noting what was left
/// open; G5 and G8's hook part share it.
pub(crate) fn on_target(
    ctx: &Ctx,
    build: &TestBuild,
    mouse: &MouseSettings,
    rounds: usize,
) -> (Result<(Hold, Played), String>, Vec<String>) {
    let target = match launch::start_target(ctx) {
        Ok(t) => t,
        Err(e) => return (Err(e), Vec::new()),
    };
    say!("G5's cases on {}", win::describe(target.hwnd));
    let driven = drive(ctx, build, &target, mouse, rounds);
    let mut left_open = probe::released();
    left_open.extend(apps::clean_up(ctx, target));
    (driven, left_open)
}

/// Runs G5 on target-window under the test build: `plain` only, as `admin` is the owner's manual check.
pub fn run(ctx: &Ctx, name: &str) -> Result<Value, String> {
    target_of(name)?;
    let build = TestBuild::open(ctx, None)?;
    let mouse = build.mouse()?;
    let (driven, left_open) = on_target(ctx, &build, &mouse, ctx.cfg.g5.rounds);
    let closed = build.close();
    let (hold, played) = driven.map_err(|e| closed.failed(&e, &left_open))?;
    let run = Run {
        hold,
        still_px: mouse.still_px,
        played,
        left_open,
    };
    let v = result(&run, &closed, &ctx.cfg.g5);
    say!("G5 hook {}; pass {}", v["hook"], v["pass"]);
    Ok(v)
}

#[cfg(test)]
mod tests;
