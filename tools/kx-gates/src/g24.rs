//! G24: Snip freezes the screen under an overlay above everything; the region picked by two clicks is saved exactly.
#![cfg(windows)]

use std::path::{Path, PathBuf};
use std::time::{Instant, SystemTime};

use serde_json::{Value, json};
use spike_core::capture::{self, Shot};
use spike_core::config::ToolButton;
use spike_core::window::foreground;
use spike_core::{screen, snip, uia};
use windows::Win32::Foundation::{HWND, POINT, RECT};
use windows::Win32::UI::WindowsAndMessaging::FindWindowW;
use windows::core::HSTRING;

use crate::apps::{self, Ctx, Opened};
use crate::facetools::{self, FaceWin};
use crate::simuser::keys_ours;
use crate::win::{self, sleep_ms};
use crate::{launch, probe, scroll};

/// The region to snip inside target-window's text box, in physical pixels.
fn region(ctx: &Ctx, app: &Opened) -> Result<RECT, String> {
    let edit = win::rect(win::focus(app.hwnd))?;
    let dpi = screen::dpi_of(&edit);
    let [x, y, w, h] = ctx.cfg.g24.region_px.map(|v| screen::physical(v, dpi));
    Ok(RECT {
        left: edit.left + x,
        top: edit.top + y,
        right: edit.left + x + w,
        bottom: edit.top + y + h,
    })
}

/// True for a snip file name.
fn is_snip(p: &Path) -> bool {
    let name = p
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    name.starts_with(snip::PREFIX) && p.extension().is_some_and(|e| e == snip::EXT)
}

/// The snip files in `dir` written since `since`, newest last.
fn snips_since(dir: &Path, since: SystemTime) -> Vec<PathBuf> {
    let Ok(all) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut found: Vec<(PathBuf, SystemTime)> = all
        .filter_map(Result::ok)
        .filter_map(|e| Some((e.path(), e.metadata().ok()?.modified().ok()?)))
        .filter(|(p, t)| *t >= since && is_snip(p))
        .collect();
    found.sort_by_key(|(_, t)| *t);
    found.into_iter().map(|(p, _)| p).collect()
}

/// For each point in `pts`, true when the overlay is the window on top there.
fn on_top(over: HWND, pts: &[POINT; 2]) -> [bool; 2] {
    pts.map(|p| win::root_at(p) == over)
}

/// The face's centre and the taskbar's centre, which the overlay must both cover.
#[allow(unsafe_code, reason = "Plain lookup by class name.")]
fn cover_points(ctx: &Ctx, fw: &FaceWin) -> Result<[POINT; 2], String> {
    let class = &ctx.cfg.g24.taskbar_class;
    // SAFETY: plain lookup by class name; the string lives for the call.
    let bar = unsafe { FindWindowW(&HSTRING::from(class), None) }
        .map_err(|e| format!("no taskbar {class}: {e}"))?;
    let (cw, ch) = fw.face.client;
    let face = POINT {
        x: fw.face.origin.x + cw / 2,
        y: fw.face.origin.y + ch / 2,
    };
    Ok([face, facetools::at(uia::centre(&win::rect(bar)?))])
}

/// What the snip gave: whether the overlay covered the face and the taskbar, how soon after the click, the saved picture, and whether it closed.
struct Snipped {
    above: [bool; 2],
    covered_ms: Option<u128>,
    saved: Option<Shot>,
    closed: bool,
}

/// Presses Snip, then clicks the corners of `r` on the overlay; waits for the file and for the overlay to close.
fn snip_region(ctx: &Ctx, fw: &FaceWin, r: &RECT, since: SystemTime) -> Result<Snipped, String> {
    let (g, t, dir) = (&ctx.cfg.g24, &ctx.cfg.timing, ctx.spike.tools.snip_dir());
    let points = cover_points(ctx, fw)?;
    let t0 = Instant::now();
    fw.press(ctx, ToolButton::Snip)?;
    let title = &ctx.spike.tools.overlay_title;
    let over = win::poll_until(g.wait_ms, t.poll_ms, || facetools::shown(title));
    let over = over.ok_or("no overlay")?;
    let covers = || (on_top(over, &points) == [true; 2]).then(|| t0.elapsed().as_millis());
    let covered_ms = win::poll_until(g.wait_ms, t.poll_ms, covers);
    let above = on_top(over, &points);
    let corners = [(r.left, r.top), (r.right - 1, r.bottom - 1)];
    for (x, y) in corners {
        facetools::click_on(over, POINT { x, y })?;
        sleep_ms(t.key_settle_ms);
    }
    let cap = ctx.spike.tools.shot_cap();
    let saved = win::poll_until(g.wait_ms, t.poll_ms, || read_snip(&dir, since, cap));
    let gone = || facetools::shown(title).is_none().then_some(());
    let closed = win::poll_until(g.wait_ms, t.poll_ms, gone).is_some();
    Ok(Snipped {
        above,
        covered_ms,
        saved,
        closed,
    })
}

