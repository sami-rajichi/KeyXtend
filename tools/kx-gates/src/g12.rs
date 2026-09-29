//! G12 ring overlay: with the face's ring test on, clicks under the ring all reach target-window, and the ring keeps 60 Hz.
//!
//! It moves the real mouse and clicks, so it runs only with the owner's yes.
#![cfg(windows)]

use std::path::Path;

use kx_test_support::ringstats::{self, FrameReport};
use serde_json::{Value, json};
use spike_core::clock::now_us;
use spike_core::folders;
use spike_core::uia::{self, Uia};
use windows::Win32::Foundation::{HWND, POINT, RECT};
use windows::Win32::UI::Accessibility::IUIAutomationElement;

use crate::apps::{self, Ctx};
use crate::facetools::{self, FaceWin};
use crate::g12pts::{grid, logged, ring_over};
use crate::out::say;
use crate::tlog;
use crate::win::{self, sleep_ms};
use crate::{launch, mouse};

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
    say!("G12 {name}: {}", win::describe(target.hwnd));
    let out = drive(ctx, &switch, pid, target.hwnd);
    let left_open = apps::clean_up(ctx, target);
    let mut v = out.map_err(|e| format!("{e}; left open: {left_open:?}"))?;
    v["face"] = json!(name);
    v["left_open"] = json!(left_open);
    Ok(v)
}

#[cfg(test)]
mod tests {
    use crate::probecfg::G12;

    #[test]
    fn the_configured_grid_holds_the_roadmaps_200_clicks_and_bad_settings_are_refused() {
        let g = crate::config::load().expect("kx-gates.toml loads").g12;
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
