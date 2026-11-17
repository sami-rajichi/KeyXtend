//! A `WinHTTP` handle closed once, and setting a number option on one.

use super::codes;
use crate::NetError;
use std::ffi::c_void;
use windows::Win32::Networking::WinHttp::{WinHttpCloseHandle, WinHttpSetOption};

/// An open `WinHTTP` handle, closed once when dropped; only `owned` makes one, so it is never null.
pub(super) struct Handle(*mut c_void);

impl Handle {
    /// The raw handle, for a `WinHTTP` call while `self` lives.
    pub(super) fn raw(&self) -> *mut c_void {
        self.0
    }
}

#[allow(unsafe_code, reason = "Lets a body move to the thread that reads it.")]
// SAFETY: a WinHTTP handle is not tied to the thread that opened it, and its one owner makes one call at a time.
unsafe impl Send for Handle {}

impl Drop for Handle {
    #[allow(unsafe_code, reason = "Closes a WinHTTP handle.")]
    fn drop(&mut self) {
        // SAFETY: the handle came from WinHTTP, is not null, and is closed only here.
        let _ = unsafe { WinHttpCloseHandle(self.0) };
    }
}

/// `h` as an owned handle, or why `WinHTTP` gave none.
///
/// # Errors
/// The kind of the thread's last error.
pub(super) fn owned(h: *mut c_void) -> Result<Handle, NetError> {
    if h.is_null() {
        Err(codes::last())
    } else {
        Ok(Handle(h))
    }
}

/// Sets `option` on `h` to the number `value`.
///
/// # Errors
/// The kind of `WinHTTP`'s error.
#[allow(unsafe_code, reason = "Sets a WinHTTP option.")]
pub(super) fn set(h: &Handle, option: u32, value: u32) -> Result<(), NetError> {
    let bytes = value.to_le_bytes();
    // SAFETY: `h` is open, and every option set here takes a DWORD, which `bytes` holds.
    unsafe { WinHttpSetOption(Some(h.raw()), option, Some(&bytes)) }.map_err(|e| codes::of(&e))
}
