//! Where the strip goes when the selection sits low or on another screen: the gate's own Notepad window, new since it
//! opened its file and not maximised, moves so its copy line sits `near_px` above the plate or the screen's bottom.
//! The strip must then go above it (judged), its place on a second screen is a note, and the window always moves back.
#![cfg(windows)]

use std::collections::HashSet;

use kx_mod_keyboard::Nav;
use kx_module_api::ports::{Point, Rect};
use kx_platform::ui::Screens;
use kx_platform_windows::screen::{WinScreens, contains, edges};
use serde_json::{Value, json};
use windows::Win32::UI::WindowsAndMessaging::{
    IsZoomed, SWP_NOACTIVATE, SWP_NOSIZE, SWP_NOZORDER, SetWindowPos,
};

use crate::apps::Opened;
use crate::board;
use crate::g19chips::docs::Line;
use crate::g19chips::gate::Gate;
use crate::g19chips::notepad::{Lines, by_keys};
use crate::g19chips::place;
use crate::ownkeys::{self, Desk, Real};
use crate::win::{self, sleep_ms};

/// The acts the step takes at the least: the near-bottom case's selection by keys and our Esc; a second screen adds
/// as many.
pub const ACTS: usize = 3;

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

/// True when window `w` is the gate's own: it was not among the windows `before` that the gate took before it opened
/// its file, so it is no owner's window holding the gate's tab.
pub fn own(before: &HashSet<isize>, w: isize) -> bool {
    !before.contains(&w)
}

/// Why the gate leaves Notepad's window where it is: not its `own`, or `zoomed` (maximised); none when it may move.
pub fn stays(own: bool, zoomed: bool) -> Option<&'static str> {
    if !own {
        return Some("not moved: the window is not the gate's own");
    }
    zoomed.then_some("not moved: Notepad is maximised")
}

/// Whether `app`'s window moves: none once `ours` allows it on `desk`, else why it stays; an error while our tab is
/// not in front, so the owner's window and its other tabs never move.
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

/// True while window `app` is maximised.
#[allow(unsafe_code, reason = "A plain window state query.")]
fn zoomed(app: &Opened) -> bool {
    // SAFETY: a plain query; a dead window reads as not maximised.
    unsafe { IsZoomed(app.hwnd) }.as_bool()
}

/// Moves `app`'s window so its top-left is `to`.
#[allow(unsafe_code, reason = "A plain move of the gate's own window.")]
fn move_to(app: &Opened, to: Point) -> Result<(), String> {
    let flags = SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE;
    // SAFETY: a plain call on the gate's own window; a dead window gives an error.
    unsafe { SetWindowPos(app.hwnd, None, to.x, to.y, 0, 0, flags) }
        .map_err(|e| format!("moving Notepad: {e}"))
}

/// `drive` run on `lines` with `app`'s window moved to `to`, once `movable` allows it for a window that is the gate's
/// `own`, then moved back whatever happened; why it stayed, when it did.
fn moved<T>(
    gate: &Gate,
    (app, own): (&Opened, bool),
    lines: &Lines,
    to: Point,
    drive: impl FnOnce(&Lines) -> Result<T, String>,
) -> Result<Result<T, &'static str>, String> {
    let was = win::rect(app.hwnd)?;
    if let Some(why) = movable(&Real, app, (own, zoomed(app)))? {
        return Ok(Err(why));
    }
    move_to(app, to)?;
    sleep_ms(gate.g().settle_ms);
    let now = win::rect(app.hwnd);
    let got = now.and_then(|r| drive(&lines.shifted(r.left - was.left, r.top - was.top)));
    let back = move_to(
        app,
        Point {
            x: was.left,
            y: was.top,
        },
    );
    let v = got?;
    back.map(|()| Ok(v))
}

/// One case on the copy line of `lines`, cleared with our Esc after.
fn one(gate: &Gate, app: &Opened, lines: &Lines, name: &str) -> Result<Value, String> {
    let (case, _) = by_keys(gate, app, lines, Line::Copy, name)?;
    gate.acts.key(app, &board::nav_key(Nav::Esc)?)?;
    Ok(case)
}

/// The copy line moved down to `near_px` above the plate or the screen's bottom: the strip must sit above it. A window
/// the gate may not move is a note, never a fail.
pub fn near_bottom(
    gate: &Gate,
    (app, own): (&Opened, bool),
    lines: &Lines,
    cases: &mut Vec<Value>,
) -> Result<Value, String> {
    let line = lines.of(Line::Copy);
    let work =
        kx_platform_windows::screen::work_area(&win::rect(app.hwnd)?).map_err(|e| e.to_string())?;
    let low = floor(place::rect(&work), &gate.clear().boxes, line);
    let dy = lift(line, low, i32::from(gate.g().near_px));
    let r = win::rect(app.hwnd)?;
    let got = moved(
        gate,
        (app, own),
        lines,
        Point {
            x: r.left,
            y: r.top.saturating_add(dy),
        },
        |l| one(gate, app, l, "near_bottom"),
    )?;
    let mut case = match got {
        Ok(case) => case,
        Err(why) => return Ok(json!({ "moved": false, "note": why, "ok": true })),
    };
    let above = case["placed"]["above"] == true;
    case["place"] = json!(case["place"] == true && above);
    let ok = case["seen"] == true && above;
    cases.push(case);
    Ok(json!({ "floor": low, "moved_px": dy, "above": above, "ok": ok }))
}

