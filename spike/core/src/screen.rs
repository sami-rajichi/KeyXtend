//! The usable screen around a box: its monitor's work area without the taskbar, and its DPI.

use windows::Win32::Foundation::{POINT, RECT};
use windows::Win32::Graphics::Gdi::{
    GetMonitorInfoW, MONITOR_DEFAULTTONEAREST, MONITORINFO, MonitorFromRect,
};
use windows::Win32::UI::HiDpi::{GetDpiForMonitor, MDT_EFFECTIVE_DPI};
use windows::Win32::UI::WindowsAndMessaging::{
    GetCursorPos, GetWindowRect, USER_DEFAULT_SCREEN_DPI,
};

use crate::hold::Pt;
use crate::window::own_titled;

/// Error when the window the bubble belongs to is not shown.
const NOT_SHOWN: &str = "the keyboard window is not shown";

/// The work area of the monitor nearest to `r`; physical pixels in a per-monitor DPI-aware thread, like UIA boxes.
pub fn work_area(r: &RECT) -> Result<RECT, String> {
    let mut info = MONITORINFO {
        cbSize: size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };
    // SAFETY: plain queries; `info` is sized for the call.
    unsafe {
        let mon = MonitorFromRect(r, MONITOR_DEFAULTTONEAREST);
        if !GetMonitorInfoW(mon, &mut info).as_bool() {
            return Err("GetMonitorInfo failed".to_string());
        }
    }
    Ok(info.rcWork)
}

/// The DPI of the monitor nearest to `r`, or the default 96 when Windows cannot say.
pub fn dpi_of(r: &RECT) -> u32 {
    let (mut x, mut y) = (0, 0);
    // SAFETY: plain queries into locals.
    let ok = unsafe {
        let mon = MonitorFromRect(r, MONITOR_DEFAULTTONEAREST);
        GetDpiForMonitor(mon, MDT_EFFECTIVE_DPI, &mut x, &mut y).is_ok()
    };
    if ok && x > 0 {
        x
    } else {
        USER_DEFAULT_SCREEN_DPI
    }
}

/// `px` logical pixels at `dpi`, in physical pixels.
pub fn physical(px: f32, dpi: u32) -> i32 {
    (px * dpi as f32 / USER_DEFAULT_SCREEN_DPI as f32).round() as i32
}

/// Top-left of a box of `size` centred `gap` above the bottom of `work`; a box wider than `work` starts at its left edge.
pub fn bottom_centre(work: &RECT, size: (i32, i32), gap: i32) -> Pt {
    Pt {
        x: work.left + ((work.right - work.left - size.0) / 2).max(0),
        y: work.bottom - size.1 - gap,
    }
}

/// Where a `d`-wide bubble goes: `inset` inside the bottom-left or bottom-right corner of `work`, never off it.
pub fn corner_spot(work: &RECT, left: bool, d: i32, inset: i32) -> Pt {
    let x = if left {
        work.left + inset
    } else {
        work.right - d - inset
    };
    Pt {
        x: x.min(work.right - d).max(work.left),
        y: (work.bottom - d - inset).max(work.top),
    }
}

/// Where a window `w` by `h` logical pixels starts, in physical pixels: the bottom centre of the primary work area.
pub fn start_spot(w: f32, h: f32) -> Result<Pt, String> {
    // The primary monitor is the one holding (0, 0).
    let origin = RECT::default();
    let (work, dpi) = (work_area(&origin)?, dpi_of(&origin));
    Ok(bottom_centre(
        &work,
        (physical(w, dpi), physical(h, dpi)),
        0,
    ))
}

/// Where the minimise bubble goes on the screen of our window `title`: a bottom corner, `left` or right.
/// `[d, inset]` are its diameter and inset in logical pixels.
pub fn bubble_for(title: &str, left: bool, [d, inset]: [f32; 2]) -> Result<Pt, String> {
    let hwnd = own_titled(title).ok_or(NOT_SHOWN)?;
    let mut r = RECT::default();
    // SAFETY: `hwnd` is one of our live windows and `r` is a local.
    unsafe { GetWindowRect(hwnd, &mut r) }.map_err(|e| format!("GetWindowRect: {e}"))?;
    let (work, dpi) = (work_area(&r)?, dpi_of(&r));
    Ok(corner_spot(
        &work,
        left,
        physical(d, dpi),
        physical(inset, dpi),
    ))
}

/// Where the mouse pointer is, in physical pixels.
pub fn cursor() -> Result<Pt, String> {
    let mut p = POINT::default();
    // SAFETY: a plain query into a local.
    unsafe { GetCursorPos(&mut p) }.map_err(|e| format!("GetCursorPos: {e}"))?;
    Ok(Pt { x: p.x, y: p.y })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_box_sits_centred_above_the_bottom_of_the_work_area() {
        let work = |left, right, bottom| RECT {
            left,
            top: 0,
            right,
            bottom,
        };
        assert_eq!(
            bottom_centre(&work(0, 1920, 1040), (900, 70), 30),
            Pt { x: 510, y: 940 }
        );
        let wide = bottom_centre(&work(100, 500, 800), (600, 50), 10);
        assert_eq!(
            wide.x, 100,
            "a box wider than the screen starts at its left edge"
        );
    }

    #[test]
    fn the_bubble_sits_in_a_bottom_corner_of_the_work_area() {
        let rect = |left, top, right, bottom| RECT {
            left,
            top,
            right,
            bottom,
        };
        let work = rect(0, 0, 1920, 1020);
        assert_eq!(corner_spot(&work, false, 46, 12), Pt { x: 1862, y: 962 });
        assert_eq!(corner_spot(&work, true, 46, 12), Pt { x: 12, y: 962 });
        let second = rect(1920, 0, 3840, 1080);
        assert_eq!(corner_spot(&second, false, 46, 12), Pt { x: 3782, y: 1022 });
        let tiny = rect(0, 0, 20, 20);
        assert_eq!(
            corner_spot(&tiny, false, 46, 12),
            Pt { x: 0, y: 0 },
            "a screen smaller than the bubble"
        );
    }

    #[test]
    fn logical_pixels_scale_with_the_dpi() {
        assert_eq!(physical(72.0, 96), 72);
        assert_eq!(physical(72.0, 120), 90);
        assert_eq!(physical(8.0, 120), 10);
        assert_eq!(physical(33.0, 144), 50);
    }
}
