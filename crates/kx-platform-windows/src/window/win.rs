//! The Windows side of the guard: our windows and their titles, the focus, and the shape that takes clicks.

use kx_module_api::ports::{Point, Rect};
use kx_platform::ui::{FocusToken, UiError, WindowGuard};
use windows::Win32::Foundation::{HWND, LPARAM, POINT, RECT};
use windows::Win32::Graphics::Gdi::{
    CombineRgn, CreateRectRgn, DeleteObject, HRGN, RGN_ERROR, RGN_OR, SetWindowRgn,
};
use windows::Win32::System::Console::GetConsoleWindow;
use windows::Win32::System::Threading::GetCurrentProcessId;
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GA_ROOT, GetAncestor, GetForegroundWindow, GetWindowRect,
    GetWindowThreadProcessId, InternalGetWindowText, IsWindow, IsWindowVisible,
    SetForegroundWindow, WindowFromPoint,
};
use windows::core::BOOL;

use super::entry;
use super::guard::guard;
use super::top::{lay_under, lift_alone};
use super::{guard_order, ours_by, should_give_back, spared};
use crate::screen::edges;
use crate::{failed, refused};

/// Longest window title read, in UTF-16 units.
const TITLE_CAP: usize = 512;

/// The Windows window guard. Call it on the thread that owns our windows, as Windows subclasses only there.
#[derive(Clone, Copy, Debug, Default)]
pub struct WinGuard;

impl WindowGuard for WinGuard {
    fn guard_own(&self, spare: Option<&str>, top: Option<&str>) -> Result<usize, UiError> {
        guard_own(spare, top)
    }

    fn shape(&self, title: &str, rects: &[Rect]) -> Result<(), UiError> {
        shape(title, rects)
    }

    fn lift_alone(&self, title: &str) -> Result<(), UiError> {
        lift_alone(title)
    }

    fn watch_entries(&self, title: &str) -> Result<(), UiError> {
        entry::watch_entries(title)
    }

    fn unwatch_entries(&self, title: &str) {
        entry::unwatch_entries(title);
    }

    fn lay_under(&self, title: &str, under: &str) -> Result<(), UiError> {
        lay_under(title, under)
    }

    fn remember(&self) -> FocusToken {
        token_of(foreground())
    }

    fn give_back(&self, token: FocusToken) {
        if let Some(noted) = window_of(token) {
            give_back(noted);
        }
    }
}

/// `hwnd` as a focus token.
pub(super) fn token_of(hwnd: HWND) -> FocusToken {
    FocusToken::new(u64::try_from(hwnd.0.addr()).unwrap_or_default())
}

/// The window `token` names; `None` when it does not fit a handle.
pub(super) fn window_of(token: FocusToken) -> Option<HWND> {
    let addr = usize::try_from(token.raw()).ok()?;
    Some(HWND(std::ptr::without_provenance_mut(addr)))
}

/// The window in front now; null when there is none.
#[must_use]
#[allow(unsafe_code, reason = "A plain window query.")]
pub fn foreground() -> HWND {
    // SAFETY: a plain query.
    unsafe { GetForegroundWindow() }
}

/// A window handle from its raw value, as kept to cross threads.
#[must_use]
pub fn from_raw(value: isize) -> HWND {
    HWND(std::ptr::without_provenance_mut(value.cast_unsigned()))
}

/// True when `hwnd` is a window of this process, other than the console host's.
#[must_use]
#[allow(unsafe_code, reason = "Plain window and process queries.")]
pub fn is_ours(hwnd: HWND) -> bool {
    let mut pid = 0u32;
    // SAFETY: plain queries; `pid` is a local that outlives the call.
    let (me, console) = unsafe {
        GetWindowThreadProcessId(hwnd, Some(&raw mut pid));
        (GetCurrentProcessId(), GetConsoleWindow())
    };
    ours_by(pid, me, hwnd == console)
}