/// The newest snip since `since`, once its file is complete: the face may still be writing it.
fn read_snip(dir: &Path, since: SystemTime, cap: usize) -> Option<Shot> {
    let bytes = std::fs::read(snips_since(dir, since).pop()?).ok()?;
    capture::from_bmp(&bytes, cap).ok()
}

/// The share of pixels in the `saved` snip that differ from `own`, or why they cannot be compared.
fn compare(saved: Option<&Shot>, own: &Shot) -> Result<f64, String> {
    let s = saved.ok_or("no complete snip file")?;
    let size = |x: &Shot| (x.width, x.height);
    if size(s) != size(own) {
        return Err(format!("the snip is {:?}, not {:?}", size(s), size(own)));
    }
    scroll::changed_share(&s.bgra, &own.bgra).ok_or_else(|| "an empty snip".to_string())
}

/// Types the pattern, snips the region with two clicks, and compares the file with kx-gates' own copy.
fn drive(ctx: &Ctx, fw: &FaceWin, app: &Opened, since: SystemTime) -> Result<Value, String> {
    keys_ours(app)?;
    spike_core::inject::text(&ctx.cfg.g24.text)?;
    sleep_ms(ctx.cfg.probes.settle_ms);
    let r = region(ctx, app)?;
    let own = capture::screen(ctx.spike.tools.shot_cap())?
        .crop(&r)
        .ok_or("the region is off the screen")?;
    let s = snip_region(ctx, fw, &r, since)?;
    let diff = compare(s.saved.as_ref(), &own);
    let back = foreground() == app.hwnd;
    let [face, bar] = s.above;
    let ok = s.covered_ms.is_some() && diff == Ok(0.0) && s.closed && back;
    let diff = diff.map_or_else(|e| json!({ "error": e }), |d| json!(d));
    Ok(json!({
        "ok": ok, "above_face": face, "above_taskbar": bar, "covered_ms": s.covered_ms,
        "size": [own.width, own.height],
        "saved": s.saved.is_some(), "differing_share": diff, "overlay_closed": s.closed, "front_back": back,
    }))
}

/// Closes an overlay a failed run left open, then deletes every snip since `since` and the frozen screen.
fn tidy(ctx: &Ctx, since: SystemTime) -> Vec<String> {
    let mut notes = Vec::new();
    if let Some(over) = facetools::shown(&ctx.spike.tools.overlay_title) {
        // The same spot twice cancels; after one corner it saves instead, and that file goes below.
        let spot = facetools::at(uia::centre(&win::rect(over).unwrap_or_default()));
        for _ in 0..2 {
            let _ = facetools::click_on(over, spot);
            sleep_ms(ctx.cfg.timing.key_settle_ms);
        }
        let still = facetools::shown(&ctx.spike.tools.overlay_title).is_some();
        notes.push(
            if still {
                "a snip overlay is still open"
            } else {
                "closed a snip overlay left open"
            }
            .to_string(),
        );
    }
    let dir = ctx.spike.tools.snip_dir();
    for f in snips_since(&dir, since) {
        if let Err(e) = std::fs::remove_file(&f) {
            notes.push(format!("{}: {e}", f.display()));
        }
    }
    notes.extend(snip::forget_frozen(&dir).err());
    notes
}

/// Runs G24 with face `name` on target-window; snip files and the overlay are cleaned even on failure.
pub fn run(ctx: &Ctx, name: &str) -> Result<Value, String> {
    let fw = FaceWin::parked(ctx, name)?;
    let since = SystemTime::now();
    let app = launch::start_target(ctx)?;
    let got = probe::front(ctx, &app).and_then(|()| drive(ctx, &fw, &app, since));
    let mut left = probe::released();
    left.extend(tidy(ctx, since));
    left.extend(apps::clean_up(ctx, app));
    let mut v = got.map_err(|e| format!("{e}; clean-up: {left:?}"))?;
    v["gate"] = json!("G24");
    v["face"] = json!(name);
    v["pass"] = v["ok"].clone();
    v["clean_up"] = json!(left);
    Ok(v)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shot(width: i32) -> Shot {
        Shot {
            left: 0,
            top: 0,
            width,
            height: 1,
            bgra: vec![0; 4 * usize::try_from(width).expect("a width")],
        }
    }

    #[test]
    fn compare_says_why_it_cannot_compare() {
        assert_eq!(compare(Some(&shot(2)), &shot(2)), Ok(0.0));
        assert!(compare(None, &shot(2)).is_err());
        assert!(compare(Some(&shot(3)), &shot(2)).is_err());
    }

    #[test]
    fn only_snip_files_count() {
        let name = |n: &str| Path::new("d").join(n);
        assert!(is_snip(&name(&format!("{}1.{}", snip::PREFIX, snip::EXT))));
        assert!(!is_snip(&name(&format!("{}1.png", snip::PREFIX))));
        assert!(!is_snip(&name("frozen.bmp")));
    }
}
