//! When the strip hides (P10 design "When the chips hide" and "Keyboard states"): a press outside it, our keyboard's
//! Esc, our panel opening with its search aimed, and the keyboard closed to the tray, which is the run's last act; the
//! minimised keyboard follows `minimised`.
#![cfg(windows)]

use kx_mod_keyboard::{Nav, Role};
use kx_module_api::ports::{Point, Rect};
use kx_platform_windows::screen::contains;
use serde_json::{Value, json};
use windows::Win32::Foundation::POINT;

use crate::apps::Opened;
use crate::board;
use crate::clippanel::Panel;
use crate::g19chips::docs::Line;
use crate::g19chips::gate::Gate;
use crate::g19chips::notepad::{Lines, by_keys};
use crate::g19chips::place;
use crate::g19chips::strip::Shown;
use crate::product::{self, Keyboard};
use crate::win::{self, sleep_ms};

/// The acts the step and the tray take: five selections by keys, a press and its release, Esc three times, the clip
/// key twice, the panel's two keys, three buttons pressed and the tray's selection.
pub const ACTS: usize = 24;
/// The cases that must show no strip: the aimed panel's and the tray's.
pub const QUIET: usize = 2;

/// The first of `points` under none of `ours`, our windows by name and box; else each point with the window over it.
pub fn clear_of(points: &[POINT], ours: &[(&str, Rect)]) -> Result<POINT, String> {
    let over = |p: &POINT| {
        let at = Point { x: p.x, y: p.y };
        ours.iter()
            .find(|(_, r)| contains(*r, at))
            .map(|(name, _)| *name)
    };
    if let Some(p) = points.iter().find(|p| over(p).is_none()) {
        return Ok(*p);
    }
    let each: Vec<String> = (points.iter())
        .map(|p| {
            format!(
                "({}, {}) under the {}",
                p.x,
                p.y,
                over(p).unwrap_or_default()
            )
        })
        .collect();
    Err(format!(
        "no test point is clear of our windows: {}",
        each.join(", ")
    ))
}

/// Our windows a press outside the strip `s` must miss: the strip, and the keyboard's plate or bubble.
fn ours(gate: &Gate, s: &Shown) -> Vec<(&'static str, Rect)> {
    let boxes = gate.clear().boxes.into_iter().map(|b| ("keyboard", b));
    std::iter::once(("strip", place::rect(&s.rect)))
        .chain(boxes)
        .collect()
}

/// The lines' points a press outside the strip may use, the Arabic line's end first.
fn outside_points(lines: &Lines) -> [POINT; 4] {
    [
        lines.end(Line::Arabic),
        lines.caret(),
        lines.end(Line::Paste),
        lines.end(Line::Cut),
    ]
}

/// The copy line selected by keys, as case `name`; the strip, if it showed.
fn select(gate: &Gate, app: &Opened, lines: &Lines, name: &str) -> Result<Option<Shown>, String> {
    Ok(by_keys(gate, app, lines, Line::Copy, name)?.1)
}

/// Presses the product's button `id` through UI Automation, then pauses `settle_ms`.
fn press(gate: &Gate, id: &str) -> Result<(), String> {
    product::invoke(&gate.reader, id)?;
    sleep_ms(gate.g().settle_ms);
    Ok(())
}

/// Our panel opens on the clip key, aimed at its search, and the strip goes; a selection made then shows none, and
/// the clip key shuts the panel. `cases` gathers the aimed case.
fn panel(
    gate: &Gate,
    app: &Opened,
    lines: &Lines,
    cases: &mut Vec<Value>,
) -> Result<Value, String> {
    let g = gate.g();
    let (clip, panel) = (
        board::id_of(Role::Clip)?,
        Panel::new(gate.ctx, g.gone_wait_ms)?,
    );
    let shown = select(gate, app, lines, "panel_before")?.is_some();
    gate.acts.key(app, &clip)?;
    let opened = panel.wait(true);
    let hid = shown && gate.strip.goes(|_| true)?;
    let before = Gate::before();
    gate.acts.keys(app, &g.caret_left)?;
    gate.acts.keys(app, &g.select_home)?;
    cases.push(gate.quiet("panel_aimed", &before)?);
    gate.acts.key(app, &clip)?;
    let shut = panel.wait(false);
    gate.acts.key(app, &board::nav_key(Nav::Esc)?)?;
    let ok = hid && opened.is_ok() && shut.is_ok();
    Ok(json!({ "hid": hid, "opened": opened.is_ok(), "shut": shut.is_ok(), "ok": ok }))
}

