//! G13 themes (roadmap G13): while the owner changes dark mode, the accent and high contrast in Windows, it times how
//! soon the keyboard redraws and saves a picture of each look, which the owner judges by eye; it changes no setting.
#![cfg(windows)]

use std::path::PathBuf;
use std::time::{Duration, Instant};

use kx_platform::ui::{SysLook, SystemLook};
use kx_platform_windows::WinLook;
use serde::Serialize;
use serde_json::{Value, json};
use spike_core::capture::Shot;
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Dwm::{
    DWM_SYSTEMBACKDROP_TYPE, DWMSBT_AUTO, DWMSBT_MAINWINDOW, DWMSBT_NONE, DWMSBT_TABBEDWINDOW,
    DWMSBT_TRANSIENTWINDOW, DWMWA_SYSTEMBACKDROP_TYPE, DwmGetWindowAttribute,
};

use crate::apps::Ctx;
use crate::budget::ms;
use crate::out::say;
use crate::product::Keyboard;
use crate::shot::{self, of_window as picture};
use crate::win::{size32, sleep_ms};

/// Suffix of picture `{n}` of a run.
const LOOK: &str = "-look-";
/// The backdrop's name when DWM gives none or one this table lacks.
const UNKNOWN: &str = "unknown";

/// Each backdrop Windows may draw behind a window, by name (DWM's own table).
const BACKDROPS: [(DWM_SYSTEMBACKDROP_TYPE, &str); 5] = [
    (DWMSBT_AUTO, "auto"),
    (DWMSBT_NONE, "none"),
    (DWMSBT_MAINWINDOW, "mica"),
    (DWMSBT_TRANSIENTWINDOW, "acrylic"),
    (DWMSBT_TABBEDWINDOW, "tabbed"),
];

/// The parts of Windows' look G13 names: dark mode, the accent colour and high contrast.
const DARK: &str = "dark";
const ACCENT: &str = "accent";
const CONTRAST: &str = "contrast";
/// Every part G13 names, which `[g13] needs` picks from.
pub const KINDS: [&str; 3] = [DARK, ACCENT, CONTRAST];

/// What changed from look `a` to look `b`, by the names in `KINDS`.
fn what_changed(a: &SysLook, b: &SysLook) -> Vec<&'static str> {
    let parts = [
        (DARK, a.dark != b.dark),
        (ACCENT, a.accent != b.accent),
        (CONTRAST, a.contrast != b.contrast),
    ];
    parts
        .iter()
        .filter(|(_, changed)| *changed)
        .map(|(name, _)| *name)
        .collect()
}

/// One change of Windows' look and how the keyboard followed it.
#[derive(Debug, Clone, Serialize)]
struct Change {
    what: Vec<&'static str>,
    dark: bool,
    contrast: bool,
    /// From when the gate saw the change (up to one poll after it) to the keyboard's first changed picture.
    followed_ms: Option<f64>,
    backdrop: &'static str,
    picture: PathBuf,
}

/// True when each part in `needs` changed at least once and the keyboard followed every change within `follow_ms`.
fn passed(changes: &[Change], needs: &[String], follow_ms: f64) -> bool {
    let seen = |n: &String| changes.iter().any(|c| c.what.contains(&n.as_str()));
    let in_time = |c: &Change| c.followed_ms.is_some_and(|f| f <= follow_ms);
    !changes.is_empty() && needs.iter().all(seen) && changes.iter().all(in_time)
}

/// The backdrop Windows draws behind window `hwnd`.
#[allow(unsafe_code, reason = "Fills a local of the attribute's size.")]
fn backdrop(hwnd: HWND) -> &'static str {
    let mut v = DWM_SYSTEMBACKDROP_TYPE::default();
    // SAFETY: `v` is a live local of the size the attribute needs.
    let got = unsafe {
        DwmGetWindowAttribute(
            hwnd,
            DWMWA_SYSTEMBACKDROP_TYPE,
            (&raw mut v).cast(),
            size32::<DWM_SYSTEMBACKDROP_TYPE>(),
        )
    };
    let named = BACKDROPS.iter().find(|(t, _)| *t == v).map(|(_, n)| *n);
    got.ok().and(named).unwrap_or(UNKNOWN)
}

