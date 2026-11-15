//! The watch on a real hidden window that is never shown or activated: each lock, unlock, sleep and wake message
//! posted to it is told once, as the PC is never locked or put to sleep in a test, and nothing after the run drops.

use super::*;
use std::sync::mpsc;
use std::time::Duration;
use windows::Win32::UI::WindowsAndMessaging::{
    FindWindowExW, GWL_EXSTYLE, GetWindowLongPtrW, GetWindowThreadProcessId, IsWindowVisible,
    PostMessageW, WS_EX_NOACTIVATE,
};
use windows::core::PCWSTR;

/// How long a test waits for a note; a wait, never a sleep.
const WAIT: Duration = Duration::from_secs(10);
/// `WM_WTSSESSION_CHANGE` and `WM_POWERBROADCAST` with their wParams, as the table lists them.
const LOCK: (u32, usize) = (0x2B1, 7);
const UNLOCK: (u32, usize) = (0x2B1, 8);
const SLEEP: (u32, usize) = (0x218, 4);
const WAKE_AUTO: (u32, usize) = (0x218, 18);
const WAKE_USER: (u32, usize) = (0x218, 7);

/// Our process's watch window, waiting until it is made.
#[allow(
    unsafe_code,
    reason = "Finds a window of our own process by its class."
)]
fn ours() -> HWND {
    let class = wide(CLASS);
    let mine = std::process::id();
    for _ in 0..u16::MAX {
        let mut after = None;
        let next = |after| {
            // SAFETY: `class` is NUL-ended; each call starts after the window before, and ends with none.
            unsafe { FindWindowExW(None, after, PCWSTR(class.as_ptr()), PCWSTR::null()) }
        };
        while let Ok(hwnd) = next(after) {
            let mut pid = 0;
            // SAFETY: a window Windows just gave, and a local it writes.
            unsafe { GetWindowThreadProcessId(hwnd, Some(&raw mut pid)) };
            if pid == mine {
                return hwnd;
            }
            after = Some(hwnd);
        }
        std::thread::yield_now();
    }
    panic!("no watch window");
}

/// True when `hwnd` is not visible and takes no activation, as every window we own but Settings.
#[allow(unsafe_code, reason = "Reads our own window's state.")]
fn hidden_and_never_active(hwnd: HWND) -> bool {
    // SAFETY: our own live window; both are plain reads.
    let (shown, ex) = unsafe { (IsWindowVisible(hwnd), GetWindowLongPtrW(hwnd, GWL_EXSTYLE)) };
    let ex = u32::try_from(ex).unwrap_or_default();
    !shown.as_bool() && ex & WS_EX_NOACTIVATE.0 != 0
}

/// Posts `msg` with `wparam` to `hwnd`, as Windows would send it.
#[allow(unsafe_code, reason = "Posts a message to our own window.")]
fn post(hwnd: HWND, (msg, wparam): (u32, usize)) {
    // SAFETY: our own live window; the message carries no pointer.
    unsafe { PostMessageW(Some(hwnd), msg, WPARAM(wparam), LPARAM(0)) }.unwrap();
}

#[test]
fn each_change_posted_to_the_window_is_told_once_and_none_after_the_drop() {
    let (said, heard) = mpsc::channel();
    let tell = move |n| {
        let _ = said.send(n);
    };
    let run = WinSessionWatch.watch(Box::new(tell)).unwrap();
    let hwnd = ours();
    assert!(
        hidden_and_never_active(hwnd),
        "never shown, never activated"
    );
    for change in [LOCK, UNLOCK, SLEEP, WAKE_AUTO, WAKE_USER, LOCK] {
        post(hwnd, change);
    }
    let told: Vec<SessionNote> = (0..5).map(|_| heard.recv_timeout(WAIT).unwrap()).collect();
    let want = [
        SessionNote::Locked,
        SessionNote::Unlocked,
        SessionNote::Sleeping,
        SessionNote::Woke,
        SessionNote::Locked,
    ];
    assert_eq!(told, want, "the second wake is told once");
    drop(run);
    assert!(
        heard.recv_timeout(WAIT).is_err(),
        "its callback went with it"
    );
}