/// The keyboard minimised to its bubble: the strip shows as `minimised` says, then the bubble brings it back.
fn minimised(gate: &Gate, app: &Opened, lines: &Lines) -> Result<Value, String> {
    let (g9, g) = (&gate.ctx.cfg.g9, gate.g());
    press(gate, &g9.put_away)?;
    let shows = select(gate, app, lines, "minimised")?.is_some();
    press(gate, &g9.bring_back)?;
    let poll = gate.ctx.cfg.timing.poll_ms;
    let back = win::poll_until(g.strip_wait_ms, poll, || Keyboard::find().ok()).is_some();
    gate.acts.key(app, &board::nav_key(Nav::Esc)?)?;
    let ok = shows == gate.sel.minimised && back;
    Ok(json!({ "shows": shows, "minimised": gate.sel.minimised, "back": back, "ok": ok }))
}

/// A press outside the strip, our Esc, our aimed panel and the minimised keyboard; `cases` gathers the panel's.
pub fn run(
    gate: &Gate,
    app: &Opened,
    lines: &Lines,
    cases: &mut Vec<Value>,
) -> Result<Value, String> {
    let outside = match select(gate, app, lines, "outside")? {
        Some(s) => {
            let at = clear_of(&outside_points(lines), &ours(gate, &s))?;
            gate.acts.down(app, at)?;
            let went = gate.strip.goes(|_| true);
            gate.acts.up(at)?;
            went?
        }
        None => false,
    };
    let esc = match select(gate, app, lines, "esc")? {
        Some(_) => gate
            .acts
            .key(app, &board::nav_key(Nav::Esc)?)
            .and_then(|()| gate.strip.goes(|_| true))?,
        None => false,
    };
    let panel = panel(gate, app, lines, cases)?;
    let minimised = minimised(gate, app, lines)?;
    let ok = outside && esc && panel["ok"] == true && minimised["ok"] == true;
    Ok(json!({ "outside": outside, "esc": esc, "panel": panel, "minimised": minimised, "ok": ok }))
}

/// The keyboard closed to the tray: the strip goes, and a selection made then shows none. The copy stays in the tray
/// until the run ends it.
pub fn tray(
    gate: &Gate,
    app: &Opened,
    lines: &Lines,
    cases: &mut Vec<Value>,
) -> Result<Value, String> {
    let shown = select(gate, app, lines, "tray_before")?.is_some();
    press(gate, &gate.g().tray)?;
    let hid = shown && gate.strip.goes(|_| true)?;
    let before = Gate::before();
    gate.acts.click(app, lines.end(Line::Copy))?;
    gate.acts.keys(app, &gate.g().select_home)?;
    let quiet = gate.quiet("tray", &before)?;
    let ok = hid && quiet["chips"] == true;
    cases.push(quiet);
    Ok(json!({ "hid": hid, "ok": ok }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r(x: i32, y: i32, w: i32, h: i32) -> Rect {
        Rect { x, y, w, h }
    }

    #[test]
    fn the_press_outside_goes_to_the_first_point_clear_of_our_windows() {
        let strip = ("strip", r(393, 271, 320, 46));
        let board = ("keyboard", r(541, 763, 838, 244));
        let under = POINT { x: 500, y: 316 };
        let below = POINT { x: 430, y: 338 };
        assert_eq!(clear_of(&[under, below], &[strip, board]), Ok(below));
        assert_eq!(clear_of(&[below], &[strip, board]), Ok(below));
        let e = clear_of(&[under], &[board, strip]).expect_err("none clear");
        assert!(e.contains("(500, 316) under the strip"), "{e}");
        let edge = POINT { x: 500, y: 317 };
        assert_eq!(
            clear_of(&[edge], &[strip]),
            Ok(edge),
            "the far edge is outside"
        );
    }
}