/// Copies the keyboard until it differs from `before`; the ms it took, none after `wait_ms`.
fn follow(hwnd: HWND, before: &Shot, wait_ms: u64, poll_ms: u64) -> Result<Option<f64>, String> {
    let start = Instant::now();
    loop {
        if picture(hwnd)? != *before {
            return Ok(Some(ms(start.elapsed())));
        }
        if start.elapsed() >= Duration::from_millis(wait_ms) {
            return Ok(None);
        }
        sleep_ms(poll_ms);
    }
}

/// Saves `shot` as picture number `n` of this run.
fn save(ctx: &Ctx, shot: &Shot, n: usize) -> Result<PathBuf, String> {
    let path = ctx.file(&shot::named(&format!("{LOOK}{n}")));
    shot::keep(&path, shot)?;
    Ok(path)
}

/// `what` with each part of `more` it lacks added at the end.
fn joined(mut what: Vec<&'static str>, more: &[&'static str]) -> Vec<&'static str> {
    for part in more {
        if !what.contains(part) {
            what.push(part);
        }
    }
    what
}

/// Follows change number `n` from look `was` to `now` against picture `before`, then takes the settled picture and
/// reads the look again, so a part that changed meanwhile joins this change; the change, that picture and that look.
fn record(
    ctx: &Ctx,
    hwnd: HWND,
    before: &Shot,
    (was, now): (&SysLook, SysLook),
    n: usize,
) -> Result<(Change, Shot, SysLook), String> {
    let (g, poll) = (&ctx.cfg.g13, ctx.cfg.timing.poll_ms);
    let followed_ms = follow(hwnd, before, g.follow_ms, poll)?;
    sleep_ms(g.follow_ms);
    let shot = picture(hwnd)?;
    let after = WinLook.read();
    let what = joined(what_changed(was, &now), &what_changed(&now, &after));
    let picture = save(ctx, &shot, n)?;
    say!(
        "G13: {} changed; followed after {followed_ms:?} ms",
        what.join(", ")
    );
    let change = Change {
        what,
        dark: after.dark,
        contrast: after.contrast.is_some(),
        followed_ms,
        backdrop: backdrop(hwnd),
        picture,
    };
    Ok((change, shot, after))
}

/// Watches Windows' look and the keyboard until the watch ends or enough changes were seen.
fn watch(ctx: &Ctx, hwnd: HWND) -> Result<Vec<Change>, String> {
    let (g, poll) = (&ctx.cfg.g13, ctx.cfg.timing.poll_ms);
    let end = Instant::now() + Duration::from_secs(g.watch_secs);
    let (mut before, mut look) = (picture(hwnd)?, WinLook.read());
    let mut changes = Vec::new();
    while Instant::now() < end && changes.len() < g.changes {
        // The picture comes first: one taken after an unchanged read may already show a change made in between.
        let shot = picture(hwnd)?;
        let now = WinLook.read();
        if what_changed(&look, &now).is_empty() {
            before = shot;
            sleep_ms(poll);
            continue;
        }
        let (change, shot, after) = record(ctx, hwnd, &before, (&look, now), changes.len() + 1)?;
        changes.push(change);
        (before, look) = (shot, after);
    }
    Ok(changes)
}

/// Runs G13 on the product, called `name`; the owner changes Windows' look while it watches.
pub fn run(ctx: &Ctx, name: &str) -> Result<Value, String> {
    let kb = Keyboard::find()?;
    let first = save(ctx, &picture(kb.hwnd)?, 0)?;
    let g = &ctx.cfg.g13;
    say!(
        "G13: change dark mode, the accent colour and high contrast in Windows now; watching {} s",
        g.watch_secs
    );
    let changes = watch(ctx, kb.hwnd)?;
    #[allow(
        clippy::cast_precision_loss,
        reason = "A wait in ms is far below 2^53."
    )]
    let pass = passed(&changes, &g.needs, g.follow_ms as f64);
    Ok(json!({
        "gate": "G13", "keyboard": name, "backdrop": backdrop(kb.hwnd), "first": first, "changes": changes,
        "pass": pass,
    }))
}

#[cfg(test)]
mod tests;