/// The window at screen point `p` and its top-level window; both null where there is none.
#[allow(unsafe_code, reason = "A plain window query.")]
pub(crate) fn window_at(p: Point) -> (HWND, HWND) {
    let hwnd = hit(p);
    // SAFETY: a plain query; null has no top-level window.
    (hwnd, unsafe { GetAncestor(hwnd, GA_ROOT) })
}

/// True when the window at screen point `p` is one of ours, such as the banner; one query, cheap enough for the hook.
pub(crate) fn on_ours(p: Point) -> bool {
    is_ours(hit(p))
}

/// The window at screen point `p`; null where there is none.
#[allow(unsafe_code, reason = "A plain window query.")]
fn hit(p: Point) -> HWND {
    // SAFETY: a plain query by value; it sends no message to another thread's window.
    unsafe { WindowFromPoint(POINT { x: p.x, y: p.y }) }
}

/// True while `hwnd` is shown.
#[allow(unsafe_code, reason = "A plain window query.")]
fn visible(hwnd: HWND) -> bool {
    // SAFETY: a plain query; a gone window reads as hidden.
    unsafe { IsWindowVisible(hwnd) }.as_bool()
}

/// This process's visible top-level windows, front first.
#[must_use]
pub fn own_windows() -> Vec<HWND> {
    let mut found = top_level();
    found.retain(|&w| visible(w) && is_ours(w));
    found
}

/// Every top-level window, hidden ones too, front first: one snapshot of the z-order.
#[allow(
    unsafe_code,
    reason = "Enumerates top-level windows into a local list."
)]
pub(super) fn top_level() -> Vec<HWND> {
    unsafe extern "system" fn collect(hwnd: HWND, list: LPARAM) -> BOOL {
        let found = std::ptr::with_exposed_provenance_mut::<Vec<HWND>>(list.0.cast_unsigned());
        // SAFETY: `list` is the address of the Vec passed below, alive and not borrowed during the call.
        unsafe { (*found).push(hwnd) };
        BOOL::from(true)
    }
    let mut found: Vec<HWND> = Vec::new();
    let list = LPARAM(
        std::ptr::from_mut(&mut found)
            .expose_provenance()
            .cast_signed(),
    );
    // SAFETY: the callback touches only `found`, which outlives the call; a failure leaves it as it is.
    let _ = unsafe { EnumWindows(Some(collect), list) };
    found
}

/// The title of `hwnd`, at most `TITLE_CAP` units. It sends the window no message, so any thread may
/// read it while the window's own thread is busy.
#[must_use]
#[allow(unsafe_code, reason = "A title query into a local buffer.")]
pub fn title(hwnd: HWND) -> String {
    let mut buf = [0u16; TITLE_CAP];
    // SAFETY: Windows writes at most the buffer's length.
    let n = unsafe { InternalGetWindowText(hwnd, &mut buf) };
    let n = usize::try_from(n).unwrap_or_default();
    String::from_utf16_lossy(buf.get(..n).unwrap_or_default())
}

/// Our visible window titled `title_is`.
#[must_use]
pub fn own_titled(title_is: &str) -> Option<HWND> {
    own_windows().into_iter().find(|&w| title(w) == title_is)
}

/// The box of our visible window titled `title_is`.
///
/// # Errors
/// `NoWindow` while it is not shown, `Refused` when Windows will not say.
#[allow(unsafe_code, reason = "A box query into a local.")]
pub fn own_bounds(title_is: &str) -> Result<RECT, UiError> {
    let hwnd = own_titled(title_is).ok_or(UiError::NoWindow)?;
    let mut r = RECT::default();
    // SAFETY: one of our live windows and a local rect.
    unsafe { GetWindowRect(hwnd, &raw mut r) }.map_err(|e| refused(&e))?;
    Ok(r)
}

/// Guards our visible windows except the one titled `spare`, back to front, with the one titled `top` last
/// so it ends on top; says how many it guarded.
///
/// # Errors
/// `Refused` when Windows will not guard a window, as from a thread that does not own it.
pub fn guard_own(spare: Option<&str>, top: Option<&str>) -> Result<usize, UiError> {
    guard_these(own_windows(), spare, top)
}

