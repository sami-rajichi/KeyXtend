//! The subclass that keeps our window from taking the focus; it unhooks itself when the window is destroyed.
//! It uses user32 only (`GWLP_WNDPROC` and a window property), so no program needs the Common Controls 6 manifest.

use kx_platform::ui::UiError;
use windows::Win32::Foundation::{ERROR_INVALID_THREAD_ID, HANDLE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::WindowsAndMessaging::{
    CallWindowProcW, DefWindowProcW, GWL_EXSTYLE, GWLP_WNDPROC, GetPropW, GetWindowLongPtrW,
    GetWindowThreadProcessId, HWND_TOPMOST, MA_NOACTIVATE, RemovePropW, SET_WINDOW_POS_FLAGS,
    STYLESTRUCT, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOOWNERZORDER, SWP_NOSIZE, SetPropW,
    SetWindowLongPtrW, SetWindowPos, WINDOWPOS, WM_MOUSEACTIVATE, WM_NCDESTROY, WM_STYLECHANGING,
    WM_WINDOWPOSCHANGING, WNDPROC,
};
use windows::core::{PCWSTR, w};

use super::{KEPT, kept_ex};
use crate::{last_refused, refused};

/// The window property that holds the window's own procedure while the guard is hooked.
const PROP: PCWSTR = w!("KeyXtend.Guard");

/// A window procedure, as Windows calls it.
pub(super) type Proc = unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> LRESULT;

/// Makes our window `hwnd` non-activating and topmost, and keeps it so until it is destroyed.
/// Windows lets only the owning thread change a window's procedure; another thread is refused.
pub(super) fn guard(hwnd: HWND) -> Result<(), UiError> {
    if !on_this_thread(hwnd) {
        return Err(UiError::Refused(ERROR_INVALID_THREAD_ID.to_hresult().0));
    }
    hook(hwnd)?;
    add_kept(hwnd);
    lift(hwnd)
}

/// True while the guard is hooked on `hwnd`.
#[allow(unsafe_code, reason = "A plain window property query.")]
pub(super) fn guarded(hwnd: HWND) -> bool {
    // SAFETY: a plain query; a gone window has no property.
    !unsafe { GetPropW(hwnd, PROP) }.is_invalid()
}

/// True when `hwnd` belongs to the calling thread.
#[allow(unsafe_code, reason = "Plain thread queries.")]
fn on_this_thread(hwnd: HWND) -> bool {
    // SAFETY: plain queries; a gone window gives thread 0, which is never ours.
    unsafe { GetWindowThreadProcessId(hwnd, None) == GetCurrentThreadId() }
}

/// Our procedure as a window-long value.
fn ours() -> isize {
    let ours: Proc = proc;
    (ours as usize).cast_signed()
}

/// Puts our procedure in front of the window's own, which the property keeps; a hooked window stays as it is.
#[allow(unsafe_code, reason = "Replaces our own window's procedure.")]
fn hook(hwnd: HWND) -> Result<(), UiError> {
    if guarded(hwnd) {
        return Ok(());
    }
    // SAFETY: a plain query of our own window's procedure.
    let own = unsafe { GetWindowLongPtrW(hwnd, GWLP_WNDPROC) };
    if own == 0 {
        return Err(last_refused());
    }
    let keep = HANDLE(std::ptr::without_provenance_mut(own.cast_unsigned()));
    // SAFETY: stores a number on our own window; `unhook` removes it before the window goes.
    unsafe { SetPropW(hwnd, PROP, Some(keep)) }.map_err(|e| refused(&e))?;
    // SAFETY: our procedure is static and hands every message on to the kept one.
    if unsafe { SetWindowLongPtrW(hwnd, GWLP_WNDPROC, ours()) } == 0 {
        let refusal = last_refused();
        // SAFETY: removes the property set above, once.
        let _ = unsafe { RemovePropW(hwnd, PROP) };
        return Err(refusal);
    }
    Ok(())
}

/// Puts the window's own procedure back, if ours is still in front, and removes the property.
#[allow(
    unsafe_code,
    reason = "Restores our own window's procedure as it goes."
)]
fn unhook(hwnd: HWND, own: HANDLE) {
    // SAFETY: plain query and change on our own window during its last message; another subclass in front stays.
    unsafe {
        if GetWindowLongPtrW(hwnd, GWLP_WNDPROC) == ours() {
            SetWindowLongPtrW(hwnd, GWLP_WNDPROC, own.0.addr().cast_signed());
        }
        let _ = RemovePropW(hwnd, PROP);
    }
}

