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
/// Picture files' extension.
const BMP: &str = ".bmp";
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
    let path = ctx.file(&format!("{LOOK}{n}{BMP}"));
    shot::keep(&path, shot)?;
    Ok(path)
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
        let what = what_changed(&look, &now);
        if what.is_empty() {
            before = shot;
            sleep_ms(poll);
            continue;
        }
        let followed_ms = follow(hwnd, &before, g.follow_ms, poll)?;
        sleep_ms(g.follow_ms);
        before = picture(hwnd)?;
        let picture = save(ctx, &before, changes.len() + 1)?;
        let (dark, contrast, backdrop) = (now.dark, now.contrast.is_some(), backdrop(hwnd));
        say!(
            "G13: {} changed; followed after {followed_ms:?} ms",
            what.join(", ")
        );
        changes.push(Change {
            what,
            dark,
            contrast,
            followed_ms,
            backdrop,
            picture,
        });
        look = now;
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
mod tests {
    use kx_platform::ui::Accent;

    use super::*;

    fn change(followed_ms: Option<f64>) -> Change {
        of(&[DARK], followed_ms)
    }

    fn of(what: &[&'static str], followed_ms: Option<f64>) -> Change {
        Change {
            what: what.to_vec(),
            dark: true,
            contrast: false,
            followed_ms,
            backdrop: "none",
            picture: PathBuf::new(),
        }
    }

    fn needs(kinds: &[&str]) -> Vec<String> {
        kinds.iter().map(ToString::to_string).collect()
    }

    #[test]
    fn each_part_of_windows_look_is_named_when_it_changes() {
        let light = SysLook::default();
        let dark = SysLook {
            dark: true,
            ..light
        };
        assert_eq!(what_changed(&light, &dark), ["dark"]);
        let accent = SysLook {
            accent: Some(Accent {
                on_light: [0, 95, 184, 255],
                on_dark: [96, 205, 255, 255],
            }),
            ..dark
        };
        assert_eq!(what_changed(&light, &accent), ["dark", "accent"]);
        assert!(what_changed(&accent, &accent).is_empty());
        let moved = SysLook {
            anim_off: true,
            ..light
        };
        assert!(
            what_changed(&light, &moved).is_empty(),
            "animations are not G13's"
        );
    }

    #[test]
    fn g13_passes_when_every_change_was_followed_in_time() {
        let dark = needs(&[DARK]);
        assert!(passed(
            &[change(Some(120.0)), change(Some(0.0))],
            &dark,
            1000.0
        ));
        assert!(
            !passed(&[change(Some(120.0)), change(Some(1500.0))], &dark, 1000.0),
            "too slow"
        );
        assert!(!passed(&[change(None)], &dark, 1000.0), "never redrawn");
        assert!(!passed(&[], &dark, 1000.0), "the owner changed nothing");
    }

    #[test]
    fn g13_fails_until_each_needed_part_of_the_look_changed() {
        let all = needs(&KINDS);
        let two = [of(&[DARK], Some(100.0)), of(&[DARK, ACCENT], Some(200.0))];
        assert!(!passed(&two, &all, 1000.0), "contrast never changed");
        let three = [two[0].clone(), two[1].clone(), of(&[CONTRAST], Some(300.0))];
        assert!(passed(&three, &all, 1000.0));
    }

    #[test]
    fn every_backdrop_the_product_sets_has_a_name() {
        let names: Vec<&str> = BACKDROPS.iter().map(|(_, n)| *n).collect();
        for kind in ["none", "mica", "acrylic"] {
            assert!(names.contains(&kind), "{kind}");
        }
    }
}
