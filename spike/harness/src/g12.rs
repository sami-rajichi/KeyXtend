//! G12 ring overlay: with the face's ring test on, clicks under the ring all reach target-window, and the ring keeps 60 Hz.
//!
//! It moves the real mouse and clicks, so it runs only with the owner's yes.

use std::path::Path;

use serde_json::{Value, json};
use spike_core::clock::now_us;
use spike_core::folders;
use spike_core::hold::Pt;
use spike_core::ringstats::{self, FrameReport};
use spike_core::targetlog::Press;
use spike_core::uia::{self, Uia};
use windows::Win32::Foundation::{HWND, POINT, RECT};
use windows::Win32::UI::Accessibility::IUIAutomationElement;

use crate::apps::{self, Ctx};
use crate::facetools::{self, FaceWin};
use crate::hookio::inside;
use crate::tlog::{self, Mouse};
use crate::win::{self, sleep_ms};
use crate::{launch, mouse};

/// `cols` by `rows` points spread over `r`, `inset` px inside its edges, row by row.
fn grid(r: RECT, [cols, rows]: [u32; 2], inset: i32) -> Vec<POINT> {
    let at = |lo: i32, hi: i32, n: u32, i: u32| {
        let (lo, hi) = (lo + inset, hi - inset);
        match n {
            0 | 1 => (lo + hi) / 2,
            _ => lo + (hi - lo) * i as i32 / (n as i32 - 1),
        }
    };
    let mut out = Vec::new();
    for j in 0..rows {
        for i in 0..cols {
            let (x, y) = (at(r.left, r.right, cols, i), at(r.top, r.bottom, rows, j));
            out.push(POINT { x, y });
        }
    }
    out
}

/// How many of `clicked`, in order, target-window logged as a left press and release within `slack` px.
fn logged(got: &[Mouse], clicked: &[POINT], slack: i32) -> usize {
    let near = |m: &Mouse, p: &POINT| (m.x - p.x).abs() <= slack && (m.y - p.y).abs() <= slack;
    let pair = |w: &[Mouse], p: &POINT| {
        w[0].press == Press::LeftDown
            && w[1].press == Press::LeftUp
            && near(&w[0], p)
            && near(&w[1], p)
    };
    let mut rest = got;
    let mut n = 0;
    for p in clicked {
        if let Some(i) = rest.windows(2).position(|w| pair(w, p)) {
            n += 1;
            rest = &rest[i + 2..];
        }
    }
    n
}

/// True when `ring` shows above `target` in `front_first` and its box holds `p`.
fn ring_over(front_first: &[HWND], ring: HWND, target: HWND, p: POINT) -> bool {
    let at = |w: HWND| front_first.iter().position(|&o| o == w);
    let above = matches!((at(ring), at(target)), (Some(r), Some(t)) if r < t);
    above && win::rect(ring).is_ok_and(|r| inside(&r, Pt { x: p.x, y: p.y }))
}

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

/// What the clicks found.
#[derive(Default)]
struct Tally {
    /// Points clicked, in order.
    clicked: Vec<POINT>,
    /// Clicks with the ring shown above target-window and over the point.
    covered: usize,
    /// Clicks where Windows put our ring, not target-window, at the point.
    blocked: usize,
}

/// Clicks each of `points`; before each click, waits for the ring to follow and checks nothing else is in the way.
fn click_all(ctx: &Ctx, ring: HWND, target: HWND, points: &[POINT]) -> Result<Tally, String> {
    let (g, t) = (&ctx.cfg.g12, &ctx.cfg.timing);
    let mut tally = Tally::default();
    for &p in points {
        mouse::move_to(p)?;
        // A hand on the mouse may pull the ring away, so the pointer is put back until the ring follows.
        let follow = || {
            if ring_over(&win::top_windows(), ring, target, p) {
                return Some(());
            }
            let _ = mouse::move_to(p);
            None
        };
        tally.covered += usize::from(win::poll_until(g.follow_ms, t.poll_ms, follow).is_some());
        let top = win::root_at(p);
        if top != target && top != ring {
            return Err(format!("{} covers the test point", win::describe(top)));
        }
        tally.blocked += usize::from(top == ring);
        mouse::click_at(p)?;
        tally.clicked.push(p);
        sleep_ms(g.settle_ms);
    }
    Ok(tally)
}

