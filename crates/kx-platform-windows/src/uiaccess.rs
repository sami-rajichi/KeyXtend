//! Reads whether this process really runs with uiAccess.

use std::ffi::c_void;
use std::io;

use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::Security::{GetTokenInformation, TOKEN_QUERY, TokenUIAccess};
use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

use crate::PlatformError;

/// Whether the process token has the uiAccess flag.
///
/// # Errors
/// `PlatformError::UiAccess` when Windows will not say, so a caller that must fail closed can refuse.
#[allow(
    unsafe_code,
    reason = "Reads this process's own token, then closes it."
)]
pub fn check() -> Result<bool, PlatformError> {
    let mut token = HANDLE::default();
    let mut value: u32 = 0;
    let mut len = 0u32;
    let size = u32::try_from(size_of::<u32>())
        .map_err(|_| PlatformError::UiAccess(io::ErrorKind::InvalidInput.into()))?;
    // SAFETY: the token handle is ours and closed below; `value` is a buffer of `size` bytes.
    let read = unsafe {
        OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &raw mut token)
            .map_err(|e| PlatformError::UiAccess(e.into()))?;
        let buf = Some(std::ptr::from_mut(&mut value).cast::<c_void>());
        let read = GetTokenInformation(token, TokenUIAccess, buf, size, &raw mut len);
        let _ = CloseHandle(token);
        read
    };
    read.map(|()| value != 0)
        .map_err(|e| PlatformError::UiAccess(e.into()))
}

/// True if the process token has the uiAccess flag; false when Windows will not say, so a check that must fail closed
/// uses [`check`].
#[must_use]
pub fn active() -> bool {
    check().unwrap_or(false)
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_test_run_has_no_ui_access() {
        assert!(matches!(super::check(), Ok(false)));
        assert!(!super::active());
    }
}
