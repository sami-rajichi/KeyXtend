//! A window's own drawing by `PrintWindow`, the reference a picture must not hold. Windows may draw a window kept out
//! of captures blank, and a blank drawing counts as none.
#![cfg(windows)]

use std::sync::mpsc;
use std::time::Duration;

use spike_core::capture::Shot;
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Gdi::{
    BI_RGB, BITMAPINFO, BITMAPINFOHEADER, CreateCompatibleBitmap, CreateCompatibleDC,
    DIB_RGB_COLORS, DeleteDC, DeleteObject, GetDC, GetDIBits, ReleaseDC, SelectObject,
};
use windows::Win32::Storage::Xps::{PRINT_WINDOW_FLAGS, PrintWindow};

use super::pic::{BGRA, BITS, Pic};
use crate::win::{self, size32};

/// `PW_RENDERFULLCONTENT`: draw the window's DirectX content too, as Qt Quick paints with it.
const FULL_CONTENT: PRINT_WINDOW_FLAGS = PRINT_WINDOW_FLAGS(2);
/// Colour planes; a DIB has one.
const PLANES: u16 = 1;

/// `f` on a thread of its own, waited for up to `wait_ms`: its value, none once the wait passed and it left running.
fn apart<T: Send + 'static>(wait_ms: u64, f: impl FnOnce() -> T + Send + 'static) -> Option<T> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = tx.send(f());
    });
    rx.recv_timeout(Duration::from_millis(wait_ms)).ok()
}

/// Window `hwnd`'s own drawing as `drawn` gives it, within `wait_ms`, so a hung window never holds the gate.
pub fn drawn_within(hwnd: HWND, wait_ms: u64) -> Result<Option<Pic>, String> {
    let key = win::key(hwnd);
    apart(wait_ms, move || drawn(HWND(key as *mut _)))
        .ok_or_else(|| format!("PrintWindow did not end within {wait_ms} ms"))?
}

/// Window `hwnd`'s own drawing placed at its box; none when Windows drew it all one colour.
fn drawn(hwnd: HWND) -> Result<Option<Pic>, String> {
    let r = win::rect(hwnd)?;
    let (width, height) = (r.right - r.left, r.bottom - r.top);
    let shot = Shot {
        left: r.left,
        top: r.top,
        width,
        height,
        bgra: print(hwnd, width, height)?,
    };
    let pic = Pic::from_shot(&shot);
    Ok((!pic.uniform()).then_some(pic))
}

/// The header of a top-down `w` × `h` DIB of 32-bit pixels.
fn info(w: i32, h: i32) -> Result<BITMAPINFO, String> {
    Ok(BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: size32::<BITMAPINFOHEADER>(),
            biWidth: w,
            // A negative height puts the top row first.
            biHeight: -h,
            biPlanes: PLANES,
            biBitCount: u16::try_from(BGRA * BITS).map_err(|e| e.to_string())?,
            biCompression: BI_RGB.0,
            ..Default::default()
        },
        ..Default::default()
    })
}

/// The `w` × `h` pixels `PrintWindow` draws of `hwnd`, top row first, blue, green, red and one more byte each.
#[allow(unsafe_code, reason = "GDI calls on handles made and freed here.")]
fn print(hwnd: HWND, w: i32, h: i32) -> Result<Vec<u8>, String> {
    let empty = || format!("a {w}x{h} window has nothing to draw");
    let rows = u32::try_from(h).map_err(|_| empty())?;
    let len = usize::try_from(w)
        .ok()
        .zip(usize::try_from(h).ok())
        .and_then(|(w, h)| w.checked_mul(h)?.checked_mul(BGRA))
        .filter(|&n| n > 0)
        .ok_or_else(empty)?;
    let mut info = info(w, h)?;
    let mut pixels = vec![0u8; len];
    // SAFETY: every handle made here is freed before returning; `pixels` holds the `h` rows of `w` 32-bit pixels that
    // `info` tells GetDIBits, and the bitmap is deselected before it runs.
    let (printed, got) = unsafe {
        let screen = GetDC(None);
        let mem = CreateCompatibleDC(Some(screen));
        let bmp = CreateCompatibleBitmap(screen, w, h);
        let old = SelectObject(mem, bmp.into());
        let printed = PrintWindow(hwnd, mem, FULL_CONTENT);
        SelectObject(mem, old);
        let buf = Some(pixels.as_mut_ptr().cast());
        let got = GetDIBits(mem, bmp, 0, rows, buf, &raw mut info, DIB_RGB_COLORS);
        let _ = DeleteObject(bmp.into());
        let _ = DeleteDC(mem);
        ReleaseDC(None, screen);
        (printed, got)
    };
    if !printed.as_bool() || got != h {
        return Err(format!("PrintWindow drew {got} of {h} rows"));
    }
    Ok(pixels)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A wait that a call ending at once never reaches.
    const LONG_MS: u64 = 60_000;

    #[test]
    fn a_call_still_running_at_its_wait_gives_none_and_one_that_ends_gives_its_value() {
        let (hold, blocked) = mpsc::channel::<()>();
        assert_eq!(
            apart(0, move || blocked.recv().is_ok()),
            None,
            "still running"
        );
        drop(hold);
        assert_eq!(apart(LONG_MS, || 7), Some(7));
    }
}
