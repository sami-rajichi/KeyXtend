//! Aiming the scroll pad at a window: its lower part laid under the keyboard, a start point on it above the keyboard,
//! the glide from there onto the pad, and that glide's last point outside the keyboard, which the pad scrolls.
#![cfg(windows)]

use spike_core::pt::Pt;
use windows::Win32::Foundation::{HWND, POINT, RECT};
use windows::Win32::UI::WindowsAndMessaging::{SWP_NOACTIVATE, SWP_NOZORDER, SetWindowPos};

use crate::simin;
use crate::win::{self, sleep_ms};

/// How far a window must reach around the keyboard's top: under it, and above it for the glide's start, in px.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Reach {
    /// How far under the keyboard's top.
    pub under_px: i32,
    /// How far above it the glide starts.
    pub lead_px: i32,
}

/// Where window box `win` must go so it reaches `reach` around keyboard box `kb`'s top at the pad's `x`: across the
/// top of work area `work`, down to `under_px` under the keyboard's top; none when it reaches so already.
pub fn laid(win: RECT, kb: RECT, x: i32, reach: Reach, work: RECT) -> Option<RECT> {
    let across = win.left <= x && x < win.right;
    let above = win.top <= kb.top - reach.lead_px;
    let under = win.bottom >= kb.top + reach.under_px;
    if across && above && under {
        return None;
    }
    Some(RECT {
        bottom: (kb.top + reach.under_px).min(work.bottom),
        ..work
    })
}

/// The glide's start: `lead_px` above keyboard box `kb`'s top at the pad's `x`; none when that is off window box `win`.
pub fn start(win: RECT, kb: RECT, x: i32, lead_px: i32) -> Option<POINT> {
    let p = POINT {
        x,
        y: kb.top - lead_px,
    };
    inside(p, win).then_some(p)
}

/// The glide's points from `from` to `to` in `steps` even steps, `from` first.
pub fn glide(from: POINT, to: POINT, steps: i32) -> Vec<POINT> {
    let steps = steps.max(1);
    let at = |a: i32, b: i32, i: i32| a + (b - a) * i / steps;
    (0..=steps)
        .map(|i| POINT {
            x: at(from.x, to.x, i),
            y: at(from.y, to.y, i),
        })
        .collect()
}

/// The last of `points` outside keyboard box `kb`: where the keyboard's entry watch keeps the pointer, so the pad
/// scrolls there.
pub fn last_outside(points: &[POINT], kb: RECT) -> Option<POINT> {
    points.iter().rev().copied().find(|&p| !inside(p, kb))
}

/// True when `p` lies in box `r`, its top and left edges in and its bottom and right edges out.
fn inside(p: POINT, r: RECT) -> bool {
    simin::inside(&r, Pt { x: p.x, y: p.y })
}

/// What G6 pictures of window box `win`: the part above keyboard box `kb`'s top, `inset` in from every side, so the
/// outline's edges and the keyboard never count; none when nothing is left.
pub fn band(win: RECT, kb: RECT, inset: i32) -> Option<RECT> {
    let r = RECT {
        left: win.left + inset,
        top: win.top + inset,
        right: win.right - inset,
        bottom: win.bottom.min(kb.top) - inset,
    };
    (r.right > r.left && r.bottom > r.top).then_some(r)
}

/// Moves the pointer through `points` with `mover`, `step_ms` apart.
pub fn walk(
    points: &[POINT],
    step_ms: u64,
    mover: impl Fn(POINT) -> Result<(), String>,
) -> Result<(), String> {
    for &p in points {
        mover(p)?;
        sleep_ms(step_ms);
    }
    Ok(())
}

/// A window laid under the keyboard; `put_back`, or dropping it, moves it back where it was.
pub struct Placed {
    hwnd: HWND,
    was: Option<RECT>,
}

impl Placed {
    /// Lays window `hwnd` so it reaches `reach` around keyboard box `kb`'s top at the pad's `x`, unless it does
    /// already, in the work area under the keyboard.
    pub fn lay(hwnd: HWND, kb: RECT, x: i32, reach: Reach) -> Result<Self, String> {
        let was = win::rect(hwnd)?;
        let work = kx_platform_windows::screen::work_area(&kb).map_err(|e| e.to_string())?;
        let moved = laid(was, kb, x, reach, work);
        if let Some(to) = moved {
            move_to(hwnd, to)?;
        }
        Ok(Self {
            hwnd,
            was: moved.map(|_| was),
        })
    }

    /// Moves the window back where it was, once; nothing when it never moved.
    pub fn put_back(&mut self) -> Result<(), String> {
        self.was.take().map_or(Ok(()), |r| move_to(self.hwnd, r))
    }
}

impl Drop for Placed {
    fn drop(&mut self) {
        let _ = self.put_back();
    }
}

/// Moves and sizes window `hwnd` to box `r`, leaving its place in the z-order and never activating it.
#[allow(unsafe_code, reason = "Moves a window the gate opened.")]
fn move_to(hwnd: HWND, r: RECT) -> Result<(), String> {
    let keep = SWP_NOZORDER | SWP_NOACTIVATE;
    let (w, h) = (r.right - r.left, r.bottom - r.top);
    // SAFETY: a plain move of a window handle; a bad handle only gives an error.
    unsafe { SetWindowPos(hwnd, None, r.left, r.top, w, h, keep) }
        .map_err(|e| format!("lay {} under the keyboard: {e}", win::describe(hwnd)))
}

#[cfg(test)]
mod tests;
