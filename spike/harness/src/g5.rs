//! G5 hold engine: with Right-click on, short clicks and drags stay normal and a still hold right-clicks.

use serde_json::{Value, json};
use spike_core::clock::now_us;
use spike_core::hold::{Act, Button, Mode, Pt};
use spike_core::targetlog::Press;
use spike_core::uiaccess;
use windows::Win32::Foundation::{HWND, POINT};

use crate::apps::{self, Ctx, Opened};
use crate::assist::{self, Assist, Report, Setup};
use crate::config::G5;
use crate::hookio::Source;
use crate::tlog::{self, Mouse};
use crate::win::{self, sleep_ms};
use crate::{launch, stats};

/// What a normal click or drag logs.
const LEFT: &[Press] = &[Press::LeftDown, Press::LeftUp];
/// What a right-click logs; target-window swallows the menu it asks for.
const RIGHT: &[Press] = &[Press::RightDown, Press::RightUp, Press::Menu];
/// G5 targets: target-window as is.
pub const PLAIN: &str = "plain";
/// G5 targets: target-window as administrator.
pub const ADMIN: &str = "admin";

/// One scripted press: move `dx` px right while pressed, then a long hold or a short click.
struct Case {
    name: &'static str,
    dx: i32,
    long: bool,
    want: &'static [Press],
}

/// The five cases; the wiggle stays inside the still radius and the short move just leaves it.
fn cases(g: &G5, still_px: i32) -> [Case; 5] {
    let case = |name, dx, long, want| Case {
        name,
        dx,
        long,
        want,
    };
    [
        case("click", 0, false, LEFT),
        case("drag", g.drag_px, false, LEFT),
        case("hold", 0, true, RIGHT),
        case("wiggle_hold", still_px - g.still_margin_px, true, RIGHT),
        case("past_still", still_px + g.still_margin_px, false, LEFT),
    ]
}

/// True when the log shows exactly the wanted presses: a hold all at `p`, a click from `p` to `dx` px right.
fn case_ok(want: &[Press], long: bool, got: &[Mouse], p: Pt, dx: i32, slack: i32) -> bool {
    let near = |m: &Mouse, q: Pt| (m.x - q.x).abs() <= slack && (m.y - q.y).abs() <= slack;
    let end = Pt {
        x: p.x + dx,
        y: p.y,
    };
    let kinds_ok = got.iter().map(|m| m.press).eq(want.iter().copied());
    let start_ok = got.first().is_some_and(|m| near(m, p));
    let end_ok = if long {
        got.iter().all(|m| near(m, p))
    } else {
        got.last().is_some_and(|m| near(m, end))
    };
    kinds_ok && start_ok && end_ok
}

/// The centre of `hwnd` on screen, in physical pixels.
fn centre(hwnd: HWND) -> Result<Pt, String> {
    let r = win::rect(hwnd)?;
    Ok(Pt {
        x: (r.left + r.right) / 2,
        y: (r.top + r.bottom) / 2,
    })
}

/// Plays `case` at `p` as the simulated user; once pressed, the button is always released.
fn play(ctx: &Ctx, case: &Case, p: Pt) -> Result<(), String> {
    let (g, sim, hold_ms) = (&ctx.cfg.g5, &ctx.cfg.sim, ctx.cfg.assist.hold_ms);
    let end = Pt {
        x: p.x + case.dx,
        y: p.y,
    };
    assist::as_user(&[Act::Move(p)])?;
    sleep_ms(sim.step_ms);
    assist::as_user(&[Act::Down(Button::Left, p)])?;
    sleep_ms(sim.step_ms);
    let moved = match case.dx {
        0 => Ok(()),
        _ => assist::as_user(&[Act::Move(end)]),
    };
    sleep_ms(if case.long {
        hold_ms + sim.hold_margin_ms
    } else {
        sim.click_ms
    });
    let released = assist::as_user(&[Act::Up(Button::Left, end)]);
    sleep_ms(g.settle_ms);
    moved.and(released)
}

