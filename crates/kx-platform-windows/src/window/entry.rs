//! The entry watch (ADR-0029 decision 4): each time the pointer enters our watched window, the guard reads Windows'
//! pointer history once and keeps the point before it for the scroll target. Windows gives the history in the caller's
//! DPI, so in our per-monitor DPI aware process every point is in physical pixels; none is ever logged.

use std::sync::Mutex;

use kx_module_api::ports::Point;
use kx_platform::ui::{Screens, UiError};
use windows::Win32::Foundation::{HANDLE, HWND, LPARAM, POINT, RECT};
use windows::Win32::Graphics::Gdi::ClientToScreen;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GMMP_USE_DISPLAY_POINTS, GetCapture, GetMouseMovePointsEx, MOUSEMOVEPOINT, TME_LEAVE,
    TRACKMOUSEEVENT, TrackMouseEvent,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GA_ROOT, GetAncestor, GetPropW, GetWindowRect, RemovePropW, SetPropW, WindowFromPoint,
};
use windows::core::{PCWSTR, w};

use super::guard::guarded;
use super::win::own_titled;
use crate::screen::{cursor, rect_of};
use crate::scroll::entry::{Kept, to_history};
use crate::scroll::lparam_point;
use crate::{WinScreens, failed, lock, refused};

/// The window property that marks the window whose entries the guard watches: the keyboard.
const WATCH: PCWSTR = w!("KeyXtend.Entries");
/// The window property set while the pointer is on the watched window.
const INSIDE: PCWSTR = w!("KeyXtend.Inside");
/// What `WATCH` holds.
const MARK: usize = 1;
/// What `INSIDE` holds after a move: the next leave under capture may ask again.
const MOVED: usize = 1;
/// What `INSIDE` holds once a leave under capture asked again, until the next move.
const ASKED: usize = 2;
/// How many points Windows keeps in its pointer history (`GetMouseMovePointsEx`).
const HISTORY: usize = 64;

/// What the guard kept at the latest entry; one keyboard per process.
static KEPT: Mutex<Option<Kept>> = Mutex::new(None);

/// Watches the entries of our shown window titled `title`.
///
/// # Errors
/// `NoWindow` while it is not shown, `Refused` when Windows says no.
pub(super) fn watch_entries(title: &str) -> Result<(), UiError> {
    mark(own_titled(title).ok_or(UiError::NoWindow)?)
}

/// Marks our guarded window `hwnd` as the one whose entries the guard watches.
///
/// # Errors
/// `Refused` for a window the guard does not hold, or when Windows says no.
pub(super) fn mark(hwnd: HWND) -> Result<(), UiError> {
    mark_with(hwnd, pointer_on(hwnd))
}

/// Marks our guarded window `hwnd` with the pointer `on` it or not: a pointer already there makes no entry.
///
/// # Errors
/// `Refused` for a window the guard does not hold, or when Windows says no.
pub(super) fn mark_with(hwnd: HWND, on: bool) -> Result<(), UiError> {
    if !guarded(hwnd) {
        return Err(failed());
    }
    put(hwnd, WATCH, MARK).map_err(|e| refused(&e))?;
    if on {
        put(hwnd, INSIDE, MOVED).map_err(|e| refused(&e))?;
        track(hwnd);
    } else {
        clear(hwnd, INSIDE);
    }
    Ok(())
}

/// True when the guard watches `hwnd`'s entries; any thread may ask.
pub(crate) fn watched(hwnd: HWND) -> bool {
    value(hwnd, WATCH) != 0
}

/// What the guard kept when the pointer last entered the watched window.
pub(crate) fn kept() -> Option<Kept> {
    *lock(&KEPT)
}

/// A move on our guarded window `hwnd` at client point `lparam`: the first one since the pointer came in keeps the
/// point before it, and each asks Windows to say when the pointer leaves.
pub(super) fn moved(hwnd: HWND, lparam: LPARAM) {
    if !watched(hwnd) {
        return;
    }
    let was = value(hwnd, INSIDE);
    if was != MOVED && put(hwnd, INSIDE, MOVED).is_err() {
        return;
    }
    track(hwnd);
    if was != 0 {
        return;
    }
    if let Some(at) = on_screen(hwnd, lparam) {
        enter(hwnd, at, history);
    }
}

/// Windows says the pointer left our guarded window `hwnd`.
#[allow(unsafe_code, reason = "A plain capture query.")]
pub(super) fn left(hwnd: HWND) {
    // SAFETY: a plain query of this thread's capture.
    left_with(hwnd, unsafe { GetCapture() } == hwnd);
}

/// A leave of our guarded window `hwnd`, `captured` by it or not: one under capture, as a pressed key gives, asks
/// again once until the next move, and any other ends the entry.
pub(super) fn left_with(hwnd: HWND, captured: bool) {
    if !watched(hwnd) {
        return;
    }
    if !captured {
        clear(hwnd, INSIDE);
    } else if value(hwnd, INSIDE) == MOVED && put(hwnd, INSIDE, ASKED).is_ok() {
        track(hwnd);
    }
}

