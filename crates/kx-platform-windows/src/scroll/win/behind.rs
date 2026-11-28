//! The window behind ours at a point (ADR-0029): the first other app's top-level window there in z-order that shows,
//! is not cloaked and takes clicks, then its deepest child window there, as `WindowFromPoint` finds it without ours.
//! Every call is a plain query that sends the windows nothing, so a hung app never holds it.

use kx_module_api::ports::Point;
use windows::Win32::Foundation::{HWND, POINT, RECT};
use windows::Win32::Graphics::Gdi::ScreenToClient;
use windows::Win32::UI::WindowsAndMessaging::{
    CWP_SKIPDISABLED, CWP_SKIPINVISIBLE, CWP_SKIPTRANSPARENT, ChildWindowFromPointEx, GetWindowRect,
};

use crate::screen::{contains, rect_of};
pub(super) use crate::window::{cloaked, ex_of, visible as shown};
use crate::window::{is_ours, is_through, top_level};

/// The first other app's window at `p` behind ours, as its deepest child window there; none when there is none.
pub(super) fn behind(p: Point) -> Option<HWND> {
    let top = top_level().into_iter().find(|&w| takes(w, p))?;
    Some(deepest(top, p))
}

/// True when top-level window `hwnd` can take the target at `p`: another app's that fits there.
pub(super) fn takes(hwnd: HWND, p: Point) -> bool {
    !is_ours(hwnd) && fits(hwnd, p)
}

/// True when top-level window `hwnd` shows at `p` and takes clicks there: shown, holding `p`, not click-through and not
/// cloaked, as on another virtual desktop. The cheap checks go first.
pub(super) fn fits(hwnd: HWND, p: Point) -> bool {
    shown(hwnd) && holds(hwnd, p) && !is_through(ex_of(hwnd)) && !cloaked(hwnd)
}

/// True when the box of `hwnd` holds screen point `p`.
#[allow(unsafe_code, reason = "A box query into a local.")]
pub(super) fn holds(hwnd: HWND, p: Point) -> bool {
    let mut r = RECT::default();
    // SAFETY: a plain query into a local rect; a gone window refuses it.
    let read = unsafe { GetWindowRect(hwnd, &raw mut r) }.is_ok();
    read && contains(rect_of([r.left, r.top, r.right, r.bottom]), p)
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
