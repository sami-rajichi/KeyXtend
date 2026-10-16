//! Keeps our guarded windows in the topmost band: a window lifted back brings its guarded owners and the windows they
//! own back too, each above its owner. It runs on the windows' own thread, which also lifts or lays named ones here.

use std::cell::{Cell, RefCell};
use std::sync::OnceLock;

use kx_platform::ui::UiError;
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::WindowsAndMessaging::{
    GW_OWNER, GetWindow, GetWindowThreadProcessId, HWND_TOPMOST, PostMessageW,
    RegisterWindowMessageW, SET_WINDOW_POS_FLAGS, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOOWNERZORDER,
    SWP_NOSIZE, SetWindowPos,
};
use windows::core::{PCWSTR, w};

use super::band::{dropped, family, is_top, to_lift};
use super::guard::{ex_of, guarded};
use super::win::{own_titled, own_windows, title, top_level};
use crate::refused;

/// The name of the message the guard posts to check a window again after the window has answered the move.
const RECHECK: PCWSTR = w!("KeyXtend.Guard.Recheck");

thread_local! {
    /// The windows the guard is lifting back into the topmost band: a move inside the lift of a window or its owner
    /// never lifts again, but another window's repair still runs, so the nesting stays as deep as our windows are many.
    static LIFTING: RefCell<Vec<HWND>> = const { RefCell::new(Vec::new()) };
    /// True while a posted check runs: the moves it makes post no more checks, so the checks always end.
    static RECHECKING: Cell<bool> = const { Cell::new(false) };
}

/// The id of the guard's check-again message; `None` when Windows gave none.
#[allow(unsafe_code, reason = "Registers a message name.")]
pub(super) fn recheck_id() -> Option<u32> {
    static ID: OnceLock<u32> = OnceLock::new();
    // SAFETY: registers a constant name; Windows gives the same id for it every time.
    let id = *ID.get_or_init(|| unsafe { RegisterWindowMessageW(RECHECK) });
    (id != 0).then_some(id)
}

/// Checks our window `hwnd` again from its queue after it has answered the move, as a lift made inside another
/// program's call may not land. The check runs once, even if that program is still busy; a posted check posts none.
#[allow(unsafe_code, reason = "Posts a message to our own window.")]
fn post_recheck(hwnd: HWND) {
    let Some(id) = recheck_id().filter(|_| !RECHECKING.get()) else {
        return;
    };
    // SAFETY: a message with no data to our own window; a gone window refuses it, which needs nothing more.
    let _ = unsafe { PostMessageW(Some(hwnd), id, WPARAM(0), LPARAM(0)) };
}

/// The posted check of our window `hwnd`: the guard lifts it back if it, or an owner, is still out of the band.
pub(super) fn recheck(hwnd: HWND) {
    let was = RECHECKING.replace(true);
    keep_top(hwnd);
    RECHECKING.set(was);
}

/// True while the guard lifts windows back on this thread.
#[cfg(test)]
pub(super) fn lifting() -> bool {
    LIFTING.with_borrow(|l| !l.is_empty())
}

/// True when `hwnd` belongs to the calling thread.
#[allow(unsafe_code, reason = "Plain thread queries.")]
pub(super) fn on_this_thread(hwnd: HWND) -> bool {
    // SAFETY: plain queries; a gone window gives thread 0, which is never ours.
    unsafe { GetWindowThreadProcessId(hwnd, None) == GetCurrentThreadId() }
}

/// Lifts our window, and the windows it owns, to the top of the topmost band.
pub(super) fn lift(hwnd: HWND) -> Result<(), UiError> {
    to_top(hwnd, SET_WINDOW_POS_FLAGS::default())
}

/// Lifts our window alone to the top of the topmost band, such as a tip as it shows; its owner stays where it is.
pub(super) fn front(hwnd: HWND) -> Result<(), UiError> {
    to_top(hwnd, SWP_NOOWNERZORDER)
}

/// Puts our window just under `above` in the z-order, without moving or activating it; its owner stays where it is.
#[allow(unsafe_code, reason = "Changes our own window's z-order.")]
pub(super) fn below(hwnd: HWND, above: HWND) -> Result<(), UiError> {
    let flags = SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_NOOWNERZORDER;
    // SAFETY: our own live window; no move, no size and no activation.
    unsafe { SetWindowPos(hwnd, Some(above), 0, 0, 0, 0, flags) }.map_err(|e| refused(&e))
}