/// The mouse presses target-window logged at or after `since` (µs).
fn presses(ctx: &Ctx, since: i64) -> Result<Vec<Mouse>, String> {
    let path = ctx.spike.resolve(&ctx.spike.target.log);
    let log = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(tlog::mouse(&log)
        .into_iter()
        .filter(|m| m.us >= since)
        .collect())
}

/// `Err` naming the window over `p`, unless it is `target`.
fn uncovered(target: HWND, p: Pt) -> Result<(), String> {
    let top = win::root_at(POINT { x: p.x, y: p.y });
    if top == target {
        Ok(())
    } else {
        Err(format!("{} covers the test point", win::describe(top)))
    }
}

/// Plays every case at `p`, round after round, checking the log after each; returns the lines and the verdict.
fn play_all(ctx: &Ctx, target: HWND, p: Pt) -> Result<(Vec<Value>, bool), String> {
    let g = &ctx.cfg.g5;
    let (mut lines, mut passed) = (Vec::new(), true);
    for round in 0..g.rounds {
        for case in cases(g, ctx.cfg.assist.still_px) {
            uncovered(target, p)?;
            let since = now_us();
            play(ctx, &case, p)?;
            let got = presses(ctx, since)?;
            let ok = case_ok(case.want, case.long, &got, p, case.dx, g.point_slack_px);
            passed &= ok;
            let seen: Vec<String> = got
                .iter()
                .map(|m| format!("{}@{},{}", m.press.name(), m.x, m.y))
                .collect();
            let verdict = if ok { "ok" } else { "FAIL" };
            println!("{round} {}: {verdict} {}", case.name, seen.join(" "));
            lines.push(json!({ "round": round, "case": case.name, "ok": ok, "got": seen }));
        }
    }
    Ok((lines, passed))
}

/// What one run of the cases gave.
struct Drive {
    cases: Vec<Value>,
    passed: bool,
    report: Report,
    /// The hook still saw input after the last case.
    alive: bool,
}

/// Turns Right-click on and plays every case on the target, then checks the hook still runs.
fn drive(ctx: &Ctx, target: &Opened) -> Result<Drive, String> {
    let (a, g) = (&ctx.cfg.assist, &ctx.cfg.g5);
    if !win::front(target.hwnd, &ctx.cfg.timing, &ctx.cfg.keys) {
        return Err("could not bring target-window to the front".to_string());
    }
    let p = centre(target.hwnd)?;
    let assist = Assist::start(Setup {
        hold_ms: a.hold_ms,
        still_px: a.still_px,
        source: Source::Simulated,
        own: Vec::new(),
        reply_ms: a.reply_ms,
    })?;
    assist.set_mode(Mode::RightClick)?;
    let (cases_out, passed) = play_all(ctx, target.hwnd, p)?;
    let before = assist.live().seen;
    assist::as_user(&[Act::Move(p)])?;
    sleep_ms(g.settle_ms);
    let alive = assist.live().seen > before;
    let report = assist.stop()?;
    Ok(Drive {
        cases: cases_out,
        passed,
        report,
        alive,
    })
}

/// Hook call times in ms: the p99, and every percentile plus the longest and the count.
fn hook_ms(us: &[i64]) -> (Option<f64>, Value) {
    let mut ms: Vec<f64> = us.iter().map(|&u| u as f64 / stats::US_PER_MS).collect();
    ms.sort_by(f64::total_cmp);
    let mut out = serde_json::Map::new();
    for (name, rank) in stats::PERCENTILES {
        out.insert(name.to_string(), json!(stats::percentile(&ms, rank)));
    }
    out.insert("max".to_string(), json!(ms.last()));
    out.insert("calls".to_string(), json!(ms.len()));
    (stats::percentile(&ms, stats::P99), Value::Object(out))
}

/// Closes target-window; one started as administrator is closed through its window.
fn close(ctx: &Ctx, target: Opened) -> Vec<String> {
    if target.child.is_some() {
        return apps::clean_up(ctx, target);
    }
    let t = &ctx.cfg.timing;
    let _ = win::close(target.hwnd);
    if win::wait_gone(target.hwnd, t.close_wait_ms, t.poll_ms) {
        Vec::new()
    } else {
        vec![format!("left open: {}", win::describe(target.hwnd))]
    }
}