/// Guards `front_first`, our windows listed front first, as `guard_own` does its shown ones.
///
/// # Errors
/// `Refused` when Windows will not guard a window.
pub(super) fn guard_these(
    front_first: Vec<HWND>,
    spare: Option<&str>,
    top: Option<&str>,
) -> Result<usize, UiError> {
    let named = front_first.into_iter().map(|w| (w, title(w)));
    let kept: Vec<(HWND, String)> = named.filter(|(_, t)| !spared(t, spare)).collect();
    let order = guard_order(kept, |(_, t)| top == Some(t.as_str()));
    for (hwnd, _) in &order {
        guard(*hwnd)?;
    }
    Ok(order.len())
}

/// Cuts our window titled `title_is` to `rects` (physical pixels from its top-left corner), so clicks
/// elsewhere reach the app below; Windows also draws nothing outside them.
///
/// # Errors
/// `NoWindow` for a window not shown, `Refused` when Windows says no.
#[allow(unsafe_code, reason = "Hands a region to our own window.")]
pub fn shape(title_is: &str, rects: &[Rect]) -> Result<(), UiError> {
    let hwnd = own_titled(title_is).ok_or(UiError::NoWindow)?;
    let all = region(rects)?;
    // SAFETY: `all` is ours until SetWindowRgn takes it; `hwnd` is one of our live windows.
    if unsafe { SetWindowRgn(hwnd, Some(all), true) } == 0 {
        // SAFETY: Windows did not take the region, so we delete it, once.
        let _ = unsafe { DeleteObject(all.into()) };
        return Err(failed());
    }
    Ok(())
}

/// One region covering every rect; the caller owns it.
#[allow(unsafe_code, reason = "Makes a region and deletes it on failure.")]
pub(super) fn region(rects: &[Rect]) -> Result<HRGN, UiError> {
    // SAFETY: makes an empty region, which the caller owns.
    let all = unsafe { CreateRectRgn(0, 0, 0, 0) };
    if all.is_invalid() {
        return Err(failed());
    }
    for r in rects {
        if let Err(e) = add(all, *r) {
            // SAFETY: deletes the region made above, once.
            let _ = unsafe { DeleteObject(all.into()) };
            return Err(e);
        }
    }
    Ok(all)
}

/// Adds rect `r` to region `all`.
#[allow(unsafe_code, reason = "Makes, joins and deletes one region.")]
fn add(all: HRGN, r: Rect) -> Result<(), UiError> {
    let [left, top, right, bottom] = edges(r);
    // SAFETY: makes one region, deleted below.
    let one = unsafe { CreateRectRgn(left, top, right, bottom) };
    if one.is_invalid() {
        return Err(failed());
    }
    // SAFETY: both regions are ours and live.
    let joined = unsafe { CombineRgn(Some(all), Some(all), Some(one), RGN_OR) };
    let result = if joined == RGN_ERROR {
        Err(failed())
    } else {
        Ok(())
    };
    // SAFETY: deletes the region made above, once.
    let _ = unsafe { DeleteObject(one.into()) };
    result
}

/// Gives the focus back to `noted` when one of ours took it and `noted` is another app's live window.
#[allow(unsafe_code, reason = "A focus change Windows allows us.")]
pub fn give_back(noted: HWND) {
    if gives_back(noted) {
        // SAFETY: one of our windows is in front, so Windows allows the change.
        let _ = unsafe { SetForegroundWindow(noted) };
    }
}

/// True when the focus should go back to `noted`: one of ours is in front and `noted` is another app's live window.
#[allow(unsafe_code, reason = "A plain window query.")]
pub(super) fn gives_back(noted: HWND) -> bool {
    // SAFETY: a plain query; a null or gone window reads as not a window.
    let alive = unsafe { IsWindow(Some(noted)) }.as_bool();
    should_give_back(alive, is_ours(foreground()), is_ours(noted))
}