/// The strip's place on a second screen, when one exists; a note, never a verdict.
pub fn second_screen(gate: &Gate, (app, own): (&Opened, bool), lines: &Lines) -> Value {
    let areas: Vec<Rect> = WinScreens.work_areas().iter().map(|s| s.rect).collect();
    let r = place::rect(&win::rect(app.hwnd).unwrap_or_default());
    let middle = Point {
        x: r.x.saturating_add(r.w / 2),
        y: r.y.saturating_add(r.h / 2),
    };
    let Some(area) = other_screen(&areas, middle) else {
        return json!({ "screens": areas.len(), "note": "one screen" });
    };
    let inset = gate.ctx.cfg.probes.inset_px;
    let to = Point {
        x: area.x.saturating_add(inset),
        y: area.y.saturating_add(inset),
    };
    let got = moved(gate, (app, own), lines, to, |l| {
        one(gate, app, l, "second_screen")
    });
    let placed = match got {
        Ok(Ok(c)) => c["placed"].clone(),
        Ok(Err(why)) => json!({ "note": why }),
        Err(e) => json!({ "error": e }),
    };
    json!({ "screens": areas.len(), "placed": placed })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r(x: i32, y: i32, w: i32, h: i32) -> Rect {
        Rect { x, y, w, h }
    }

    #[test]
    fn the_floor_is_the_plate_under_the_line_else_the_screens_bottom() {
        let (work, line) = (r(0, 0, 1920, 1032), r(100, 200, 400, 24));
        assert_eq!(floor(work, &[], line), 1032);
        let plate = r(0, 700, 1920, 332);
        assert_eq!(floor(work, &[plate], line), 700);
        let aside = r(1000, 700, 900, 332);
        assert_eq!(floor(work, &[aside], line), 1032, "not under the line");
        let above = r(0, 0, 1920, 100);
        assert_eq!(floor(work, &[above], line), 1032, "over the line");
    }

    #[test]
    fn the_line_moves_to_sit_near_px_above_the_floor() {
        let line = r(100, 200, 400, 24);
        assert_eq!(lift(line, 700, 4), 700 - 4 - 224);
        assert_eq!(lift(line, 210, 4), -18, "up when it sits too low");
    }

    #[test]
    fn only_a_window_new_since_the_gate_opened_its_file_is_its_own() {
        let before: HashSet<isize> = [7, 9].into_iter().collect();
        assert!(own(&before, 12), "a new window");
        assert!(
            !own(&before, 9),
            "the owner's window, holding the gate's tab"
        );
    }

    #[test]
    fn a_window_not_the_gates_own_or_maximised_stays_and_says_why() {
        assert_eq!(stays(true, false), None);
        let not_ours = stays(false, false).expect("stays");
        assert!(not_ours.contains("not the gate's own"), "{not_ours}");
        assert_eq!(stays(false, true), Some(not_ours), "not ours comes first");
        assert!(stays(true, true).is_some_and(|w| w.contains("maximised")));
    }

    #[test]
    fn only_our_own_new_window_with_our_tab_in_front_moves() {
        use crate::ownkeys::fake::{Fake, OUR_TAB, OWNER_TAB, hwnd, notepad};
        let app = notepad(None);
        let ours = Fake::new(app.hwnd, &[OUR_TAB]);
        assert_eq!(movable(&ours, &app, (true, false)), Ok(None));
        let theirs = movable(&ours, &app, (false, false)).expect("a reason");
        assert!(theirs.is_some_and(|w| w.contains("not the gate's own")));
        assert!(movable(&ours, &app, (true, true)).is_ok_and(|w| w.is_some()));
        let tab = Fake::new(app.hwnd, &[OWNER_TAB]);
        assert!(
            movable(&tab, &app, (true, false)).is_err(),
            "the owner's tab"
        );
        let away = Fake::new(hwnd(9), &[OUR_TAB]);
        assert!(movable(&away, &app, (true, false)).is_err(), "not in front");
    }

    #[test]
    fn the_second_screen_is_the_first_area_away_from_the_window() {
        let one = [r(0, 0, 1920, 1032)];
        assert_eq!(other_screen(&one, Point { x: 10, y: 10 }), None);
        let two = [r(0, 0, 1920, 1032), r(1920, 0, 1280, 984)];
        assert_eq!(other_screen(&two, Point { x: 10, y: 10 }), Some(two[1]));
        assert_eq!(other_screen(&two, Point { x: 2000, y: 10 }), Some(two[0]));
    }
}