/// Starts the target called `name`; the admin one needs uiAccess, or we could not close it.
fn start(ctx: &Ctx, name: &str) -> Result<Opened, String> {
    match name {
        PLAIN => launch::start_target(ctx),
        ADMIN if !uiaccess::active() => {
            Err("G5 admin needs the installed uiAccess harness to close its window".to_string())
        }
        ADMIN => launch::start_target_admin(ctx),
        _ => Err(format!("G5 runs on {PLAIN} or {ADMIN}, not {name}")),
    }
}

/// Runs G5 on target-window, `plain` or `admin`.
pub fn run(ctx: &Ctx, name: &str) -> Result<Value, String> {
    let target = start(ctx, name)?;
    println!("G5 {name}: {}", win::describe(target.hwnd));
    let driven = drive(ctx, &target);
    let left_open = close(ctx, target);
    let d = driven.map_err(|e| format!("{e}; left open: {left_open:?}"))?;
    let (p99, hook) = hook_ms(&d.report.hook_us);
    let fast = p99.is_some_and(|ms| ms <= ctx.cfg.g5.hook_p99_ms);
    let pass = d.passed && d.alive && fast && d.report.errors.is_empty();
    println!("hook ms {hook}; alive {}; pass {pass}", d.alive);
    Ok(json!({
        "gate": "G5", "target": name, "hold_ms": ctx.cfg.assist.hold_ms, "pass": pass,
        "cases": d.cases, "hook_ms": hook, "hook_alive": d.alive, "seen": d.report.seen,
        "max_wait_ms": d.report.max_wait_ms,
        "errors": d.report.errors, "left_open": left_open,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    const P: Pt = Pt { x: 100, y: 50 };

    fn at(press: Press, x: i32, y: i32) -> Mouse {
        Mouse { press, us: 0, x, y }
    }

    #[test]
    fn a_drag_must_start_at_the_press_and_end_where_it_moved() {
        let got = [at(Press::LeftDown, 100, 50), at(Press::LeftUp, 130, 51)];
        assert!(case_ok(LEFT, false, &got, P, 30, 1));
        assert!(!case_ok(LEFT, false, &got, P, 0, 1));
        let late = [at(Press::LeftDown, 103, 50), at(Press::LeftUp, 130, 50)];
        assert!(!case_ok(LEFT, false, &late, P, 30, 1));
    }

    #[test]
    fn a_hold_must_log_a_right_click_all_at_the_press() {
        let right = [
            at(Press::RightDown, 100, 50),
            at(Press::RightUp, 100, 50),
            at(Press::Menu, 100, 50),
        ];
        assert!(case_ok(RIGHT, true, &right, P, 6, 1));
        let moved = [right[0], at(Press::RightUp, 106, 50), right[2]];
        assert!(!case_ok(RIGHT, true, &moved, P, 6, 1));
        let also_left = [at(Press::LeftDown, 100, 50), right[0], right[1], right[2]];
        assert!(!case_ok(RIGHT, true, &also_left, P, 0, 1));
        assert!(!case_ok(RIGHT, true, &[], P, 0, 1));
    }

    #[test]
    fn the_cases_straddle_the_still_radius_by_more_than_the_slack() {
        let cfg = crate::config::load().expect("harness.toml loads");
        let (g, still) = (&cfg.g5, cfg.assist.still_px);
        let [_, drag, _, wiggle, past] = cases(g, still);
        assert!(g.still_margin_px > g.point_slack_px);
        assert!(wiggle.dx > 0 && wiggle.dx < still);
        assert!(past.dx > still && drag.dx > past.dx);
    }

    #[test]
    fn hook_ms_reports_a_true_p99_over_many_calls() {
        let us: Vec<i64> = (1..=200).collect();
        let (p99, all) = hook_ms(&us);
        assert_eq!(p99, Some(0.198));
        assert_eq!(all["calls"], json!(200));
        assert_eq!(all["max"], json!(0.2));
        assert_eq!(hook_ms(&[]).0, None);
    }
}
