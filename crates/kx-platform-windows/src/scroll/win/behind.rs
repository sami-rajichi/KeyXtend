//! The window behind ours at a point (ADR-0029): the first other app's top-level window there in z-order that shows,
//! is not cloaked and takes clicks, then its deepest child window there, as `WindowFromPoint` finds it without ours.
//! Every call is a plain query that sends the windows nothing, so a hung app never holds it.

use kx_module_api::ports::Point;
use windows::Win32::Foundation::{HWND, POINT, RECT};
use windows::Win32::Graphics::Dwm::{DWMWA_CLOAKED, DwmGetWindowAttribute};
use windows::Win32::Graphics::Gdi::ScreenToClient;
use windows::Win32::UI::WindowsAndMessaging::{
    CWP_SKIPDISABLED, CWP_SKIPINVISIBLE, CWP_SKIPTRANSPARENT, ChildWindowFromPointEx, GWL_EXSTYLE,
    GetWindowLongPtrW, GetWindowRect, IsWindowVisible,
};

use crate::screen::{contains, rect_of};
use crate::window::{is_ours, is_through, top_level};

/// The first other app's window at `p` behind ours, as its deepest child window there; none when there is none.
pub(super) fn behind(p: Point) -> Option<HWND> {
    let top = top_level().into_iter().find(|&w| takes(w, p))?;
    Some(deepest(top, p))
}

/// True when top-level window `hwnd` can take the target at `p`: another app's, shown, holding `p`, taking clicks and
/// not cloaked, as on another virtual desktop. The cheap checks go first.
pub(super) fn takes(hwnd: HWND, p: Point) -> bool {
    !is_ours(hwnd) && shown(hwnd) && holds(hwnd, p) && !is_through(ex_of(hwnd)) && !cloaked(hwnd)
}

/// True while `hwnd` is shown.
#[allow(unsafe_code, reason = "A plain window query.")]
fn shown(hwnd: HWND) -> bool {
    // SAFETY: a plain query; a gone window reads as hidden.
    unsafe { IsWindowVisible(hwnd) }.as_bool()
}

/// True when the box of `hwnd` holds screen point `p`.
#[allow(unsafe_code, reason = "A box query into a local.")]
fn holds(hwnd: HWND, p: Point) -> bool {
    let mut r = RECT::default();
    // SAFETY: a plain query into a local rect; a gone window refuses it.
    let read = unsafe { GetWindowRect(hwnd, &raw mut r) }.is_ok();
    read && contains(rect_of([r.left, r.top, r.right, r.bottom]), p)
}

/// The extended style of `hwnd`; 0 for a gone window.
#[allow(unsafe_code, reason = "A plain style query.")]
fn ex_of(hwnd: HWND) -> u32 {
    // SAFETY: a plain query; a gone window gives 0.
    u32::try_from(unsafe { GetWindowLongPtrW(hwnd, GWL_EXSTYLE) }).unwrap_or_default()
}

/// True while the system cloaks `hwnd`, so it does not show though Windows calls it visible.
#[allow(unsafe_code, reason = "A window attribute read into a local.")]
fn cloaked(hwnd: HWND) -> bool {
    let mut value = 0u32;
    let size = u32::try_from(size_of::<u32>()).unwrap_or_default();
    // SAFETY: the call writes one u32 into a local that outlives it; a refusal reads as not cloaked.
    let read = unsafe { DwmGetWindowAttribute(hwnd, DWMWA_CLOAKED, (&raw mut value).cast(), size) };
    read.is_ok() && value != 0
}

/// The deepest shown, enabled child window of `top` that takes clicks at screen point `p`; `top` when none does.
#[allow(unsafe_code, reason = "Plain window queries into a local point.")]
fn deepest(top: HWND, p: Point) -> HWND {
    let skip = CWP_SKIPINVISIBLE | CWP_SKIPDISABLED | CWP_SKIPTRANSPARENT;
    let mut at = top;
    loop {
        let mut pt = POINT { x: p.x, y: p.y };
        // SAFETY: plain queries on a window and a local point; each child is deeper, so the walk ends.
        let child = unsafe {
            if !ScreenToClient(at, &raw mut pt).as_bool() {
                return at;
            }
            ChildWindowFromPointEx(at, pt, skip)
        };
        if child.is_invalid() || child == at {
            return at;
        }
        at = child;
    }
}
