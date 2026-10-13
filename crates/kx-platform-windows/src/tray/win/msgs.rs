//! The icon window's messages: queueing one to it, the taskbar's restart message, and letting the taskbar reach it.

use kx_platform::ui::UiError;
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    ChangeWindowMessageFilterEx, MSGFLT_ALLOW, PostMessageW, RegisterWindowMessageW,
};
use windows::core::w;

use crate::refused;

/// Queues `msg` to the icon's window, from its own thread or the owner's; refused when Windows will not queue it.
#[allow(unsafe_code, reason = "Posts a message to our own window.")]
pub(super) fn post(hwnd: HWND, msg: u32) -> Result<(), UiError> {
    // SAFETY: a plain message with no pointers, to a window of ours that lives until the icon drops.
    unsafe { PostMessageW(Some(hwnd), msg, WPARAM(0), LPARAM(0)) }.map_err(|e| refused(&e))
}

/// The taskbar's start message, which Windows names once per session.
#[allow(unsafe_code, reason = "Registers a message name.")]
pub(super) fn restart_msg() -> u32 {
    // SAFETY: a constant null-terminated name.
    unsafe { RegisterWindowMessageW(w!("TaskbarCreated")) }
}

/// Lets `msg` reach `hwnd` from the taskbar, which runs below our integrity level when we have uiAccess; message 0
/// needs nothing.
#[allow(unsafe_code, reason = "Changes our own window's message filter.")]
pub(super) fn allow(hwnd: HWND, msg: u32) -> Result<(), UiError> {
    if msg == 0 {
        return Ok(());
    }
    // SAFETY: changes only our own window's filter, for one message.
    unsafe { ChangeWindowMessageFilterEx(hwnd, msg, MSGFLT_ALLOW, None) }.map_err(|e| refused(&e))
}