/// Adds the kept styles; the subclass then keeps them against later changes.
#[allow(
    unsafe_code,
    reason = "Reads and sets our own window's extended style."
)]
fn add_kept(hwnd: HWND) {
    // The kept styles fit in 31 bits, so this never falls back.
    let kept = isize::try_from(KEPT).unwrap_or_default();
    // SAFETY: plain style query and change on our own live window.
    unsafe {
        let ex = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
        SetWindowLongPtrW(hwnd, GWL_EXSTYLE, ex | kept);
    }
}

/// Lifts our window, and the windows it owns, to the top of the topmost band.
fn lift(hwnd: HWND) -> Result<(), UiError> {
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

/// The style index `WM_STYLECHANGING` carries, from the low half of `wparam`.
fn style_index(wparam: WPARAM) -> i32 {
    let [b0, b1, b2, b3, ..] = wparam.0.to_le_bytes();
    i32::from_le_bytes([b0, b1, b2, b3])
}

/// The struct a message's `lparam` points to.
fn pointee<T>(lparam: LPARAM) -> *mut T {
    std::ptr::with_exposed_provenance_mut(lparam.0.cast_unsigned())
}

/// The kept procedure, for `CallWindowProcW`, which also takes the special values Windows may keep there.
#[allow(
    unsafe_code,
    reason = "Turns the kept procedure value back into a function pointer."
)]
pub(super) fn next(own: HANDLE) -> WNDPROC {
    let code = std::ptr::with_exposed_provenance::<()>(own.0.addr());
    // SAFETY: a non-null pointer the size of a function pointer; only CallWindowProcW uses it.
    (!code.is_null()).then(|| unsafe { std::mem::transmute::<*const (), Proc>(code) })
}

/// Keeps every show and move passive, answers clicks with no activation, keeps the styles, and unhooks at the end.
#[allow(unsafe_code, reason = "The procedure of our guarded windows.")]
unsafe extern "system" fn proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    // SAFETY: a plain query of our own window's property.
    let own = unsafe { GetPropW(hwnd, PROP) };
    match msg {
        WM_MOUSEACTIVATE => return LRESULT(isize::try_from(MA_NOACTIVATE).unwrap_or_default()),
        WM_WINDOWPOSCHANGING => {
            // SAFETY: for this message `lparam` points to a live WINDOWPOS.
            if let Some(pos) = unsafe { pointee::<WINDOWPOS>(lparam).as_mut() } {
                pos.flags |= SWP_NOACTIVATE;
            }
        }
        WM_STYLECHANGING if style_index(wparam) == GWL_EXSTYLE.0 => {
            // SAFETY: for this message `lparam` points to a live STYLESTRUCT.
            if let Some(style) = unsafe { pointee::<STYLESTRUCT>(lparam).as_mut() } {
                style.styleNew = kept_ex(style.styleOld, style.styleNew);
            }
        }
        WM_NCDESTROY => unhook(hwnd, own),
        _ => {}
    }
    match next(own) {
        // SAFETY: hands the message to the window's own procedure, as a subclass must.
        Some(own) => unsafe { CallWindowProcW(Some(own), hwnd, msg, wparam, lparam) },
        // SAFETY: with no kept procedure, Windows' default handling is the safe answer.
        None => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows::Win32::UI::WindowsAndMessaging::GWL_STYLE;

    #[test]
    fn the_style_index_is_read_from_the_low_half_either_way() {
        let signed = WPARAM(isize::try_from(GWL_EXSTYLE.0).unwrap().cast_unsigned());
        assert_eq!(style_index(signed), GWL_EXSTYLE.0, "sign-extended");
        let low = WPARAM(usize::try_from(GWL_EXSTYLE.0.cast_unsigned()).unwrap());
        assert_eq!(style_index(low), GWL_EXSTYLE.0, "zero-extended");
        let plain = WPARAM(isize::try_from(GWL_STYLE.0).unwrap().cast_unsigned());
        assert_ne!(style_index(plain), GWL_EXSTYLE.0);
    }

    #[test]
    fn no_kept_procedure_means_the_default_one() {
        assert!(next(HANDLE::default()).is_none());
    }
}
