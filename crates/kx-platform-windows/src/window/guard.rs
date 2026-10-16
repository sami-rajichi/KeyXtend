//! The subclass that keeps our window from taking the focus and in the topmost band; it unhooks itself when the window
//! is destroyed. It uses user32 only (`GWLP_WNDPROC` and window properties), so no program needs a Common Controls 6
//! manifest.

use kx_platform::ui::UiError;
use windows::Win32::Foundation::{ERROR_INVALID_THREAD_ID, HANDLE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::Controls::WM_MOUSELEAVE;
use windows::Win32::UI::WindowsAndMessaging::{
    CallWindowProcW, DefWindowProcW, GWL_EXSTYLE, GWLP_WNDPROC, GetPropW, GetWindowLongPtrW,
    MA_NOACTIVATE, RemovePropW, STYLESTRUCT, SWP_NOACTIVATE, SetPropW, SetWindowLongPtrW,
    WINDOWPOS, WM_MOUSEACTIVATE, WM_MOUSEMOVE, WM_NCDESTROY, WM_STYLECHANGING, WM_WINDOWPOSCHANGED,
    WM_WINDOWPOSCHANGING, WNDPROC,
};
use windows::core::{PCWSTR, w};

use super::entry;
use super::top::{keep_top, lift, on_this_thread, recheck, recheck_id};
use super::{KEPT, holds_kept, is_through, kept_ex};
use crate::{last_refused, refused};

/// The window property that holds the window's own procedure while the guard is hooked.
const PROP: PCWSTR = w!("KeyXtend.Guard");
/// The window property set while the guard is hooked on a window that let clicks through when it was first guarded.
const THROUGH_PROP: PCWSTR = w!("KeyXtend.Through");

/// A window procedure, as Windows calls it.
pub(super) type Proc = unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> LRESULT;

/// Makes our window `hwnd` non-activating and topmost, and keeps it so until it is destroyed.
/// Windows lets only the owning thread change a window's procedure; another thread is refused.
pub(super) fn guard(hwnd: HWND) -> Result<(), UiError> {
    if !on_this_thread(hwnd) {
        return Err(UiError::Refused(ERROR_INVALID_THREAD_ID.to_hresult().0));
    }
    hook(hwnd)?;
    add_kept(hwnd)?;
    lift(hwnd)
}

/// True while the guard is hooked on `hwnd`; another program may ask too, as a test tool does.
#[must_use]
#[allow(unsafe_code, reason = "A plain window property query.")]
pub fn guarded(hwnd: HWND) -> bool {
    // SAFETY: a plain query; a gone window has no property.
    !unsafe { GetPropW(hwnd, PROP) }.is_invalid()
}

/// True when our window let clicks through as the guard first saw it.
#[allow(unsafe_code, reason = "A plain window property query.")]
fn was_through(hwnd: HWND) -> bool {
    // SAFETY: a plain query; a window without the property gives null.
    !unsafe { GetPropW(hwnd, THROUGH_PROP) }.is_invalid()
}

/// Our window's extended style; 0 for a gone window.
#[allow(unsafe_code, reason = "A plain style query.")]
pub(super) fn ex_of(hwnd: HWND) -> u32 {
    // SAFETY: a plain query of our own window.
    let ex = unsafe { GetWindowLongPtrW(hwnd, GWL_EXSTYLE) };
    u32::try_from(ex).unwrap_or_default()
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
    keep_props(hwnd, own)?;
    // SAFETY: our procedure is static and hands every message on to the kept one.
    if unsafe { SetWindowLongPtrW(hwnd, GWLP_WNDPROC, ours()) } == 0 {
        let refusal = last_refused();
        drop_props(hwnd);
        return Err(refusal);
    }
    Ok(())
}

/// Stores the window's own procedure `own` on it, and marks it when it lets clicks through now.
#[allow(unsafe_code, reason = "Stores two numbers on our own window.")]
fn keep_props(hwnd: HWND, own: isize) -> Result<(), UiError> {
    let keep = HANDLE(std::ptr::without_provenance_mut(own.cast_unsigned()));
    // SAFETY: stores a number on our own window; `unhook` removes it before the window goes.
    unsafe { SetPropW(hwnd, PROP, Some(keep)) }.map_err(|e| refused(&e))?;
    if !is_through(ex_of(hwnd)) {
        return Ok(());
    }
    let mark = HANDLE(std::ptr::without_provenance_mut(1));
    // SAFETY: stores a marker number on our own window; `unhook` removes it too.
    unsafe { SetPropW(hwnd, THROUGH_PROP, Some(mark)) }.map_err(|e| {
        drop_props(hwnd);
        refused(&e)
    })
}

/// Removes the guard's properties from our window; a missing one is skipped.
#[allow(unsafe_code, reason = "Removes our own window's properties.")]
fn drop_props(hwnd: HWND) {
    // SAFETY: removes properties the guard set on our own window, once each.
    unsafe {
        let _ = RemovePropW(hwnd, PROP);
        let _ = RemovePropW(hwnd, THROUGH_PROP);
    }
}

/// Puts the window's own procedure back, if ours is still in front, and removes the properties.
#[allow(
    unsafe_code,
    reason = "Restores our own window's procedure as it goes."
)]
fn unhook(hwnd: HWND, own: HANDLE) {
    // SAFETY: plain query and change on our own window during its last message; another subclass in front stays.
    unsafe {
        if GetWindowLongPtrW(hwnd, GWLP_WNDPROC) == ours() {
            // A refused restore leaves nothing more to try as the window goes.
            let _ = SetWindowLongPtrW(hwnd, GWLP_WNDPROC, own.0.addr().cast_signed());
        }
    }
    drop_props(hwnd);
    entry::forget(hwnd);
}

/// Adds the kept styles, refused when they did not land; the subclass then keeps them against later changes.
#[allow(
    unsafe_code,
    reason = "Reads and sets our own window's extended style."
)]
pub(super) fn add_kept(hwnd: HWND) -> Result<(), UiError> {
    // The kept styles fit in 31 bits, so this never falls back.
    let kept = isize::try_from(KEPT).unwrap_or_default();
    // SAFETY: plain style query and change on our own live window.
    unsafe {
        let ex = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
        SetWindowLongPtrW(hwnd, GWL_EXSTYLE, ex | kept);
    }
    if holds_kept(ex_of(hwnd)) {
        Ok(())
    } else {
        Err(last_refused())
    }
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

/// Keeps every show and move passive and in the topmost band, answers clicks with no activation, keeps the styles,
/// watches the pointer's entries and unhooks at the end; the guard's own posted check goes no further.
#[allow(unsafe_code, reason = "The procedure of our guarded windows.")]
unsafe extern "system" fn proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if recheck_id() == Some(msg) {
        recheck(hwnd);
        return LRESULT(0);
    }
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
                style.styleNew = kept_ex(style.styleNew, was_through(hwnd));
            }
        }
        WM_MOUSEMOVE => entry::moved(hwnd, lparam),
        WM_MOUSELEAVE => entry::left(hwnd),
        WM_NCDESTROY => unhook(hwnd, own),
        _ => {}
    }
    let answer = match next(own) {
        // SAFETY: hands the message to the window's own procedure, as a subclass must.
        Some(own) => unsafe { CallWindowProcW(Some(own), hwnd, msg, wparam, lparam) },
        // SAFETY: with no kept procedure, Windows' default handling is the safe answer.
        None => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    };
    if msg == WM_WINDOWPOSCHANGED {
        keep_top(hwnd);
    }
    answer
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
