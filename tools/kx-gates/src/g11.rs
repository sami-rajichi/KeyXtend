//! G11 DPI (roadmap G11; mixed DPI is P14's): at each scale the owner sets, the keyboard draws at its screen's own DPI,
//! never stretched blurry, and keeps its logical size; each run records one scale and a picture, and changes nothing.
#![cfg(windows)]

use std::path::Path;
use std::time::UNIX_EPOCH;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Gdi::{MONITOR_DEFAULTTONEAREST, MonitorFromWindow};
use windows::Win32::UI::HiDpi::{
    AreDpiAwarenessContextsEqual, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, GetDpiForMonitor,
    GetDpiForWindow, GetWindowDpiAwarenessContext, MDT_EFFECTIVE_DPI,
};
use windows::Win32::UI::WindowsAndMessaging::USER_DEFAULT_SCREEN_DPI;

use crate::apps::Ctx;
use crate::product::{self, Keyboard};
use crate::{out, shot, win};

/// Suffix of the keyboard's picture, before `shot`'s extension.
const PICTURE: &str = "-keyboard";
/// Percent of a whole.
const PCT: u32 = 100;

/// What one run saw.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Record {
    /// The screen's scale, in percent.
    pub scale: u32,
    /// The DPI the keyboard window draws at.
    pub window_dpi: u32,
    /// The DPI of the keyboard's screen.
    pub screen_dpi: u32,
    /// The window is per-monitor DPI aware (v2).
    pub pmv2: bool,
    /// The window's width and height in logical pixels.
    pub size: [f64; 2],
    /// The product build: its program's last change, in seconds since 1970.
    pub build: u64,
}

impl Record {
    /// True when Windows draws the window at its screen's DPI with no stretching.
    fn sharp(&self) -> bool {
        self.pmv2 && self.window_dpi == self.screen_dpi
    }
}

/// The verdict over one series of records.
#[derive(Debug, PartialEq, Serialize)]
struct Verdict {
    /// Wanted scales with no run yet.
    missing: Vec<u32>,
    /// Scales whose latest run was stretched by Windows.
    blurred: Vec<u32>,
    /// The largest difference in logical width or height between the wanted scales, in px.
    size_gap_px: f64,
    /// True when every wanted scale ran, none was stretched, and the size held.
    pass: bool,
}

/// The latest record of each scale in `scales`, judged together.
fn verdict(records: &[Record], scales: &[u32], slack_px: f64) -> Verdict {
    let latest = |s: &u32| records.iter().rev().find(|r| r.scale == *s);
    let missing = scales
        .iter()
        .filter(|s| latest(s).is_none())
        .copied()
        .collect::<Vec<_>>();
    let got: Vec<&Record> = scales.iter().filter_map(latest).collect();
    let blurred = got
        .iter()
        .filter(|r| !r.sharp())
        .map(|r| r.scale)
        .collect::<Vec<_>>();
    let gap = |i: usize| {
        let (lo, hi) = got
            .iter()
            .map(|r| r.size[i])
            .fold((f64::MAX, f64::MIN), |(a, b), v| (a.min(v), b.max(v)));
        if got.is_empty() { 0.0 } else { hi - lo }
    };
    let size_gap_px = gap(0).max(gap(1));
    let pass = missing.is_empty() && blurred.is_empty() && size_gap_px <= slack_px;
    Verdict {
        missing,
        blurred,
        size_gap_px,
        pass,
    }
}

/// The records of one series, oldest first: the latest record's build only, from its last run at scale `first` on.
fn series(records: &[Record], first: u32) -> Vec<Record> {
    let Some(build) = records.last().map(|r| r.build) else {
        return Vec::new();
    };
    let same: Vec<Record> = records
        .iter()
        .filter(|r| r.build == build)
        .cloned()
        .collect();
    let from = same.iter().rposition(|r| r.scale == first).unwrap_or(0);
    same[from..].to_vec()
}

/// The G11 records in `results`, the lines of `results.jsonl`, oldest first.
fn records(results: &str) -> Vec<Record> {
    let g11 = |v: &Value| v["gate"] == "G11";
    let parsed = results
        .lines()
        .filter_map(|l| serde_json::from_str::<Value>(l).ok());
    parsed
        .filter(g11)
        .filter_map(|v| serde_json::from_value(v["record"].clone()).ok())
        .collect()
}

/// The display scale, in percent, of `dpi`.
fn scale_of(dpi: u32) -> u32 {
    dpi * PCT / USER_DEFAULT_SCREEN_DPI
}

/// `px` physical pixels at `dpi` in logical pixels; a DPI of 0 reads as 1.
fn logical(px: i32, dpi: u32) -> f64 {
    f64::from(px) * f64::from(USER_DEFAULT_SCREEN_DPI) / f64::from(dpi.max(1))
}

/// The build of the program at `exe`: its last change, in seconds since 1970.
fn build_of(exe: &Path) -> Result<u64, String> {
    let changed = std::fs::metadata(exe).and_then(|m| m.modified());
    let changed = changed.map_err(|e| format!("{}: {e}", exe.display()))?;
    Ok(changed
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs())
}

/// What window `hwnd` of product build `build` tells about its DPI and size.
#[allow(unsafe_code, reason = "Plain DPI queries on a live window.")]
fn read(hwnd: HWND, build: u64) -> Result<Record, String> {
    let r = win::rect(hwnd)?;
    let (mut dx, mut dy) = (0, 0);
    // SAFETY: plain queries on a live window and its screen; the locals are filled by the call.
    let (window_dpi, pmv2) = unsafe {
        let monitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
        GetDpiForMonitor(monitor, MDT_EFFECTIVE_DPI, &raw mut dx, &raw mut dy)
            .map_err(|e| format!("screen DPI: {e}"))?;
        let ctx = GetWindowDpiAwarenessContext(hwnd);
        (
            GetDpiForWindow(hwnd),
            AreDpiAwarenessContextsEqual(ctx, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2).as_bool(),
        )
    };
    let size = [
        logical(r.right - r.left, window_dpi),
        logical(r.bottom - r.top, window_dpi),
    ];
    Ok(Record {
        scale: scale_of(dx),
        window_dpi,
        screen_dpi: dx,
        pmv2,
        size,
        build,
    })
}

/// Runs G11 on the product, called `name`: records this scale, then judges this build's runs since its last run at
/// the first scale.
pub fn run(ctx: &Ctx, name: &str) -> Result<Value, String> {
    let kb = Keyboard::find()?;
    let (_, exe) = product::running().ok_or("the product is not running")?;
    let record = read(kb.hwnd, build_of(Path::new(&exe))?)?;
    let path = ctx.file(&shot::named(PICTURE));
    shot::keep(&path, &shot::of_window(kb.hwnd)?)?;
    let before = std::fs::read_to_string(ctx.out.join(out::RESULTS)).unwrap_or_default();
    let mut all = records(&before);
    all.push(record.clone());
    let g = &ctx.cfg.g11;
    let first = g.scales.first().copied().unwrap_or_default();
    let v = verdict(&series(&all, first), &g.scales, g.slack_px);
    Ok(json!({
        "gate": "G11", "keyboard": name, "record": record, "sharp": record.sharp(), "picture": path,
        "verdict": v, "pass": v.pass,
    }))
}

#[cfg(test)]
mod tests;
