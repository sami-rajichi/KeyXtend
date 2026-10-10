//! Keeps our guarded windows in the topmost band: a window lifted back brings its guarded owners and the windows they
//! own back too, each above its owner. It runs on the windows' own thread, from the guard's procedure.

use std::cell::RefCell;

use kx_platform::ui::UiError;
use windows::Win32::Foundation::HWND;
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::WindowsAndMessaging::{
    GW_OWNER, GetWindow, GetWindowThreadProcessId, HWND_TOPMOST, SET_WINDOW_POS_FLAGS,
    SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOOWNERZORDER, SWP_NOSIZE, SetWindowPos,
};

use super::guard::{ex_of, guarded};
use super::win::top_level;
use super::{family, is_top, to_lift};
use crate::refused;

thread_local! {
    /// The windows the guard is lifting back into the topmost band: a move inside the lift of a window or its owner
    /// never lifts again, but another window's repair still runs, so the nesting stays as deep as our windows are many.
    static LIFTING: RefCell<Vec<HWND>> = const { RefCell::new(Vec::new()) };
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
/// from another program does. The furthest guarded owner goes up, then every window it owns, each above its owner.
pub(super) fn keep_top(hwnd: HWND) {
    let chain = chain(hwnd);
    let states: Vec<(bool, bool)> = LIFTING.with_borrow(|l| {
        let state = |w: &HWND| (is_top(ex_of(*w)), l.contains(w));
        chain.iter().map(state).collect()
    });
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
