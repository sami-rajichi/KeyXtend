//! The window writes G9 makes as an outside program would, and the repair that puts each window back as G9 found it.
#![cfg(windows)]

use std::collections::HashMap;

use windows::Win32::Foundation::{ERROR_SUCCESS, GetLastError, HWND, SetLastError, WIN32_ERROR};
use windows::Win32::UI::WindowsAndMessaging::{
    GWL_EXSTYLE, GetWindowLongPtrW, HWND_NOTOPMOST, HWND_TOPMOST, SWP_NOACTIVATE, SWP_NOMOVE,
    SWP_NOSIZE, SetWindowLongPtrW, SetWindowPos, WS_EX_TOPMOST,
};

use kx_platform_windows::window::{KEPT, THROUGH};

use crate::{product, win};

/// A window's extended style.
#[allow(unsafe_code, reason = "A plain query of a window's style.")]
pub fn ex_of(w: HWND) -> u32 {
    // SAFETY: a plain query; a gone window reads as 0.
    let ex = unsafe { GetWindowLongPtrW(w, GWL_EXSTYLE) };
    u32::try_from(ex).unwrap_or_default()
}

/// Writes extended style `ex` to window `w`; an error when Windows refused it, which a guard's change never is.
#[allow(unsafe_code, reason = "A style write and its error code.")]
fn write_ex(w: HWND, ex: u32) -> Result<(), String> {
    let value = isize::try_from(ex).unwrap_or_default();
    // SAFETY: plain calls; a write's old value can be 0, so the error code tells a refusal.
    let refused = unsafe {
        SetLastError(WIN32_ERROR(0));
        let old = SetWindowLongPtrW(w, GWL_EXSTYLE, value);
        old == 0 && GetLastError() != ERROR_SUCCESS
    };
    if refused {
        Err(format!(
            "Windows refused a style write to {}",
            win::describe(w)
        ))
    } else {
        Ok(())
    }
}

/// Puts window `w` in or out of the topmost band without moving or activating it.
#[allow(unsafe_code, reason = "Moves another window in the z-order.")]
fn set_topmost(w: HWND, on: bool) -> Result<(), String> {
    let band = if on { HWND_TOPMOST } else { HWND_NOTOPMOST };
    // SAFETY: a plain call on a window handle; Windows reports a refusal.
    unsafe {
        SetWindowPos(
            w,
            Some(band),
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
        )
    }
    .map_err(|e| format!("topmost on {}: {e}", win::describe(w)))
}

/// An outside program turns topmost off on `windows`.
pub fn drop_topmost(windows: &[HWND]) -> Result<(), String> {
    windows.iter().try_for_each(|&w| set_topmost(w, false))
}

/// An outside program clears the guard's flags and click-through on `windows`.
pub fn strip_flags(windows: &[HWND]) -> Result<(), String> {
    windows
        .iter()
        .try_for_each(|&w| write_ex(w, ex_of(w) & !(KEPT | THROUGH)))
}

/// Puts every shown product window G9 saw back to its `first` style, topmost included; how many needed it.
pub fn repair(first: &HashMap<isize, u32>) -> usize {
    let mut fixed = 0;
    for w in product::shown() {
        let Some(&was) = first.get(&win::key(w)) else {
            continue;
        };
        if ex_of(w) == was {
            continue;
        }
        let _ = write_ex(w, was);
        let _ = set_topmost(w, was & WS_EX_TOPMOST.0 != 0);
        fixed += 1;
    }
    fixed
}
