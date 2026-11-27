//! The probes' own windows and screen reads: plain topmost windows of one colour that never take focus, shielded from
//! capture on demand; a screen box grabbed by the freeze's own GDI copy; and a wait for DWM's next frame.
#![allow(
    unsafe_code,
    reason = "The probes make, show, shield and destroy their own windows and read the screen."
)]

use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, Instant};

use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows::Win32::Graphics::Dwm::DwmFlush;
use windows::Win32::Graphics::Gdi::{
    CreateSolidBrush, DeleteObject, GetMonitorInfoW, MONITOR_DEFAULTTOPRIMARY, MONITORINFO,
    MonitorFromPoint, UpdateWindow,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, MSG, PM_REMOVE, PeekMessageW,
    RegisterClassW, SW_HIDE, SW_SHOWNOACTIVATE, ShowWindow, TranslateMessage, WNDCLASSW,
    WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP,
};
use windows::core::{HSTRING, PCWSTR};

use crate::WinGuard;
use crate::screen::rect_of;
use crate::snip::aware::Aware;
use crate::worker::size32;
use kx_module_api::ports::BASE_DPI;
use kx_platform::ui::WindowGuard;

/// Bytes in one BGRA pixel.
pub(super) const PIXEL: usize = 4;
/// The start of each colour's window class name.
const CLASS_TAG: &str = "KeyXtend.SnipProbe";

/// A colour as red, green and blue.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Rgb(pub(super) u8, pub(super) u8, pub(super) u8);

impl Rgb {
    /// Whether the BGRA `pixel` is within `by` of this colour in each channel.
    pub(super) fn near(self, pixel: [u8; PIXEL], by: u8) -> bool {
        let (Rgb(r, g, b), [sb, sg, sr, _]) = (self, pixel);
        sb.abs_diff(b) <= by && sg.abs_diff(g) <= by && sr.abs_diff(r) <= by
    }
}

/// This thread reads the screen in physical pixels, as the freeze does, until the guard drops.
pub(super) fn physical() -> Aware {
    Aware::physical().unwrap()
}

/// A plain window of one colour and a title of its own, topmost and never activated; destroyed when dropped.
pub(super) struct Plain(HWND, String);

impl Plain {
    /// A hidden window over `at`, painted `colour`.
    pub(super) fn new(at: RECT, colour: Rgb) -> Self {
        static NEXT: AtomicU32 = AtomicU32::new(0);
        let title = format!("{CLASS_TAG} {}", NEXT.fetch_add(1, Ordering::Relaxed));
        let (class, name) = (class(colour), HSTRING::from(&title));
        let ex = WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE;
        let (w, h) = (at.right - at.left, at.bottom - at.top);
        // SAFETY: a window of our registered class, with no parent, menu or creation data.
        let made = unsafe {
            CreateWindowExW(
                ex, &class, &name, WS_POPUP, at.left, at.top, w, h, None, None, None, None,
            )
        };
        Self(made.unwrap(), title)
    }

    pub(super) fn hwnd(&self) -> HWND {
        self.0
    }

    /// Shows it without activating it, painted at once.
    pub(super) fn show(&self) {
        // SAFETY: our window, on the thread that made it.
        unsafe {
            let _ = ShowWindow(self.0, SW_SHOWNOACTIVATE);
            let _ = UpdateWindow(self.0);
        }
    }

    pub(super) fn hide(&self) {
        // SAFETY: our window, on the thread that made it.
        let _ = unsafe { ShowWindow(self.0, SW_HIDE) };
    }

    /// Keeps it out of every capture when `on`, or lets it back in, through the real `WindowGuard::shield` by title.
    pub(super) fn shield(&self, on: bool) {
        WinGuard.shield(&self.1, on).unwrap();
    }
}

impl Drop for Plain {
    fn drop(&mut self) {
        // SAFETY: our window, destroyed once on the thread that made it.
        let _ = unsafe { DestroyWindow(self.0) };
    }
}