/// Drops the watch's marks from `hwnd` as it goes, and the entry it kept.
pub(super) fn forget(hwnd: HWND) {
    if watched(hwnd) {
        *lock(&KEPT) = None;
    }
    clear(hwnd, WATCH);
    clear(hwnd, INSIDE);
}

/// Keeps the point before `at`, where the pointer entered `hwnd`, from `history` from `at` back.
pub(super) fn enter(hwnd: HWND, at: Point, history: impl FnOnce(Point) -> Vec<(i32, i32)>) {
    let Some(keyboard) = bounds(hwnd) else {
        return;
    };
    let screens = WinScreens.work_areas();
    *lock(&KEPT) = Some(Kept::new(&history(at), at, keyboard, &screens));
}

/// Windows' pointer history from `at` back, newest first, as 16-bit display points; empty when `at` is not in it. A
/// time of 0 finds the newest match of the point.
#[allow(unsafe_code, reason = "Reads the pointer history into a local buffer.")]
fn history(at: Point) -> Vec<(i32, i32)> {
    let (x, y) = to_history(at);
    let from = MOUSEMOVEPOINT {
        x,
        y,
        ..MOUSEMOVEPOINT::default()
    };
    let mut buf = [MOUSEMOVEPOINT::default(); HISTORY];
    let Ok(size) = u32::try_from(size_of::<MOUSEMOVEPOINT>()) else {
        return Vec::new();
    };
    // SAFETY: `from` and `buf` are locals that outlive the call, which writes at most the buffer's length.
    let found =
        unsafe { GetMouseMovePointsEx(size, &raw const from, &mut buf, GMMP_USE_DISPLAY_POINTS) };
    let found = usize::try_from(found).unwrap_or_default();
    buf.iter().take(found).map(|p| (p.x, p.y)).collect()
}

/// Client point `lparam` of `hwnd` on the screen.
#[allow(unsafe_code, reason = "A point conversion into a local.")]
fn on_screen(hwnd: HWND, lparam: LPARAM) -> Option<Point> {
    let p = lparam_point(lparam.0);
    let mut at = POINT { x: p.x, y: p.y };
    // SAFETY: our own window and a local point.
    let done = unsafe { ClientToScreen(hwnd, &raw mut at) }.as_bool();
    done.then_some(Point { x: at.x, y: at.y })
}

/// The box of our window `hwnd`.
#[allow(unsafe_code, reason = "A box query into a local.")]
fn bounds(hwnd: HWND) -> Option<kx_module_api::ports::Rect> {
    let mut r = RECT::default();
    // SAFETY: our own window and a local rect.
    unsafe { GetWindowRect(hwnd, &raw mut r) }.ok()?;
    Some(rect_of([r.left, r.top, r.right, r.bottom]))
}

/// True while the pointer is on `hwnd` itself, not on a cut-out part of it or on another window.
#[allow(unsafe_code, reason = "Plain window queries.")]
fn pointer_on(hwnd: HWND) -> bool {
    let Ok(p) = cursor() else {
        return false;
    };
    // SAFETY: plain queries; no window gives null.
    let root = unsafe { GetAncestor(WindowFromPoint(POINT { x: p.x, y: p.y }), GA_ROOT) };
    root == hwnd
}

/// Asks Windows to post `WM_MOUSELEAVE` to our window `hwnd` when the pointer leaves it.
#[allow(unsafe_code, reason = "A tracking request for our own window.")]
fn track(hwnd: HWND) {
    let mut ask = TRACKMOUSEEVENT {
        cbSize: u32::try_from(size_of::<TRACKMOUSEEVENT>()).unwrap_or_default(),
        dwFlags: TME_LEAVE,
        hwndTrack: hwnd,
        dwHoverTime: 0,
    };
    // SAFETY: a local request sized for the call, for our own window; a refusal only misses one leave.
    let _ = unsafe { TrackMouseEvent(&raw mut ask) };
}

/// The number `hwnd` holds in property `name`; 0 without it.
#[allow(unsafe_code, reason = "A plain window property query.")]
fn value(hwnd: HWND, name: PCWSTR) -> usize {
    // SAFETY: a plain query; a window without the property gives null.
    unsafe { GetPropW(hwnd, name) }.0.addr()
}

/// Stores `number`, never 0, in property `name` of our window `hwnd`.
#[allow(unsafe_code, reason = "Stores a marker number on our own window.")]
fn put(hwnd: HWND, name: PCWSTR, number: usize) -> windows::core::Result<()> {
    let mark = HANDLE(std::ptr::without_provenance_mut(number));
    // SAFETY: stores a number on our own window; `forget` removes it before the window goes.
    unsafe { SetPropW(hwnd, name, Some(mark)) }
}

/// Removes property `name` from our window `hwnd`, if it is there.
#[allow(unsafe_code, reason = "Removes our own window's property.")]
fn clear(hwnd: HWND, name: PCWSTR) {
    // SAFETY: removes a property this module set on our own window; a missing one gives an error, which is fine.
    let _ = unsafe { RemovePropW(hwnd, name) };
}
