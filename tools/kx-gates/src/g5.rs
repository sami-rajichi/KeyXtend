//! G5 hold engine on the product's test build: with Right-click on, short clicks and drags stay normal, a still hold
//! right-clicks, even into a hung target, and the hook never times out. Plain targets only; admin windows are manual.
#![cfg(windows)]

use kx_mod_keyboard::{Assist, Role};
use kx_mod_mouse::MouseSettings;
use kx_platform_windows::screen;
use serde_json::{Value, json};
use spike_core::uia;

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
/// Why the admin-window case is the owner's manual check (ruling B); `g5 admin` says it and every `plain` result
/// carries it.
pub const ADMIN_NOTE: &str = "The admin-window case is the owner's manual check on the installed product: it needs \
                              uiAccess, and the test build is never staged, signed or installed, so Windows blocks \
                              its input into admin windows.";
/// The key that turns Right-click on and off.
const RCLICK: Role = Role::Assist(Assist::RClick);

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
fn both<T>(first: Result<T, String>, then: Result<(), String>) -> Result<T, String> {
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

/// The result line: the verdict, the cases, the summed stats line with its p99, and how the test build ended.
fn result(run: &Run, closed: &Closed, per_case: u64) -> Value {
    let lines = closed.lines.as_deref().unwrap_or_default();
    let pass = run.played.matched && hook_ok(lines, run.played.count, per_case);
    let sum = hooklog::sum(lines);
    let mut hook = serde_json::to_value(sum).unwrap_or_default();
    if let Some(fields) = hook.as_object_mut() {
        fields.insert("p99_ms".to_string(), json!(sum.p99_ms()));
    }
    json!({
        "gate": "G5", "target": PLAIN, "hold_ms": run.hold.ms, "still_px": run.still_px,
        "radius_px": run.hold.radius_px, "pass": pass, "cases": run.played.lines, "hook": hook,
        "hook_lines": lines.len(), "admin": ADMIN_NOTE, "test_build": closed.json(), "left_open": run.left_open,
    })
}

/// Parks target-window in front, refuses rules this screen breaks, turns Right-click on, plays every case at its
/// centre, then turns Right-click off, even after a failed case.
fn drive(
    ctx: &Ctx,
    build: &TestBuild,
    target: &Opened,
    mouse: &MouseSettings,
) -> Result<(Hold, Played), String> {
    launch::park_target(target.hwnd)?;
    if !win::front(target.hwnd, &ctx.cfg.timing, &ctx.cfg.keys) {
        return Err("could not bring target-window to the front".to_string());
    }
    let p = uia::centre(&win::rect(target.hwnd)?);
    let hold = Hold {
        ms: u64::from(mouse.hold_ms),
        radius_px: g5play::radius(mouse.still_px, screen::dpi_of(&g5play::dot(p))),
    };
    let (alive_ms, hang_max_ms) = (ctx.cfg.mouse_guard.alive_ms, ctx.target.hang_max_ms);
    refuse(hold, alive_ms, &ctx.cfg.g5, hang_max_ms)?;
    build.set(RCLICK, true)?;
    let played = g5play::play_all(ctx, target.hwnd, p, hold);
    let played = both(played, build.set(RCLICK, false))?;
    Ok((hold, played))
}

/// Runs G5 on target-window under the test build: `plain` only, as `admin` is the owner's manual check.
pub fn run(ctx: &Ctx, name: &str) -> Result<Value, String> {
    target_of(name)?;
    let build = TestBuild::open(ctx, None)?;
    let mouse = build.mouse()?;
    let target = launch::start_target(ctx)?;
    say!("G5 {name}: {}", win::describe(target.hwnd));
    let driven = drive(ctx, &build, &target, &mouse);
    let mut left_open = probe::released();
    left_open.extend(apps::clean_up(ctx, target));
    let closed = build.close();
    let (hold, played) = driven.map_err(|e| {
        format!(
            "{e}; left open: {left_open:?}; test build: {}",
            closed.json()
        )
    })?;
    let run = Run {
        hold,
        still_px: mouse.still_px,
        played,
        left_open,
    };
    let v = result(&run, &closed, ctx.cfg.g5.min_events_per_case);
    say!("G5 hook {}; pass {}", v["hook"], v["pass"]);
    Ok(v)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Cases one run plays.
    const PLAYED: usize = 6;

    fn g5() -> G5 {
        crate::config::load().expect("kx-gates.toml loads").g5
    }

    fn line(events: u64, reinstalls: u32, send_errors: u32) -> HookLine {
        HookLine {
            events,
            reinstalls,
            send_errors,
            ..HookLine::default()
        }
    }

    #[test]
    fn the_hook_passes_with_a_line_no_reinstall_no_send_error_and_enough_events() {
        let per = g5().min_events_per_case;
        let enough = per * PLAYED as u64;
        assert!(hook_ok(&[line(enough, 0, 0)], PLAYED, per));
        let halves = [line(enough / 2, 0, 0), line(enough - enough / 2, 0, 0)];
        assert!(hook_ok(&halves, PLAYED, per), "the lines add up");
        assert!(!hook_ok(&[], PLAYED, per), "no line");
        assert!(!hook_ok(&[line(enough, 1, 0)], PLAYED, per), "a reinstall");
        assert!(!hook_ok(&[line(enough, 0, 1)], PLAYED, per), "a send error");
        assert!(!hook_ok(&[line(enough - 1, 0, 0)], PLAYED, per), "too few");
    }

    /// A hold of `ms` with the default still radius at 100 %.
    fn held(ms: u64) -> Hold {
        Hold { ms, radius_px: 7 }
    }

    #[test]
    fn a_hold_the_guard_would_take_for_a_lost_hook_is_refused() {
        let g = g5();
        let (alive, cap) = (2_000, g.hang_ms);
        let e = refuse(held(alive), alive, &g, cap).expect_err("as long as the alive watch");
        assert!(e.contains("alive_ms"), "{e}");
        assert!(refuse(held(alive + 1), alive, &g, cap).is_err());
        assert_eq!(refuse(held(alive - 1), alive, &g, cap), Ok(()));
    }

    #[test]
    fn a_hang_past_target_windows_cap_or_a_lead_past_the_hold_is_refused() {
        let g = g5();
        let alive = 2_000;
        let e = refuse(held(1_500), alive, &g, g.hang_ms - 1).expect_err("past the cap");
        assert!(e.contains("hang_max_ms"), "{e}");
        let e = refuse(held(g.hang_lead_ms), alive, &g, g.hang_ms).expect_err("lead past the hold");
        assert!(e.contains("hang_lead_ms"), "{e}");
    }

    #[test]
    fn a_drag_that_stays_within_the_radius_and_its_margin_is_refused() {
        let g = g5();
        let (alive, cap) = (2_000, g.hang_ms);
        let wide = Hold {
            ms: 1_500,
            radius_px: g.drag_px - g.still_margin_px,
        };
        let e = refuse(wide, alive, &g, cap).expect_err("the drag would stay still");
        assert!(e.contains("drag_px"), "{e}");
        let fits = Hold {
            radius_px: wide.radius_px - 1,
            ..wide
        };
        assert_eq!(refuse(fits, alive, &g, cap), Ok(()));
    }

    #[test]
    fn a_failed_play_and_a_failed_switch_off_both_show() {
        let (play, off) = (
            || Err::<u8, _>("play".to_string()),
            || Err("off".to_string()),
        );
        assert_eq!(both(Ok(1), Ok(())), Ok(1));
        assert_eq!(both(play(), Ok(())), play());
        assert_eq!(both(Ok(1), off()), Err("off".to_string()));
        assert_eq!(both(play(), off()), Err("play; off".to_string()));
    }

    #[test]
    fn admin_is_the_owners_manual_check_and_every_plain_result_says_so() {
        assert_eq!(target_of(PLAIN), Ok(()));
        assert_eq!(target_of(ADMIN), Err(ADMIN_NOTE.to_string()));
        assert!(target_of("x").is_err());
        let per = g5().min_events_per_case;
        let run = Run {
            hold: Hold {
                ms: 1_500,
                radius_px: 9,
            },
            still_px: 7,
            played: Played {
                lines: Vec::new(),
                matched: true,
                count: PLAYED,
            },
            left_open: Vec::new(),
        };
        let closed = |lines| Closed {
            quit: true,
            owner_back: None,
            lines,
        };
        let good = closed(Ok(vec![line(per * PLAYED as u64, 0, 0)]));
        let v = result(&run, &good, per);
        assert_eq!(v["admin"].as_str(), Some(ADMIN_NOTE));
        assert_eq!(
            (v["pass"].as_bool(), v["hook_lines"].as_u64()),
            (Some(true), Some(1))
        );
        assert_eq!(v["test_build"]["quit"].as_bool(), Some(true));
        let unread = result(&run, &closed(Err("no log".into())), per);
        assert_eq!(unread["pass"].as_bool(), Some(false), "no line, no pass");
        assert_eq!(unread["admin"].as_str(), Some(ADMIN_NOTE));
    }
}
