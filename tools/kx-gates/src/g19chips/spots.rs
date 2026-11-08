//! Where the strip goes when the selection sits low or on another screen, on a filled target-window the gate starts and
//! closes, never on Notepad: its window moves so its first line sits `near_px` above the plate or the screen's bottom,
//! then back. The strip must then go above it (judged), its place on a second screen is a note, and a case whose
//! window cannot move says it was not exercised.
#![cfg(windows)]

use std::collections::HashSet;

use kx_mod_keyboard::Nav;
use kx_module_api::ports::{Point, Rect};
use kx_platform::ui::Screens;
use kx_platform_windows::screen::{WinScreens, contains, edges};
use serde_json::{Value, json};
use windows::Win32::UI::WindowsAndMessaging::IsZoomed;

use crate::apps::Opened;
use crate::g19chips::docs::Line;
use crate::g19chips::gate::Gate;
use crate::g19chips::kept::{Kept, Target};
use crate::g19chips::notepad::Lines;
use crate::g19chips::place;
use crate::g19chips::plan::EXERCISED;
use crate::g25::part;
use crate::ownkeys::{self, Desk, Real};
use crate::win::{self, sleep_ms};
use crate::{board, launch, probe};

/// The acts the step takes at the least: the near-bottom case's click, Shift+End and our Esc; a second screen adds as
/// many.
pub const ACTS: usize = 3;
/// The target's first line, which its rows hold where Notepad's hold the copy line.
const FIRST: Line = Line::Copy;

/// The gate's started target-window, kept so it goes back and closes also on a panic.
type Mine<'c> = Kept<Target<'c>>;

/// The lowest a line may reach on `work` before the strip has no room below: the screen's bottom, or the top of a
/// box of ours under the line's span.
pub fn floor(work: Rect, ours: &[Rect], line: Rect) -> i32 {
    let [left, top, right, _] = edges(line);
    let under = |b: &&Rect| {
        let [bl, bt, br, _] = edges(**b);
        bl < right && left < br && bt > top
    };
    let bottom = edges(work)[3];
    ours.iter()
        .filter(under)
        .map(|b| b.y)
        .fold(bottom, i32::min)
}

/// How far a window moves down (or up) so `line`'s bottom sits `near` px above `floor`.
pub fn lift(line: Rect, floor: i32, near: i32) -> i32 {
    floor.saturating_sub(near).saturating_sub(edges(line)[3])
}

/// The first work area of `areas` that does not hold `at`.
pub fn other_screen(areas: &[Rect], at: Point) -> Option<Rect> {
    areas.iter().copied().find(|a| !contains(*a, at))
}

/// True when window `w` is the gate's own: it was not among the windows `before` that the gate took before it started
/// it, so it is no window of the owner's.
pub fn own(before: &HashSet<isize>, w: isize) -> bool {
    !before.contains(&w)
}

/// Why the gate leaves a window where it is: not its `own`, or `zoomed` (maximised); none when it may move.
pub fn stays(own: bool, zoomed: bool) -> Option<&'static str> {
    if !own {
        return Some("not moved: the window is not the gate's own");
    }
    zoomed.then_some("not moved: the window is maximised")
}

/// Whether `app`'s window moves: none once `ours` allows it on `desk`, else why it stays; an error while it is not
/// in front.
pub fn movable(
    desk: &dyn Desk,
    app: &Opened,
    (own, zoomed): (bool, bool),
) -> Result<Option<&'static str>, String> {
    if let Some(why) = stays(own, zoomed) {
        return Ok(Some(why));
    }
    ownkeys::ours(desk, app).map(|()| None)
}

/// A case that did not run, and `why`: no verdict, so it is never a pass.
pub fn unrun(why: &str) -> Value {
    json!({ EXERCISED: false, "reason": why })
}

/// True while window `app` is maximised.
#[allow(unsafe_code, reason = "A plain window state query.")]
fn zoomed(app: &Opened) -> bool {
    // SAFETY: a plain query; a dead window reads as not maximised.
    unsafe { IsZoomed(app.hwnd) }.as_bool()
}

/// `drive` run on `lines` with the `kept` window moved to `to`, once `movable` allows it for a window that is the
/// gate's `own`, then moved back whatever happened, also on a panic; why it stayed, when it did.
fn moved<T>(
    gate: &Gate,
    (kept, own): (&Mine, bool),
    lines: &Lines,
    to: Point,
    drive: impl FnOnce(&Lines) -> Result<T, String>,
) -> Result<Result<T, &'static str>, String> {
    let app = &kept.get().app;
    let was = win::rect(app.hwnd)?;
    if let Some(why) = movable(&Real, app, (own, zoomed(app)))? {
        return Ok(Err(why));
    }
    let from = Point {
        x: was.left,
        y: was.top,
    };
    kept.go(to, from)?;
    sleep_ms(gate.g().settle_ms);
    let now = win::rect(app.hwnd);
    let got = now.and_then(|r| drive(&lines.shifted(r.left - was.left, r.top - was.top)));
    let back = kept.back();
    let v = got?;
    back.map(|()| Ok(v))
}

