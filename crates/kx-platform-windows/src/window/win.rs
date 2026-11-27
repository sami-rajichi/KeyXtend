//! The Windows side of the guard: the guard port, and our windows, their titles and where they stand.

use kx_module_api::ports::{Point, Rect};
use kx_platform::ui::{FocusToken, UiError, Watched, WindowGuard};
use windows::Win32::Foundation::{HWND, LPARAM, POINT, RECT};
use windows::Win32::Graphics::Dwm::{DWMWA_CLOAKED, DwmGetWindowAttribute};
use windows::Win32::System::Console::GetConsoleWindow;
use windows::Win32::System::Threading::GetCurrentProcessId;
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GA_ROOT, GW_OWNER, GWL_EXSTYLE, GetAncestor, GetWindow, GetWindowLongPtrW,
    GetWindowRect, GetWindowThreadProcessId, InternalGetWindowText, IsWindowVisible,
    WindowFromPoint,
};
use windows::core::BOOL;

use super::focus::{foreground, give_back, token_of, window_of};
use super::guard::guard;
use super::shape::shape;
use super::top::{lay_low, lay_under, lift_alone};
use super::{entry, pages, shield};
use super::{guard_order, ours_by, spared};
use crate::refused;

/// Longest window title read, in UTF-16 units.
const TITLE_CAP: usize = 512;

/// The Windows window guard. Call it on the thread that owns our windows, as Windows subclasses only there.
#[derive(Clone, Copy, Debug, Default)]
pub struct WinGuard;

impl WindowGuard for WinGuard {
    fn guard_own(&self, spare: &[&str], top: Option<&str>) -> Result<usize, UiError> {
        guard_own(spare, top)
    }

    fn shape(&self, title: &str, rects: &[Rect]) -> Result<(), UiError> {
        shape(title, rects)
    }

    fn lift_alone(&self, title: &str) -> Result<(), UiError> {
        lift_alone(title)
    }

    fn watch_entries(&self, title: &str) -> Result<Watched, UiError> {
        entry::watch_entries(title)
    }

    fn unwatch_entries(&self, title: &str) {
        entry::unwatch_entries(title);
    }

    fn lay_under(&self, title: &str, under: &str) -> Result<(), UiError> {
        lay_under(title, under)
    }

    fn lay_low(&self, title: &str) -> Result<(), UiError> {
        lay_low(title)
    }

    fn mark_pages(&self, title: &str) -> Result<(), UiError> {
        pages::mark_pages(title)
    }

    fn shield(&self, title: &str, on: bool) -> Result<(), UiError> {
        shield::shield(title, on)
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
pub(crate) fn visible(hwnd: HWND) -> bool {
    // SAFETY: a plain query; a gone window reads as hidden.
    unsafe { IsWindowVisible(hwnd) }.as_bool()
}

/// The extended style of `hwnd`; 0 for a gone window.
#[allow(unsafe_code, reason = "A plain style query.")]
pub(crate) fn ex_of(hwnd: HWND) -> u32 {
    // SAFETY: a plain query; a gone window gives 0.
    u32::try_from(unsafe { GetWindowLongPtrW(hwnd, GWL_EXSTYLE) }).unwrap_or_default()
}

/// True while the system cloaks `hwnd`, so it does not show though Windows calls it visible.
#[allow(unsafe_code, reason = "A window attribute read into a local.")]
pub(crate) fn cloaked(hwnd: HWND) -> bool {
    let mut value = 0u32;
    let size = u32::try_from(size_of::<u32>()).unwrap_or_default();
    // SAFETY: the call writes one u32 into a local that outlives it; a refusal reads as not cloaked.
    let read = unsafe { DwmGetWindowAttribute(hwnd, DWMWA_CLOAKED, (&raw mut value).cast(), size) };
    read.is_ok() && value != 0
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
pub(crate) fn top_level() -> Vec<HWND> {
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

/// The window that owns `hwnd`, if any.
#[allow(unsafe_code, reason = "A plain owner query.")]
pub(super) fn owner(hwnd: HWND) -> Option<HWND> {
    // SAFETY: a plain query; a window with no owner gives an error.
    let owner = unsafe { GetWindow(hwnd, GW_OWNER) }.ok()?;
    (!owner.is_invalid()).then_some(owner)
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

/// Guards our visible windows but those titled one of `spare`, with their shadows and tips, back to front, with the
/// one titled `top` last so it ends on top; says how many it guarded.
///
/// # Errors
/// `Refused` when Windows will not guard a window, as from a thread that does not own it.
pub fn guard_own(spare: &[&str], top: Option<&str>) -> Result<usize, UiError> {
    guard_these(own_windows(), spare, top)
}

/// Guards `front_first`, our windows listed front first, as `guard_own` does its shown ones.
///
/// # Errors
/// `Refused` when Windows will not guard a window.
pub(super) fn guard_these(
    front_first: Vec<HWND>,
    spare: &[&str],
    top: Option<&str>,
) -> Result<usize, UiError> {
    let named: Vec<(HWND, String)> = front_first.into_iter().map(|w| (w, title(w))).collect();
    let marks: Vec<_> = named
        .iter()
        .map(|(w, t)| (*w, owner(*w), spare.contains(&t.as_str())))
        .collect();
    let left = spared(&marks);
    let kept = named
        .into_iter()
        .zip(left)
        .filter(|(_, l)| !l)
        .map(|(n, _)| n);
    let order = guard_order(kept.collect(), |(_, t)| top == Some(t.as_str()));
    for (hwnd, _) in &order {
        guard(*hwnd)?;
    }
    Ok(order.len())
}
