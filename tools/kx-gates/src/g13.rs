//! G13 themes (roadmap G13): while the owner changes dark mode, the accent and high contrast in Windows, it times how
//! soon the keys redraw and saves a picture of each look, which the owner judges by eye; it changes no setting.
//! Only the keys' solid faces are compared, never the window's clear margin, the plate's corners or a backdrop.
#![cfg(windows)]

use std::path::PathBuf;
use std::time::{Duration, Instant};

use kx_platform::ui::{SysLook, SystemLook};
use kx_platform_windows::WinLook;
use kx_theme::Themes;
use serde::Serialize;
use serde_json::{Value, json};
use spike_core::capture::Shot;
use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::Graphics::Dwm::{
    DWM_SYSTEMBACKDROP_TYPE, DWMSBT_AUTO, DWMSBT_MAINWINDOW, DWMSBT_NONE, DWMSBT_TABBEDWINDOW,
    DWMSBT_TRANSIENTWINDOW, DWMWA_SYSTEMBACKDROP_TYPE, DwmGetWindowAttribute,
};

use crate::apps::Ctx;
use crate::board;
use crate::budget::ms;
use crate::mouse::round_px;
use crate::out::say;
use crate::product::Keyboard;
use crate::shot::{self, of_window as picture};
use crate::uilook::{self, FOLLOW_KEY, THEME_KEY, UiLook};
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
/// A key inset from each side by half its size has nothing left.
pub const MAX_INSET: f64 = 0.5;

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
    /// From when the gate saw the change (up to one poll after it) to the first change of the key faces.
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

/// Why G13 cannot run with look `ui`: `needs` asks for the accent, which the keyboard then never takes.
fn accent_blocked(ui: &UiLook, themes: &Themes, needs: &[String]) -> Option<String> {
    if !needs.iter().any(|n| n == ACCENT) || ui.takes_accent(themes) {
        return None;
    }
    let can: Vec<&str> = themes
        .list
        .iter()
        .filter(|t| t.look.accent)
        .map(|t| t.look.id.as_str())
        .collect();
    let (theme, follow) = (&ui.theme, ui.follow_accent);
    Some(format!(
        "G13 needs the keyboard to take the accent: set [ui] {FOLLOW_KEY} = true and {THEME_KEY} = one of {can:?} \
         first, then put back {FOLLOW_KEY} = {follow} and {THEME_KEY} = \"{theme}\" after the run"
    ))
}

/// Box `r` inset by `share` of its width and height on every side; none when nothing is left.
fn inner(r: &RECT, share: f64) -> Option<RECT> {
    let dx = round_px(f64::from(r.right - r.left) * share);
    let dy = round_px(f64::from(r.bottom - r.top) * share);
    let face = RECT {
        left: r.left + dx,
        top: r.top + dy,
        right: r.right - dx,
        bottom: r.bottom - dy,
    };
    (face.right > face.left && face.bottom > face.top).then_some(face)
}

/// The keys' solid faces on keyboard `kb`: each key's box inset by `share`, so no margin, corner or backdrop shows.
fn faces(kb: &Keyboard, share: f64) -> Result<Vec<RECT>, String> {
    let ids = board::ids()?;
    let faces: Vec<RECT> = (ids.iter())
        .filter_map(|id| kb.rect(id).ok())
        .filter_map(|r| inner(&r, share))
        .collect();
    if faces.is_empty() {
        return Err("no key face to compare".to_string());
    }
    Ok(faces)
}

/// The key faces `faces` cut from picture `all`.
fn cut(all: &Shot, faces: &[RECT]) -> Result<Vec<Shot>, String> {
    let piece = |f: &RECT| {
        let why = || "a key face lies outside the keyboard's picture".to_string();
        all.crop(f).ok_or_else(why)
    };
    faces.iter().map(piece).collect()
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

/// The keyboard G13 watches: its window and its keys' faces.
struct Watched<'a> {
    ctx: &'a Ctx<'a>,
    hwnd: HWND,
    faces: Vec<RECT>,
}