/// The window class painted `colour`, registered on first use; registering it again fails and frees its brush.
fn class(colour: Rgb) -> HSTRING {
    let Rgb(r, g, b) = colour;
    let name = HSTRING::from(format!("{CLASS_TAG}.{r:02x}{g:02x}{b:02x}"));
    let paint = COLORREF(u32::from_le_bytes([r, g, b, 0]));
    // SAFETY: a class of plain values and a live name; a registered class owns its brush, a refused one does not.
    unsafe {
        let brush = CreateSolidBrush(paint);
        let wc = WNDCLASSW {
            lpfnWndProc: Some(proc),
            hInstance: GetModuleHandleW(None).unwrap().into(),
            hbrBackground: brush,
            lpszClassName: PCWSTR(name.as_ptr()),
            ..Default::default()
        };
        if RegisterClassW(&raw const wc) == 0 {
            let _ = DeleteObject(brush.into());
        }
    }
    name
}

/// The probes' window procedure: the default one, whose background is the class's colour.
unsafe extern "system" fn proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    // SAFETY: default handling of a message for the probe's own window.
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

/// Reads this thread's messages, so our windows paint, then waits for DWM's next frame.
pub(super) fn frame() {
    let mut msg = MSG::default();
    // SAFETY: a local message, read and handed on for this thread's own windows.
    unsafe {
        while PeekMessageW(&raw mut msg, None, 0, 0, PM_REMOVE).as_bool() {
            let _ = TranslateMessage(&raw const msg);
            DispatchMessageW(&raw const msg);
        }
        let _ = DwmFlush();
    }
}

/// Waits a frame at a time, at most `within`, until `done`; says whether it came.
pub(super) fn until(within: Duration, mut done: impl FnMut() -> bool) -> bool {
    let end = Instant::now() + within;
    while !done() {
        if Instant::now() > end {
            return false;
        }
        frame();
    }
    true
}

/// The screen box `at`, grabbed by the freeze's own GDI copy as BGRX pixels, top row first.
pub(super) fn grab(at: RECT) -> Arc<[u8]> {
    let rect = rect_of([at.left, at.top, at.right, at.bottom]);
    let grabbed = crate::snip::grab::grab(&[(rect, BASE_DPI)]).unwrap();
    grabbed.first().unwrap().shot().bgrx().expose().clone()
}

/// Pixels in the box `at`.
pub(super) fn area(at: RECT) -> usize {
    let side = |n: i32| usize::try_from(n).unwrap_or_default();
    side(at.right - at.left) * side(at.bottom - at.top)
}

/// How many BGRA pixels of `pixels` are exactly `colour`.
pub(super) fn count(pixels: &[u8], colour: Rgb) -> usize {
    let (whole, _) = pixels.as_chunks::<PIXEL>();
    whole.iter().filter(|p| colour.near(**p, 0)).count()
}

/// The primary monitor's work area, in physical pixels when the thread reads them.
pub(super) fn work_area() -> RECT {
    let mut info = MONITORINFO {
        cbSize: size32::<MONITORINFO>(),
        ..Default::default()
    };
    // SAFETY: the primary monitor's handle and a local whose size it states.
    unsafe {
        let primary = MonitorFromPoint(POINT::default(), MONITOR_DEFAULTTOPRIMARY);
        assert!(GetMonitorInfoW(primary, &raw mut info).as_bool());
    }
    info.rcWork
}

/// The box of `w` by `h` whose top-left is `margin` inside `within`'s.
pub(super) fn inset(within: RECT, margin: i32, w: i32, h: i32) -> RECT {
    let (left, top) = (within.left + margin, within.top + margin);
    RECT {
        left,
        top,
        right: left + w,
        bottom: top + h,
    }
}

/// The box of `w` by `h` at the centre of `of`.
pub(super) fn centred(of: RECT, w: i32, h: i32) -> RECT {
    let left = of.left + (of.right - of.left - w) / 2;
    let top = of.top + (of.bottom - of.top - h) / 2;
    RECT {
        left,
        top,
        right: left + w,
        bottom: top + h,
    }
}
