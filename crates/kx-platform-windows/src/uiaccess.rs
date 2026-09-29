//! Reads whether this process really runs with uiAccess.

use std::ffi::c_void;

use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::Security::{GetTokenInformation, TOKEN_QUERY, TokenUIAccess};
use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

/// True if the process token has the uiAccess flag; false when Windows will not say.
#[must_use]
#[allow(
    unsafe_code,
    reason = "Reads this process's own token, then closes it."
)]
pub fn active() -> bool {
    let mut token = HANDLE::default();
    let mut value: u32 = 0;
    let mut len = 0u32;
    let Ok(size) = u32::try_from(size_of::<u32>()) else {
        return false;
    };
    // SAFETY: the token handle is ours and closed below; `value` is a buffer of `size` bytes.
    unsafe {
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &raw mut token).is_err() {
            return false;
        }
        let buf = Some(std::ptr::from_mut(&mut value).cast::<c_void>());
        let ok = GetTokenInformation(token, TokenUIAccess, buf, size, &raw mut len).is_ok();
        let _ = CloseHandle(token);
        ok && value != 0
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_test_run_has_no_ui_access() {
        assert!(!super::active());
    }
}