impl Watched<'_> {
    /// The keyboard's picture and its key faces now.
    fn now(&self) -> Result<(Shot, Vec<Shot>), String> {
        let all = picture(self.hwnd)?;
        let keys = cut(&all, &self.faces)?;
        Ok((all, keys))
    }

    /// Copies the key faces until they differ from `before`; the ms it took, none after `wait_ms`.
    fn follow(&self, before: &[Shot], wait_ms: u64) -> Result<Option<f64>, String> {
        let start = Instant::now();
        loop {
            if self.now()?.1 != before {
                return Ok(Some(ms(start.elapsed())));
            }
            if start.elapsed() >= Duration::from_millis(wait_ms) {
                return Ok(None);
            }
            sleep_ms(self.ctx.cfg.timing.poll_ms);
        }
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

/// Follows change number `n` from look `was` to `now` against key faces `before`, then takes the settled picture and
/// reads the look again, so a part that changed meanwhile joins this change; the change, its key faces and that look.
fn record(
    w: &Watched,
    before: &[Shot],
    (was, now): (&SysLook, SysLook),
    n: usize,
) -> Result<(Change, Vec<Shot>, SysLook), String> {
    let (ctx, g) = (w.ctx, &w.ctx.cfg.g13);
    let followed_ms = w.follow(before, g.follow_ms)?;
    sleep_ms(g.follow_ms);
    let (shot, keys) = w.now()?;
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
        backdrop: backdrop(w.hwnd),
        picture,
    };
    Ok((change, keys, after))
}

/// Watches Windows' look and the keyboard until the watch ends or enough changes were seen.
fn watch(w: &Watched) -> Result<Vec<Change>, String> {
    let (g, poll) = (&w.ctx.cfg.g13, w.ctx.cfg.timing.poll_ms);
    let end = Instant::now() + Duration::from_secs(g.watch_secs);
    let (mut before, mut look) = (w.now()?.1, WinLook.read());
    let mut changes = Vec::new();
    while Instant::now() < end && changes.len() < g.changes {
        // The picture comes first: one taken after an unchanged read may already show a change made in between.
        let keys = w.now()?.1;
        let now = WinLook.read();
        if what_changed(&look, &now).is_empty() {
            before = keys;
            sleep_ms(poll);
            continue;
        }
        let (change, keys, after) = record(w, &before, (&look, now), changes.len() + 1)?;
        changes.push(change);
        (before, look) = (keys, after);
    }
    Ok(changes)
}

/// Runs G13 on the product, called `name`; the owner changes Windows' look while it watches. It stops at the start
/// when the running copy's `[ui]` settings keep the accent off the keys.
pub fn run(ctx: &Ctx, name: &str) -> Result<Value, String> {
    let (g, ui) = (&ctx.cfg.g13, uilook::running()?);
    let themes = Themes::shipped().map_err(|e| format!("theme {e}"))?;
    if let Some(why) = accent_blocked(&ui, &themes, &g.needs) {
        return Err(why);
    }
    let kb = Keyboard::find()?;
    let faces = faces(&kb, g.key_inset)?;
    let first = save(ctx, &picture(kb.hwnd)?, 0)?;
    say!(
        "G13: change dark mode, the accent colour and high contrast in Windows now; watching {} s. Afterwards put \
         Windows' look back, and any [ui] setting you changed for this run.",
        g.watch_secs
    );
    let changes = watch(&Watched {
        ctx,
        hwnd: kb.hwnd,
        faces,
    })?;
    #[allow(
        clippy::cast_precision_loss,
        reason = "A wait in ms is far below 2^53."
    )]
    let pass = passed(&changes, &g.needs, g.follow_ms as f64);
    Ok(json!({
        "gate": "G13", "keyboard": name, "ui": ui, "backdrop": backdrop(kb.hwnd), "first": first, "changes": changes,
        "pass": pass,
    }))
}

#[cfg(test)]
mod tests;