/// Removes the stats file and any part file, and their folder when it is empty and not the app-data folder itself.
fn drop_stats(file: &Path) {
    let _ = std::fs::remove_file(file);
    let _ = std::fs::remove_file(ringstats::part_file(file));
    let root = folders::local_data().ok();
    if let Some(dir) = file.parent().filter(|d| root.as_deref() != Some(*d)) {
        let _ = std::fs::remove_dir(dir);
    }
}

/// Waits for the face to write whole frame stats, reads them, then drops the file.
fn take_stats(ctx: &Ctx, file: &Path) -> Result<FrameReport, String> {
    let t = &ctx.cfg.timing;
    let parse = |text: String| serde_json::from_str::<FrameReport>(&text).ok();
    let read = || std::fs::read_to_string(file).ok().and_then(parse);
    let got = win::poll_until(t.read_wait_ms, t.poll_ms, read);
    drop_stats(file);
    got.ok_or_else(|| format!("no frame stats at {}", file.display()))
}

/// Sets the ring test on or off with the strip's switch; true once the ring shows as asked.
fn flip(ctx: &Ctx, switch: &IUIAutomationElement, pid: u32, on: bool) -> bool {
    let (t, title) = (&ctx.cfg.timing, &ctx.spike.ring.title);
    let showing = || facetools::shown_by(pid, title).is_some();
    if showing() == on {
        return true;
    }
    let wait = if on {
        ctx.cfg.g12.open_ms
    } else {
        t.close_wait_ms
    };
    let as_asked = || (showing() == on).then_some(());
    uia::invoke(switch).is_ok() && win::poll_until(wait, t.poll_ms, as_asked).is_some()
}

/// Clicks every grid point under the running ring, then turns it off and reads what it measured.
fn drive(
    ctx: &Ctx,
    switch: &IUIAutomationElement,
    pid: u32,
    target: HWND,
) -> Result<Value, String> {
    let (g, file) = (&ctx.cfg.g12, ctx.spike.ring.stats_path()?);
    let all = grid(text_box(target)?, g.grid, g.inset_px);
    let points: Vec<POINT> = all
        .iter()
        .copied()
        .filter(|&p| win::root_at(p) == target)
        .collect();
    let opened = flip(ctx, switch, pid, true);
    let Some(ring) = facetools::shown_by(pid, &ctx.spike.ring.title).filter(|_| opened) else {
        let off = flip(ctx, switch, pid, false);
        drop_stats(&file);
        return Err(format!("the ring did not open; turned off again: {off}"));
    };
    let since = now_us();
    let clicks = click_all(ctx, ring, target, &points);
    let closed = flip(ctx, switch, pid, false);
    // Read even after a failed click, so the stats file never stays behind.
    let stats = take_stats(ctx, &file).map_err(|e| format!("{e}; ring closed: {closed}"));
    let t = clicks.map_err(|e| format!("{e}; ring closed: {closed}"))?;
    let f = stats?;
    sleep_ms(g.settle_ms);
    let reached = logged(&tlog::mouse_since(ctx, since)?, &t.clicked, g.slack_px);
    let smooth = f.fps >= g.min_fps && f.p99_ms <= g.p99_ms && !f.capped;
    let (n, left_out) = (t.clicked.len(), all.len() - points.len());
    let all_hit = left_out == 0 && reached == n && t.covered == n && t.blocked == 0;
    Ok(json!({
        "gate": "G12", "clicks": n, "reached": reached, "covered": t.covered, "blocked": t.blocked,
        "left_out": left_out, "frames": f, "closed": closed, "pass": closed && smooth && all_hit,
    }))
}

