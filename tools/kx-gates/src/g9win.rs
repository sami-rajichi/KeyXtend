//! The window writes G9 makes as an outside program would, how it reads the guard's mark, and the repair that puts each
//! window it wrote back as G9 found it.
#![cfg(windows)]

use std::collections::HashMap;

use serde::Serialize;

use windows::Win32::Foundation::{ERROR_SUCCESS, GetLastError, HWND, SetLastError, WIN32_ERROR};
use windows::Win32::UI::WindowsAndMessaging::{
    GW_OWNER, GWL_EXSTYLE, GetWindow, GetWindowLongPtrW, HWND_NOTOPMOST, HWND_TOPMOST,
    SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SetWindowLongPtrW, SetWindowPos, WS_EX_LAYERED,
    WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_EX_TRANSPARENT,
};

use kx_platform_windows::window::{KEPT, THROUGH};

use crate::{win, winfind};

/// Whether the guard is on a window, as G9 can tell from its mark.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Guard {
    On,
    Off,
    /// The keyboard showed no mark at the first look, so a missing mark says nothing.
    Unknown,
}

/// The guard on a window whose mark reads `hooked`, when the keyboard's read as `readable` at the first look.
pub fn guard_of(hooked: bool, readable: bool) -> Guard {
    match (hooked, readable) {
        (true, _) => Guard::On,
        (false, true) => Guard::Off,
        (false, false) => Guard::Unknown,
    }
}

/// The key of the window that owns `w`, as G9 names windows; the keyboard's owner is its shadow.
#[allow(unsafe_code, reason = "A plain owner query.")]
pub fn owner_of(w: HWND) -> Option<String> {
    // SAFETY: a plain query; a window with no owner gives an error.
    let owner = unsafe { GetWindow(w, GW_OWNER) }.ok()?;
    (!owner.is_invalid()).then(|| format!("{:#x}", win::key(owner)))
}

/// Each flag by name: the guard's three, then the two that make a window click-through.
pub const FLAGS: [(&str, u32); 5] = [
    ("no_activate", WS_EX_NOACTIVATE.0),
    ("topmost", WS_EX_TOPMOST.0),
    ("tool_window", WS_EX_TOOLWINDOW.0),
    ("layered", WS_EX_LAYERED.0),
    ("transparent", WS_EX_TRANSPARENT.0),
];

/// The names of the flags in `want` that `ex` lacks.
pub fn missing(ex: u32, want: u32) -> Vec<&'static str> {
    let lacks = |&&(_, bit): &&(&str, u32)| want & bit != 0 && ex & bit == 0;
    FLAGS.iter().filter(lacks).map(|(name, _)| *name).collect()
}

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

/// An outside program turns click-through on for `windows`.
pub fn through_on(windows: &[HWND]) -> Result<(), String> {
    windows
        .iter()
        .try_for_each(|&w| write_ex(w, ex_of(w) | THROUGH))
}

/// Whether window `who` is back as G9 found it: its writes, `wrote`, worked and its style `now` is the first, `was`.
pub fn put_back(wrote: Result<(), String>, now: u32, was: u32, who: &str) -> Result<(), String> {
    wrote?;
    if now == was {
        Ok(())
    } else {
        Err(format!("{who}: style {now:#010x}, not {was:#010x}"))
    }
}

/// What the repair did: the windows it put back, and why each other one is not.
#[derive(Debug, Default)]
pub struct Repaired {
    /// Windows whose style needed it and is now the first again.
    pub fixed: usize,
    /// Windows that still differ, or whose writes Windows refused.
    pub not_put_back: Vec<String>,
}

/// Puts each window G9 `written` back to its `first` style, topmost included, shown or hidden; gone ones need nothing.
pub fn repair(first: &HashMap<isize, u32>, written: &[HWND]) -> Repaired {
    let mut done = Repaired::default();
    for &w in written {
        let Some(&was) = first.get(&win::key(w)) else {
            continue;
        };
        if !winfind::exists(w) || ex_of(w) == was {
            continue;
        }
        let wrote = write_ex(w, was).and(set_topmost(w, was & WS_EX_TOPMOST.0 != 0));
        match put_back(wrote, ex_of(w), was, &win::describe(w)) {
            Ok(()) => done.fixed += 1,
            Err(e) => done.not_put_back.push(e),
        }
    }
    done
}