/// Puts our window at the top of the topmost band with the `extra` flags, without moving or activating it.
#[allow(unsafe_code, reason = "Changes our own window's z-order.")]
fn to_top(hwnd: HWND, extra: SET_WINDOW_POS_FLAGS) -> Result<(), UiError> {
    let flags = SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | extra;
    // SAFETY: our own live window; no move, no size and no activation.
    unsafe { SetWindowPos(hwnd, Some(HWND_TOPMOST), 0, 0, 0, 0, flags) }.map_err(|e| refused(&e))
}

/// The window that owns `hwnd`, when the guard holds it on this thread; another thread's window is never followed.
#[allow(unsafe_code, reason = "A plain owner query.")]
fn guarded_owner(hwnd: HWND) -> Option<HWND> {
    // SAFETY: a plain query; a window with no owner gives an error.
    let owner = unsafe { GetWindow(hwnd, GW_OWNER) }.ok()?;
    (!owner.is_invalid() && guarded(owner) && on_this_thread(owner)).then_some(owner)
}

/// Our window, then each owner the guard holds on this thread, nearest first.
pub(super) fn chain(hwnd: HWND) -> Vec<HWND> {
    let mut chain = vec![hwnd];
    while let Some(owner) = chain.last().copied().and_then(guarded_owner) {
        if chain.contains(&owner) {
            break;
        }
        chain.push(owner);
    }
    chain
}

/// Our guarded windows on this thread with their guarded owners, front first.
fn owned_here() -> Vec<(HWND, HWND)> {
    let ours = top_level()
        .into_iter()
        .filter(|&w| guarded(w) && on_this_thread(w));
    ours.filter_map(|w| guarded_owner(w).map(|o| (w, o)))
        .collect()
}

/// Lifts our window back to the top of the topmost band when a move took it, or an owner, out, as `HWND_NOTOPMOST`
/// from another program does. The furthest guarded owner goes up, then every window it owns, each above its owner;
/// the window is checked again after it has answered the move.
pub(super) fn keep_top(hwnd: HWND) {
    let chain = chain(hwnd);
    let states: Vec<(bool, bool)> = LIFTING.with_borrow(|l| {
        let state = |w: &HWND| (is_top(ex_of(*w)), l.contains(w));
        chain.iter().map(state).collect()
    });
    if dropped(&states) {
        post_recheck(hwnd);
    }
    let Some(root) = to_lift(&states).map(|i| chain[i]) else {
        return;
    };
    LIFTING.with_borrow_mut(|l| l.extend(&chain));
    // A refused move leaves nothing more to try for that window; the others still go up.
    let _ = lift(root);
    for w in family(root, &owned_here()) {
        let _ = front(w);
    }
    LIFTING.with_borrow_mut(|l| l.retain(|w| !chain.contains(w)));
}

/// Lifts our shown windows titled `title` alone to the top of the topmost band, never activating them and leaving
/// their owners where they are: the guard may have lifted a banner or the list after a tip's owner.
///
/// # Errors
/// `NoWindow` while none shows, `Refused` when Windows says no.
pub(super) fn lift_alone(title_is: &str) -> Result<(), UiError> {
    shown_titled(title_is)?
        .into_iter()
        .rev()
        .try_for_each(front)
}

/// Puts our shown windows titled `title` just under our window titled `under`, in their order, never activating them
/// and leaving their owners where they are.
///
/// # Errors
/// `NoWindow` while either is not shown, `Refused` when Windows says no.
pub(super) fn lay_under(title_is: &str, under: &str) -> Result<(), UiError> {
    let above = own_titled(under).ok_or(UiError::NoWindow)?;
    let named = shown_titled(title_is)?;
    named.into_iter().rev().try_for_each(|w| below(w, above))
}

/// Our shown windows titled `title`, front first.
///
/// # Errors
/// `NoWindow` while none shows.
fn shown_titled(title_is: &str) -> Result<Vec<HWND>, UiError> {
    let named: Vec<HWND> = own_windows()
        .into_iter()
        .filter(|&w| title(w) == title_is)
        .collect();
    if named.is_empty() {
        Err(UiError::NoWindow)
    } else {
        Ok(named)
    }
}
