//! The icon window's messages: queueing one to itself, the taskbar's restart message, and letting the taskbar reach it.

use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    ChangeWindowMessageFilterEx, MSGFLT_ALLOW, PostMessageW, RegisterWindowMessageW,
};
use windows::core::w;

/// Queues `msg` to our own window.
#[allow(unsafe_code, reason = "Posts a message to our own window.")]
pub(super) fn post(hwnd: HWND, msg: u32) {
    // SAFETY: a plain message with no pointers, to a window this thread made.
    let _ = unsafe { PostMessageW(Some(hwnd), msg, WPARAM(0), LPARAM(0)) };
}

/// The taskbar's start message, which Windows names once per session.
#[allow(unsafe_code, reason = "Registers a message name.")]
pub(super) fn restart_msg() -> u32 {
    // SAFETY: a constant null-terminated name.
    unsafe { RegisterWindowMessageW(w!("TaskbarCreated")) }
}

/// Lets `msg` reach `hwnd` from the taskbar, which runs below our integrity level when we have uiAccess.
#[allow(unsafe_code, reason = "Changes our own window's message filter.")]
pub(super) fn allow(hwnd: HWND, msg: u32) {
    if msg != 0 {
        // SAFETY: changes only our own window's filter, for one message.
        let _ = unsafe { ChangeWindowMessageFilterEx(hwnd, msg, MSGFLT_ALLOW, None) };
    }
}