/// One case on the target's first line, too long for its box: a click at its start and Shift+End select it, and our
/// Esc clears it after.
fn one(gate: &Gate, app: &Opened, lines: &Lines, name: &str) -> Result<Value, String> {
    let before = Gate::before();
    gate.acts.click(app, lines.start(FIRST))?;
    gate.acts.keys(app, &gate.g().select_end)?;
    let (case, _) = gate.case(name, &before, &gate.g().editable_chips)?;
    gate.acts.key(app, &board::nav_key(Nav::Esc)?)?;
    Ok(case)
}

/// The first line moved down to `near_px` above the plate or the screen's bottom: the strip must sit above it.
fn near_bottom(
    gate: &Gate,
    (kept, own): (&Mine, bool),
    lines: &Lines,
    cases: &mut Vec<Value>,
) -> Result<Value, String> {
    let app = &kept.get().app;
    let line = lines.of(FIRST);
    let work =
        kx_platform_windows::screen::work_area(&win::rect(app.hwnd)?).map_err(|e| e.to_string())?;
    let low = floor(place::rect(&work), &gate.clear().boxes, line);
    let dy = lift(line, low, i32::from(gate.g().near_px));
    let r = win::rect(app.hwnd)?;
    let to = Point {
        x: r.left,
        y: r.top.saturating_add(dy),
    };
    let got = moved(gate, (kept, own), lines, to, |l| {
        one(gate, app, l, "near_bottom")
    })?;
    let mut case = match got {
        Ok(case) => case,
        Err(why) => return Ok(unrun(why)),
    };
    let above = case["placed"]["above"] == true;
    case["place"] = json!(case["place"] == true && above);
    let ok = case["seen"] == true && above;
    cases.push(case);
    Ok(json!({ EXERCISED: true, "floor": low, "moved_px": dy, "above": above, "ok": ok }))
}

/// The strip's place on a second screen, when one exists; a note, never a verdict.
fn second_screen(gate: &Gate, (kept, own): (&Mine, bool), lines: &Lines) -> Value {
    let app = &kept.get().app;
    let areas: Vec<Rect> = WinScreens.work_areas().iter().map(|s| s.rect).collect();
    let r = place::rect(&win::rect(app.hwnd).unwrap_or_default());
    let middle = Point {
        x: r.x.saturating_add(r.w / 2),
        y: r.y.saturating_add(r.h / 2),
    };
    let Some(area) = other_screen(&areas, middle) else {
        return unrun("one screen");
    };
    let inset = gate.ctx.cfg.probes.inset_px;
    let to = Point {
        x: area.x.saturating_add(inset),
        y: area.y.saturating_add(inset),
    };
    let got = moved(gate, (kept, own), lines, to, |l| {
        one(gate, app, l, "second_screen")
    });
    match got {
        Ok(Ok(c)) => json!({ EXERCISED: true, "screens": areas.len(), "placed": c["placed"] }),
        Ok(Err(why)) => unrun(why),
        Err(e) => json!({ "screens": areas.len(), "error": e }),
    }
}

/// Both place cases when the target cannot be used: the near-bottom case fails with `e`, and the second screen says it.
pub fn neither(e: String) -> (Value, Value) {
    let second = json!({ "error": e });
    (part(Err(e)), second)
}

/// Both place cases, `run` on the target's `rows` once read; with none, both report why, as there is no other window
/// to fall back on.
pub fn on_rows<L>(
    rows: Result<L, String>,
    run: impl FnOnce(L) -> (Value, Value),
) -> (Value, Value) {
    rows.map_or_else(neither, run)
}

/// Both place cases on the `kept` target, in front, which is the gate's `own` when new; its rows read once.
fn on_target(gate: &Gate, (kept, own): (&Mine, bool), cases: &mut Vec<Value>) -> (Value, Value) {
    let app = &kept.get().app;
    let name = &gate.ctx.target.edit_name;
    let rows = probe::front(gate.ctx, app).and_then(|()| {
        Lines::read_in(gate, || gate.uia().named(app.hwnd, name)).map_err(|e| {
            format!("the target-window's box shows no text rows through UI Automation: {e}")
        })
    });
    on_rows(rows, |lines| {
        let near = part(near_bottom(gate, (kept, own), &lines, cases));
        let second = probe::front(gate.ctx, app).map_or_else(
            |e| json!({ "error": e }),
            |()| second_screen(gate, (kept, own), &lines),
        );
        (near, second)
    })
}

/// The near-bottom case and the second screen on a target-window filled with its numbered lines, which the gate starts
/// and closes after, also on an error or a panic.
pub fn run(gate: &Gate, cases: &mut Vec<Value>, left: &mut Vec<String>) -> (Value, Value) {
    let ctx = gate.ctx;
    let before = win::snapshot();
    let target = match launch::start_target_with(ctx, std::slice::from_ref(&ctx.target.fill_arg)) {
        Ok(t) => Kept::new(Target::new(ctx, t)),
        Err(e) => return neither(e),
    };
    let mine = own(&before, win::key(target.get().app.hwnd));
    let got = on_target(gate, (&target, mine), cases);
    left.extend(target.close());
    got
}

#[cfg(test)]
mod tests;
