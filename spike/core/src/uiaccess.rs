//! Reads whether this process really runs with uiAccess.

use std::ffi::c_void;

use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::Security::{GetTokenInformation, TOKEN_QUERY, TokenUIAccess};
use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

/// True if the process token has the uiAccess flag.
pub fn active() -> bool {
    let mut token = HANDLE::default();
    let mut value: u32 = 0;
    let mut len = 0u32;
    // SAFETY: the token handle is ours and closed below; `value` is a 4-byte buffer.
    unsafe {
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token).is_err() {
            return false;
        }
        let buf = Some(std::ptr::from_mut(&mut value).cast::<c_void>());
        let ok = GetTokenInformation(token, TokenUIAccess, buf, 4, &mut len).is_ok();
        let _ = CloseHandle(token);
        ok && value != 0
    }
}
