//! G5's cases: what each plays at the test point under the test build's hold, and what target-window must log.
#![cfg(windows)]

use std::time::Instant;

use kx_module_api::ports::{BASE_DPI, physical};
use kx_target_window::hang;
use kx_target_window::log::Press;
use serde_json::{Value, json};
use spike_core::clock::now_us;
use spike_core::pt::Pt;
use windows::Win32::Foundation::{HWND, LPARAM, POINT, RECT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::PostMessageW;

use crate::apps::Ctx;
use crate::out::say;
use crate::p4cfg::G5;
use crate::simin::{self, Act};
use crate::tlog::{self, Mouse};
use crate::win::{self, sleep_ms};

/// What a normal click or drag logs.
pub const LEFT: &[Press] = &[Press::LeftDown, Press::LeftUp];
/// What a right-click logs; target-window swallows the menu it asks for.
pub const RIGHT: &[Press] = &[Press::RightDown, Press::RightUp, Press::Menu];

/// The test build's hold as G5 plays it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hold {
    /// A still press right-clicks after this, in ms.
    pub ms: u64,
    /// Moves up to this far count as still, in physical px at the test point.
    pub radius_px: i32,
}

/// The still radius in physical px of `still_px` logical px on a screen of `dpi`, as the engine measures it; a screen
/// that reports no DPI counts as 100 %.
pub fn radius(still_px: u32, dpi: u32) -> i32 {
    let dpi = if dpi == 0 { BASE_DPI } else { dpi };
    physical(f64::from(still_px), dpi)
}

/// The one-pixel box at `p`, whose screen's DPI the engine reads for a press there.
pub fn dot(p: Pt) -> RECT {
    RECT {
        left: p.x,
        top: p.y,
        right: p.x.saturating_add(1),
        bottom: p.y.saturating_add(1),
    }
}

/// One scripted press: move `dx` px right while pressed, then a long hold or a short click; the hung one makes
/// target-window sleep across its right-click.
pub struct Case {
    /// Its name in the result.
    pub name: &'static str,
    /// How far right it moves while pressed, in physical px.
    pub dx: i32,
    /// It holds past the hold time; else it is a short click.
    pub long: bool,
    /// Target-window sleeps across its right-click.
    pub hang: bool,
    /// What target-window must log.
    pub want: &'static [Press],
}

/// How many cases a round plays.
pub const CASES: usize = 7;

/// The cases; the wiggle stays inside the still radius `radius_px`, the short move just leaves it, and the held drag
/// leaves it, then holds past the hold time, as over a drop target.
pub fn cases(g: &G5, radius_px: i32) -> [Case; CASES] {
    let case = |name, dx, long, hang, want| Case {
        name,
        dx,
        long,
        hang,
        want,
    };
    [
        case("click", 0, false, false, LEFT),
        case("drag", g.drag_px, false, false, LEFT),
        case("hold", 0, true, false, RIGHT),
        case(
            "wiggle_hold",
            radius_px - g.still_margin_px,
            true,
            false,
            RIGHT,
        ),
        case(
            "past_still",
            radius_px + g.still_margin_px,
            false,
            false,
            LEFT,
        ),
        case("held_drag", g.drag_px, true, false, LEFT),
        case("hung_hold", 0, true, true, RIGHT),
    ]
}

/// True when the log shows exactly the wanted presses: a right-click all at `p`, a click or drag from `p` to `dx` px
/// right.
fn case_ok(want: &[Press], got: &[Mouse], p: Pt, dx: i32, slack: i32) -> bool {
    let near = |m: &Mouse, q: Pt| (m.x - q.x).abs() <= slack && (m.y - q.y).abs() <= slack;
    let end = Pt {
        x: p.x + dx,
        y: p.y,
    };
    let kinds_ok = got.iter().map(|m| m.press).eq(want.iter().copied());
    let start_ok = got.first().is_some_and(|m| near(m, p));
    let end_ok = if want == RIGHT {
        got.iter().all(|m| near(m, p))
    } else {
        got.last().is_some_and(|m| near(m, end))
    };
    kinds_ok && start_ok && end_ok
}