/// Runs G12 on face `name`: the face is parked, target-window opens, the ring test runs over it, then both close.
pub fn run(ctx: &Ctx, name: &str) -> Result<Value, String> {
    let uia = Uia::new()?;
    let face = FaceWin::parked(ctx, name)?.hwnd;
    let pid = win::owner(face).0;
    let switch = uia
        .named(face, &ctx.spike.ring.label)
        .ok_or("the strip has no ring test switch")?;
    // A test left on is stopped first and its stats dropped.
    if !flip(ctx, &switch, pid, false) {
        return Err("a ring test left on did not stop".into());
    }
    drop_stats(&ctx.spike.ring.stats_path()?);
    let target = launch::start_target(ctx)?;
    println!("G12 {name}: {}", win::describe(target.hwnd));
    let out = drive(ctx, &switch, pid, target.hwnd);
    let left_open = apps::clean_up(ctx, target);
    let mut v = out.map_err(|e| format!("{e}; left open: {left_open:?}"))?;
    v["face"] = json!(name);
    v["left_open"] = json!(left_open);
    Ok(v)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::probecfg::G12;

    fn press(press: Press, x: i32, y: i32) -> Mouse {
        Mouse { press, us: 0, x, y }
    }

    fn click(x: i32, y: i32) -> [Mouse; 2] {
        [press(Press::LeftDown, x, y), press(Press::LeftUp, x, y)]
    }

    fn hwnd(v: isize) -> HWND {
        HWND(v as *mut _)
    }

    #[test]
    fn the_grid_spans_the_box_inside_its_inset_row_by_row() {
        let r = RECT {
            left: 100,
            top: 50,
            right: 500,
            bottom: 350,
        };
        let g = grid(r, [3, 2], 50);
        let xy: Vec<(i32, i32)> = g.iter().map(|p| (p.x, p.y)).collect();
        assert_eq!(
            xy,
            [
                (150, 100),
                (300, 100),
                (450, 100),
                (150, 300),
                (300, 300),
                (450, 300)
            ]
        );
        assert_eq!(
            grid(r, [1, 1], 0)
                .iter()
                .map(|p| (p.x, p.y))
                .collect::<Vec<_>>(),
            [(300, 200)]
        );
    }

    #[test]
    fn a_lost_click_counts_once_and_later_clicks_still_match() {
        let pts = [
            POINT { x: 10, y: 10 },
            POINT { x: 50, y: 10 },
            POINT { x: 90, y: 10 },
        ];
        let mut got: Vec<Mouse> = [click(10, 10), click(91, 10)].concat();
        assert_eq!(logged(&got, &pts, 1), 2, "the middle click is lost");
        got.insert(2, press(Press::RightDown, 50, 10));
        assert_eq!(logged(&got, &pts, 1), 2, "a right press is not a click");
        assert_eq!(
            logged(&click(12, 10), &pts[..1], 1),
            0,
            "too far from the point"
        );
    }

    #[test]
    fn a_press_without_its_release_does_not_count() {
        let got = [
            press(Press::LeftDown, 10, 10),
            press(Press::RightDown, 10, 10),
        ];
        assert_eq!(logged(&got, &[POINT { x: 10, y: 10 }], 1), 0);
    }

    #[test]
    fn a_ring_covers_only_while_it_shows_above_the_target() {
        let (ring, target, other) = (hwnd(1), hwnd(2), hwnd(3));
        let p = POINT { x: 5, y: 5 };
        assert!(
            !ring_over(&[target, ring], ring, target, p),
            "behind the target"
        );
        assert!(!ring_over(&[other, target], ring, target, p), "hidden");
    }

    #[test]
    fn the_configured_grid_holds_the_roadmaps_200_clicks_and_bad_settings_are_refused() {
        let g = crate::config::load().expect("harness.toml loads").g12;
        assert_eq!(g.grid[0] * g.grid[1], 200);
        assert_eq!(g.check(), Ok(()));
        let bad: [fn(&mut G12); 5] = [
            |b| b.grid[1] = 0,
            |b| b.inset_px = -1,
            |b| b.slack_px = -1,
            |b| b.min_fps = f64::NAN,
            |b| b.p99_ms = 0.0,
        ];
        for f in bad {
            let mut b = g.clone();
            f(&mut b);
            assert!(b.check().is_err());
        }
    }
}
