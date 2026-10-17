//! The numbers the guard and the entry watch keep on our windows as window properties, which any thread may read
//! without a message.

use kx_platform::ui::UiError;
use windows::Win32::Foundation::{HANDLE, HWND};
use windows::Win32::UI::WindowsAndMessaging::{GetPropW, RemovePropW, SetPropW};
use windows::core::PCWSTR;

use crate::refused;

/// The number `hwnd` holds in property `name`; 0 without it, or for a gone window.
#[allow(unsafe_code, reason = "A plain window property query.")]
pub(super) fn value(hwnd: HWND, name: PCWSTR) -> usize {
    // SAFETY: a plain query; a window without the property gives null.
    unsafe { GetPropW(hwnd, name) }.0.addr()
}

/// Stores `number`, never 0, in property `name` of our window `hwnd`.
///
/// # Errors
/// `Refused` when Windows says no.
#[allow(unsafe_code, reason = "Stores a number on our own window.")]
pub(super) fn put(hwnd: HWND, name: PCWSTR, number: usize) -> Result<(), UiError> {
    let mark = HANDLE(std::ptr::without_provenance_mut(number));
    // SAFETY: stores a number on our own window; `clear` removes it before the window goes.
    unsafe { SetPropW(hwnd, name, Some(mark)) }.map_err(|e| refused(&e))
}

/// Removes property `name` from our window `hwnd`, if it is there.
#[allow(unsafe_code, reason = "Removes our own window's property.")]
pub(super) fn clear(hwnd: HWND, name: PCWSTR) {
    // SAFETY: removes a property the guard set on our own window; a missing one gives an error, which is fine.
    let _ = unsafe { RemovePropW(hwnd, name) };
}