/// Asks target-window `target` to sleep `ms`, as a hung app does; it caps the time itself.
#[allow(unsafe_code, reason = "Posts our own message to our own test window.")]
fn hang_target(target: HWND, ms: u64) -> Result<(), String> {
    let asked = WPARAM(usize::try_from(ms).unwrap_or(usize::MAX));
    // SAFETY: posting a message to any window handle is allowed; a bad handle only gives an error.
    unsafe { PostMessageW(Some(target), hang::MESSAGE, asked, LPARAM(0)) }
        .map_err(|e| format!("the hang message to target-window: {e}"))
}

/// Waits while `case` holds since `pressed`: a short click, or the hold time and its margin; the hung case asks for its
/// hang `hang_lead_ms` before the hold time.
fn wait_held(
    ctx: &Ctx,
    target: HWND,
    case: &Case,
    hold: Hold,
    pressed: Instant,
) -> Result<(), String> {
    let (g, sim) = (&ctx.cfg.g5, &ctx.cfg.sim);
    if !case.long {
        sleep_ms(sim.click_ms);
        return Ok(());
    }
    let until = |ms: u64| sleep_ms(ms.saturating_sub(win::ms_since(pressed)));
    let hung = if case.hang {
        until(hold.ms.saturating_sub(g.hang_lead_ms));
        hang_target(target, g.hang_ms)
    } else {
        Ok(())
    };
    until(hold.ms.saturating_add(sim.hold_margin_ms));
    hung
}

/// Plays `case` at `p` as the simulated user; once pressed, the button is always released. The hung case waits for
/// target-window to wake before its log is read.
fn play(ctx: &Ctx, target: HWND, case: &Case, p: Pt, hold: Hold) -> Result<(), String> {
    let (g, sim) = (&ctx.cfg.g5, &ctx.cfg.sim);
    let end = Pt {
        x: p.x + case.dx,
        y: p.y,
    };
    simin::as_user(&[Act::Move(p)])?;
    sleep_ms(sim.step_ms);
    simin::as_user(&[Act::Down(p)])?;
    let pressed = Instant::now();
    sleep_ms(sim.step_ms);
    let moved = match case.dx {
        0 => Ok(()),
        _ => simin::as_user(&[Act::Move(end)]),
    };
    let waited = wait_held(ctx, target, case, hold, pressed);
    let released = simin::as_user(&[Act::Up(end)]);
    sleep_ms(g.settle_ms + if case.hang { g.hang_ms } else { 0 });
    moved.and(waited).and(released)
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

/// What target-window logged, each press named with its offset from the test point `p`, never a screen point.
fn offsets(got: &[Mouse], p: Pt) -> Vec<String> {
    let at = |m: &Mouse| format!("{}{:+},{:+}", m.press.name(), m.x - p.x, m.y - p.y);
    got.iter().map(at).collect()
}

/// What the cases gave.
#[derive(Debug, Clone, PartialEq)]
pub struct Played {
    /// One line per case played: its round, name, verdict and what target-window logged.
    pub lines: Vec<Value>,
    /// How many cases logged what they wanted.
    pub ok: usize,
    /// How many cases played.
    pub count: usize,
}

impl Played {
    /// True when every case played logged what it wanted.
    pub fn matched(&self) -> bool {
        self.ok == self.count
    }
}

/// Plays every case at `p` on `target` for `rounds` rounds, checking the log after each.
pub fn play_all(
    ctx: &Ctx,
    target: HWND,
    p: Pt,
    hold: Hold,
    rounds: usize,
) -> Result<Played, String> {
    let g = &ctx.cfg.g5;
    let mut played = Played {
        lines: Vec::new(),
        ok: 0,
        count: 0,
    };
    for round in 0..rounds {
        for case in cases(g, hold.radius_px) {
            uncovered(target, p)?;
            let since = now_us();
            play(ctx, target, &case, p, hold)?;
            let got = tlog::mouse_since(ctx, since)?;
            let ok = case_ok(case.want, &got, p, case.dx, g.point_slack_px);
            let seen = offsets(&got, p);
            say!(
                "{round} {}: {} {}",
                case.name,
                if ok { "ok" } else { "FAIL" },
                seen.join(" ")
            );
            played
                .lines
                .push(json!({ "round": round, "case": case.name, "ok": ok, "got": seen }));
            played.ok += usize::from(ok);
            played.count += 1;
        }
    }
    Ok(played)
}

#[cfg(test)]
mod tests;
