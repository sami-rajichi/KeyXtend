//! The entry watch (ADR-0029 decision 4): each time the pointer enters our watched window, the guard reads Windows'
//! pointer history once and keeps the point before it for the scroll target. Windows gives the history in the caller's
//! DPI, so in our per-monitor DPI aware process every point is in physical pixels; none is ever logged.

use std::sync::Mutex;

use kx_module_api::ports::Point;
use kx_platform::ui::{Screens, UiError, Watched};
use windows::Win32::Foundation::{HWND, LPARAM, POINT, RECT};
use windows::Win32::Graphics::Gdi::ClientToScreen;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GMMP_USE_DISPLAY_POINTS, GetCapture, GetMouseMovePointsEx, MOUSEMOVEPOINT, TME_LEAVE,
    TRACKMOUSEEVENT, TrackMouseEvent,
};
use windows::Win32::UI::WindowsAndMessaging::GetWindowRect;
use windows::core::{PCWSTR, w};

use super::guard::guarded;
use super::pages::passed_at;
use super::prop::{clear, put, value};
use super::watch::{Inside, Seen, step};
use super::win::{is_ours, own_titled, title, top_level, window_at};
use crate::screen::{cursor, rect_of};
use crate::scroll::entry::{Kept, to_history};
use crate::scroll::lparam_point;
use crate::{WinScreens, failed, lock};

/// The window property that marks the window whose entries the guard watches: the keyboard.
const WATCH: PCWSTR = w!("KeyXtend.Entries");
/// The window property that holds where the pointer is for the watched window (`Inside::mark`).
const INSIDE: PCWSTR = w!("KeyXtend.Inside");
/// What `WATCH` holds.
const MARK: usize = 1;
/// How many points Windows keeps in its pointer history (`GetMouseMovePointsEx`).
const HISTORY: usize = 64;

/// What the guard kept at the latest entry; one keyboard per process.
static KEPT: Mutex<Option<Kept>> = Mutex::new(None);

/// Watches the entries of our shown window titled `title`; `Unguarded` while the guard does not hold it yet.
///
/// # Errors
/// `NoWindow` while it is not shown, `Refused` when Windows says no.
pub(super) fn watch_entries(title: &str) -> Result<Watched, UiError> {
    mark(own_titled(title).ok_or(UiError::NoWindow)?)
}

/// Stops watching the entries of our windows titled `title_is`, shown or not, and forgets the entry kept.
pub(super) fn unwatch_entries(title_is: &str) {
    let named = top_level()
        .into_iter()
        .filter(|&w| is_ours(w) && title(w) == title_is);
    named.for_each(drop_marks);
    *lock(&KEPT) = None;
}

/// Marks our guarded window `hwnd` as the one whose entries the guard watches; `Unguarded`, marking nothing, while the
/// guard does not hold it yet.
///
/// # Errors
/// `Refused` when Windows says no.
pub(super) fn mark(hwnd: HWND) -> Result<Watched, UiError> {
    if !guarded(hwnd) {
        return Ok(Watched::Unguarded);
    }
    mark_with(hwnd, pointer_on(hwnd)).map(|()| Watched::Held)
}

/// Marks our guarded window `hwnd` with the pointer `on` it or not: a pointer already there makes no entry.
///
/// # Errors
/// `Refused` for a window the guard does not hold, or when Windows says no.
pub(super) fn mark_with(hwnd: HWND, on: bool) -> Result<(), UiError> {
    if !guarded(hwnd) {
        return Err(failed());
    }
    put(hwnd, WATCH, MARK)?;
    apply(hwnd, Seen::Mark { on })?;
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
    if see(hwnd, Seen::Move)
        && let Some(at) = on_screen(hwnd, lparam)
    {
        enter(hwnd, at, history, passed_at);
    }
}

/// Windows says the pointer left our guarded window `hwnd`, under its capture or not.
#[allow(unsafe_code, reason = "A plain capture query.")]
pub(super) fn left(hwnd: HWND) {
    if watched(hwnd) {
        // SAFETY: a plain query of this thread's capture.
        let captured = unsafe { GetCapture() } == hwnd;
        see(hwnd, Seen::Leave { captured });
    }
}

/// Our guarded window `hwnd` lost the capture, with the pointer on it or not.
pub(super) fn released(hwnd: HWND) {
    if watched(hwnd) {
        let on = pointer_on(hwnd);
        see(hwnd, Seen::Lost { on });
    }
}

/// Applies what Windows saw on our window `hwnd` when the guard watches it; true when it makes an entry.
pub(super) fn see(hwnd: HWND, seen: Seen) -> bool {
    watched(hwnd) && apply(hwnd, seen) == Ok(true)
}

/// Takes the watch's step after `seen` on `hwnd`, asking for the next leave when it says; true for an entry.
///
/// # Errors
/// `Refused` when Windows will not keep the new state, which then asks and enters nothing.
fn apply(hwnd: HWND, seen: Seen) -> Result<bool, UiError> {
    let now = Inside::from_mark(value(hwnd, INSIDE));
    let after = step(now, seen);
    if after.next != now {
        store(hwnd, after.next)?;
    }
    if after.track {
        track(hwnd);
    }
    Ok(after.entry)
}

/// Keeps `inside` on our window `hwnd`; `Out` keeps no property.
///
/// # Errors
/// `Refused` when Windows says no.
fn store(hwnd: HWND, inside: Inside) -> Result<(), UiError> {
    if inside == Inside::Out {
        clear(hwnd, INSIDE);
        Ok(())
    } else {
        put(hwnd, INSIDE, inside.mark())
    }
}

/// Drops the watch's marks from `hwnd` as it goes, and the entry it kept.
pub(super) fn forget(hwnd: HWND) {
    if watched(hwnd) {
        *lock(&KEPT) = None;
    }
    drop_marks(hwnd);
}

/// Removes the watch's marks from `hwnd`, so the guard no longer watches it.
fn drop_marks(hwnd: HWND) {
    clear(hwnd, WATCH);
    clear(hwnd, INSIDE);
}

/// Keeps the point before `at`, where the pointer entered `hwnd`, from `history` from `at` back, passing over our
/// windows that do not page (`ours`); a history outside the keyboard only on them keeps the last entry (`Kept::after`).
pub(super) fn enter(
    hwnd: HWND,
    at: Point,
    history: impl FnOnce(Point) -> Vec<(i32, i32)>,
    ours: impl Fn(Point) -> bool,
) {
    let Some(keyboard) = bounds(hwnd) else {
        return;
    };
    let screens = WinScreens.work_areas();
    let now = Kept::past(&history(at), at, keyboard, &screens, ours).after(kept());
    *lock(&KEPT) = Some(now);
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
fn pointer_on(hwnd: HWND) -> bool {
    cursor().is_ok_and(|p| window_at(p).1 == hwnd)
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
