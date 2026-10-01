//! G11 DPI (roadmap G11; mixed DPI is P14's): at each scale the owner sets, the keyboard draws at its screen's own DPI,
//! never stretched blurry, and keeps its logical size; each run records one scale and a picture, and changes nothing.
#![cfg(windows)]

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
use crate::product::Keyboard;
use crate::{out, shot, win};

/// Suffix of the keyboard's picture.
const PICTURE: &str = "-keyboard.bmp";
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
}

impl Record {
    /// True when Windows draws the window at its screen's DPI with no stretching.
    fn sharp(&self) -> bool {
        self.pmv2 && self.window_dpi == self.screen_dpi
    }
}

/// The verdict over every record so far.
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

/// What window `hwnd` tells about its DPI and size.
#[allow(unsafe_code, reason = "Plain DPI queries on a live window.")]
fn read(hwnd: HWND) -> Result<Record, String> {
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
    })
}

/// Runs G11 on the product, called `name`: records this scale, then judges every scale recorded so far.
pub fn run(ctx: &Ctx, name: &str) -> Result<Value, String> {
    let kb = Keyboard::find()?;
    let record = read(kb.hwnd)?;
    let path = ctx.file(PICTURE);
    shot::keep(&path, &shot::of_window(kb.hwnd)?)?;
    let before = std::fs::read_to_string(ctx.out.join(out::RESULTS)).unwrap_or_default();
    let mut all = records(&before);
    all.push(record.clone());
    let g = &ctx.cfg.g11;
    let v = verdict(&all, &g.scales, g.slack_px);
    Ok(json!({
        "gate": "G11", "keyboard": name, "record": record, "sharp": record.sharp(), "picture": path,
        "verdict": v, "pass": v.pass,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The keyboard as recorded on this PC on 2026-10-01: 125 %, 850 × 257 px, per-monitor v2.
    fn at(scale: u32, size: [f64; 2]) -> Record {
        let dpi = scale * USER_DEFAULT_SCREEN_DPI / PCT;
        Record {
            scale,
            window_dpi: dpi,
            screen_dpi: dpi,
            pmv2: true,
            size,
        }
    }

    #[test]
    fn a_dpi_reads_as_its_scale_and_pixels_as_logical_pixels() {
        assert_eq!([96, 120, 144, 192].map(scale_of), [100, 125, 150, 200]);
        assert!(
            (logical(1063, 120) - 850.4).abs() < 1e-9,
            "this PC's keyboard at 125 %"
        );
        assert!(
            (logical(10, 0) - 960.0).abs() < 1e-9,
            "a DPI of 0 reads as 1, never a division by 0"
        );
    }

    #[test]
    fn every_wanted_scale_sharp_at_one_size_passes() {
        let all = [
            at(100, [680.0, 205.6]),
            at(150, [680.0, 205.3]),
            at(200, [679.5, 205.5]),
        ];
        let v = verdict(&all, &[100, 150, 200], 2.0);
        assert!(v.pass, "{v:?}");
        assert!((v.size_gap_px - 0.5).abs() < 1e-9);
    }

    #[test]
    fn a_missing_scale_a_stretched_window_or_a_changed_size_fails() {
        let wanted = [100, 150, 200];
        let two = [
            at(100, [680.0, 205.0]),
            at(125, [680.0, 205.0]),
            at(150, [680.0, 205.0]),
        ];
        assert_eq!(
            verdict(&two, &wanted, 2.0).missing,
            [200],
            "125 % is only recorded"
        );
        let mut stretched = at(200, [680.0, 205.0]);
        stretched.window_dpi = USER_DEFAULT_SCREEN_DPI;
        let three = [two[0].clone(), two[2].clone(), stretched];
        assert_eq!(verdict(&three, &wanted, 2.0).blurred, [200]);
        let grown = [two[0].clone(), two[2].clone(), at(200, [700.0, 205.0])];
        assert!(!verdict(&grown, &wanted, 2.0).pass);
        let again = [
            grown[0].clone(),
            grown[1].clone(),
            grown[2].clone(),
            at(200, [681.0, 205.0]),
        ];
        assert!(
            verdict(&again, &wanted, 2.0).pass,
            "the latest run of a scale counts"
        );
    }

    #[test]
    fn records_come_from_g11_lines_of_the_results_log_only() {
        let line = json!({ "gate": "G11", "record": at(150, [680.0, 205.0]) }).to_string();
        let log = format!(
            "{{\"gate\":\"G2\"}}\nnot json\n{line}\n{{\"gate\":\"G11\",\"error\":\"x\"}}\n"
        );
        assert_eq!(records(&log), [at(150, [680.0, 205.0])]);
    }
}
