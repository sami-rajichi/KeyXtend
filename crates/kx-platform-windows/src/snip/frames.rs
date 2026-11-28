//! The windows a freeze offers (edge 3): every top-level window front first, told shown or not by its looks, and a
//! shown one's frame without its shadow from DWM, in physical pixels.

use kx_module_api::ports::Rect;
use kx_platform::snip::{Looks, Seen};
use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::Graphics::Dwm::{DWMWA_EXTENDED_FRAME_BOUNDS, DwmGetWindowAttribute};
use windows::Win32::UI::WindowsAndMessaging::{
    GetShellWindow, GetWindowRect, IsIconic, WS_EX_TOOLWINDOW,
};

use crate::screen::rect_of;
use crate::window::{cloaked, ex_of, is_ours, is_through, top_level, visible};

/// Every top-level window, front first, with how it shows; only a shown one's frame is read.
pub(super) fn windows() -> Vec<(Rect, Seen)> {
    let desktop = shell();
    let one = |hwnd| {
        let how = seen(hwnd, desktop);
        let shown = how == Seen::Shown;
        (if shown { frame(hwnd) } else { Rect::default() }, how)
    };
    top_level().into_iter().map(one).collect()
}

/// How `hwnd` shows, `desktop` being the system's own desktop window.
pub(crate) fn seen(hwnd: HWND, desktop: HWND) -> Seen {
    Seen::of(&Window::new(hwnd, desktop))
}

/// A top-level window, with the system's desktop window to tell it from.
pub(super) struct Window {
    hwnd: HWND,
    desktop: HWND,
}

impl Window {
    /// The window `hwnd`, `desktop` being the system's own desktop window.
    pub(super) const fn new(hwnd: HWND, desktop: HWND) -> Self {
        Self { hwnd, desktop }
    }
}

impl Looks for Window {
    fn shown(&self) -> bool {
        visible(self.hwnd)
    }

    fn ours(&self) -> bool {
        is_ours(self.hwnd)
    }

    fn desktop(&self) -> bool {
        self.hwnd == self.desktop
    }

    #[allow(unsafe_code, reason = "A plain window query.")]
    fn minimised(&self) -> bool {
        // SAFETY: a plain query; a gone window reads as not minimised.
        unsafe { IsIconic(self.hwnd) }.as_bool()
    }

    fn tool(&self) -> bool {
        ex_of(self.hwnd) & WS_EX_TOOLWINDOW.0 != 0
    }

    /// The scroll target's rule too, so no click-through overlay takes a pick.
    fn through(&self) -> bool {
        is_through(ex_of(self.hwnd))
    }

    fn cloaked(&self) -> bool {
        cloaked(self.hwnd)
    }
}

/// The system's desktop window; null when the shell is not running.
#[allow(unsafe_code, reason = "A plain window query.")]
pub(crate) fn shell() -> HWND {
    // SAFETY: a plain query.
    unsafe { GetShellWindow() }
}

/// The frame of `hwnd` without its shadow: DWM's bounds, never scaled, else its window box, physical only while the
/// thread is; empty when Windows will not say.
#[allow(unsafe_code, reason = "Window box queries into a local.")]
pub(super) fn frame(hwnd: HWND) -> Rect {
    let mut r = RECT::default();
    let size = u32::try_from(size_of::<RECT>()).unwrap_or_default();
    let at = (&raw mut r).cast();
    // SAFETY: the call writes one RECT into a local that outlives it.
    let bounds = unsafe { DwmGetWindowAttribute(hwnd, DWMWA_EXTENDED_FRAME_BOUNDS, at, size) };
    // SAFETY: a plain query into the same local; a gone window refuses it.
    if bounds.is_err() && unsafe { GetWindowRect(hwnd, &raw mut r) }.is_err() {
        return Rect::default();
    }
    rect_of([r.left, r.top, r.right, r.bottom])
}
