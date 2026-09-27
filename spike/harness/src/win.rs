//! Window helpers: list, find, describe, bring to front and close; plus polling until something happens.

use std::collections::HashSet;
use std::time::{Duration, Instant};

use windows::Win32::Foundation::{CloseHandle, HWND, LPARAM, WPARAM};
use windows::Win32::System::Threading::{
    OpenProcess, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION, QueryFullProcessImageNameW,
};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GUITHREADINFO, GetClassNameW, GetGUIThreadInfo, GetSystemMetrics, GetWindowTextW,
    GetWindowThreadProcessId, IsChild, IsIconic, IsWindow, IsWindowVisible, PostMessageW,
    SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN, SW_RESTORE,
    SetForegroundWindow, ShowWindow, WM_CLOSE,
};
use windows::core::{BOOL, PWSTR};

use crate::config::{Keys, Timing};

/// Longest window title or class we read.
const TEXT_CAP: usize = 512;
/// Longest program path we read: the Windows long-path limit.
const PATH_CAP: usize = 32_768;

/// Sleeps `ms` milliseconds.
pub fn sleep_ms(ms: u64) {
    std::thread::sleep(Duration::from_millis(ms));
}

/// Calls `f` every `poll_ms` until it gives a value or `timeout_ms` has passed.
pub fn poll_until<T>(timeout_ms: u64, poll_ms: u64, mut f: impl FnMut() -> Option<T>) -> Option<T> {
    let end = Instant::now() + Duration::from_millis(timeout_ms);
    loop {
        if let Some(v) = f() {
            return Some(v);
        }
        if Instant::now() >= end {
            return None;
        }
        sleep_ms(poll_ms);
    }
}

/// A window handle as a plain number, for sets.
pub fn key(hwnd: HWND) -> isize {
    hwnd.0 as isize
}

/// Every visible top-level window, topmost first.
pub fn top_windows() -> Vec<HWND> {
    unsafe extern "system" fn collect(hwnd: HWND, found: LPARAM) -> BOOL {
        // SAFETY: `found` is the `Vec` passed below, alive for the whole call.
        unsafe {
            if IsWindowVisible(hwnd).as_bool() {
                (*(found.0 as *mut Vec<HWND>)).push(hwnd);
            }
        }
        BOOL(1)
    }
    let mut found: Vec<HWND> = Vec::new();
    // SAFETY: the callback only touches `found`, which outlives the call.
    let _ = unsafe { EnumWindows(Some(collect), LPARAM(&mut found as *mut _ as isize)) };
    found
}

/// The windows that exist now, to spot new ones later.
pub fn snapshot() -> HashSet<isize> {
    top_windows().into_iter().map(key).collect()
}

/// The window title.
pub fn title(hwnd: HWND) -> String {
    let mut buf = [0u16; TEXT_CAP];
    // SAFETY: the buffer is valid for its whole length.
    let n = unsafe { GetWindowTextW(hwnd, &mut buf) };
    String::from_utf16_lossy(&buf[..usize::try_from(n).unwrap_or(0)])
}

/// The window class name.
pub fn class(hwnd: HWND) -> String {
    let mut buf = [0u16; TEXT_CAP];
    // SAFETY: the buffer is valid for its whole length.
    let n = unsafe { GetClassNameW(hwnd, &mut buf) };
    String::from_utf16_lossy(&buf[..usize::try_from(n).unwrap_or(0)])
}

/// `"title" [class]`, for reports.
pub fn describe(hwnd: HWND) -> String {
    if hwnd.is_invalid() {
        return "(none)".to_string();
    }
    format!("{:?} [{}]", title(hwnd), class(hwnd))
}

/// The process and thread that own `hwnd`.
pub fn owner(hwnd: HWND) -> (u32, u32) {
    let mut pid = 0;
    // SAFETY: plain query into a local.
    let thread = unsafe { GetWindowThreadProcessId(hwnd, Some(&mut pid)) };
    (pid, thread)
}

/// Full path of the program that owns `hwnd`.
pub fn program(hwnd: HWND) -> Result<String, String> {
    let pid = owner(hwnd).0;
    let mut buf = vec![0u16; PATH_CAP];
    let mut len = PATH_CAP as u32;
    // SAFETY: the process handle is ours and closed before returning; `len` is the buffer length.
    unsafe {
        let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid)
            .map_err(|e| format!("OpenProcess {pid}: {e}"))?;
        let read = QueryFullProcessImageNameW(
            process,
            PROCESS_NAME_WIN32,
            PWSTR(buf.as_mut_ptr()),
            &mut len,
        );
        let _ = CloseHandle(process);
        read.map_err(|e| format!("QueryFullProcessImageNameW: {e}"))?;
    }
    buf.truncate((len as usize).min(PATH_CAP));
    Ok(String::from_utf16_lossy(&buf))
}

fn thread_info(thread: u32) -> GUITHREADINFO {
    let mut info = GUITHREADINFO {
        cbSize: size_of::<GUITHREADINFO>() as u32,
        ..Default::default()
    };
    // SAFETY: `info` is a local with `cbSize` set, as the call requires; failure leaves it zeroed.
    let _ = unsafe { GetGUIThreadInfo(thread, &mut info) };
    info
}

/// The caret window of `thread`, or a null handle.
pub fn caret(thread: u32) -> HWND {
    thread_info(thread).hwndCaret
}

/// The window with keyboard focus in the thread that owns `hwnd`, or a null handle.
pub fn focus(hwnd: HWND) -> HWND {
    thread_info(owner(hwnd).1).hwndFocus
}

/// True if `w` is `parent` itself or one of its child windows.
pub fn contains(parent: HWND, w: HWND) -> bool {
    // SAFETY: plain query; any handle values are allowed.
    w == parent || unsafe { IsChild(parent, w) }.as_bool()
}

/// The other visible top-level windows of `hwnd`'s process, such as notices and dialogs.
pub fn popups(hwnd: HWND) -> Vec<String> {
    let pid = owner(hwnd).0;
    top_windows()
        .into_iter()
        .filter(|&w| w != hwnd && owner(w).0 == pid)
        .map(describe)
        .collect()
}

/// The virtual desktop: left, top, width and height in physical pixels.
pub fn desktop() -> (i32, i32, i32, i32) {
    // SAFETY: plain metric queries.
    unsafe {
        (
            GetSystemMetrics(SM_XVIRTUALSCREEN),
            GetSystemMetrics(SM_YVIRTUALSCREEN),
            GetSystemMetrics(SM_CXVIRTUALSCREEN),
            GetSystemMetrics(SM_CYVIRTUALSCREEN),
        )
    }
}

/// What a wanted window looks like.
#[derive(Debug, Clone, Default)]
pub struct Match {
    /// Text the title must contain.
    pub title_has: Option<String>,
    /// Exact class name.
    pub class: Option<String>,
    /// Windows to ignore because they existed before.
    pub skip: HashSet<isize>,
}

/// The topmost visible top-level window that fits `m`.
pub fn find(m: &Match) -> Option<HWND> {
    top_windows().into_iter().find(|&w| {
        !m.skip.contains(&key(w))
            && m.class.as_ref().is_none_or(|c| class(w) == *c)
            && m.title_has
                .as_ref()
                .is_none_or(|t| title(w).contains(t.as_str()))
    })
}

/// Waits up to `timeout_ms` for a window that fits `m`.
pub fn wait_for(m: &Match, timeout_ms: u64, poll_ms: u64) -> Option<HWND> {
    poll_until(timeout_ms, poll_ms, || find(m))
}

/// True while `hwnd` is a live window.
pub fn exists(hwnd: HWND) -> bool {
    // SAFETY: plain query; any handle value is allowed.
    unsafe { IsWindow(Some(hwnd)).as_bool() }
}

/// Waits up to `timeout_ms` for `hwnd` to go away; true if it did.
pub fn wait_gone(hwnd: HWND, timeout_ms: u64, poll_ms: u64) -> bool {
    poll_until(timeout_ms, poll_ms, || (!exists(hwnd)).then_some(())).is_some()
}

/// Waits up to `timeout_ms` until no window fits `m`; returns one that still does.
pub fn wait_no_match(m: &Match, timeout_ms: u64, poll_ms: u64) -> Option<HWND> {
    match poll_until(timeout_ms, poll_ms, || find(m).is_none().then_some(())) {
        Some(()) => None,
        None => find(m),
    }
}

/// Asks `hwnd` to close.
pub fn close(hwnd: HWND) -> Result<(), String> {
    // SAFETY: posting a message to any window handle is allowed; bad handles give an error.
    unsafe { PostMessageW(Some(hwnd), WM_CLOSE, WPARAM(0), LPARAM(0)) }
        .map_err(|e| format!("WM_CLOSE to {}: {e}", describe(hwnd)))
}

/// Brings `hwnd` to the front with the Alt-tap trick; true once it is in front.
pub fn front(hwnd: HWND, t: &Timing, keys: &Keys) -> bool {
    for _ in 0..t.front_tries.max(1) {
        if spike_core::window::foreground() == hwnd {
            return true;
        }
        // SAFETY: plain calls on a window handle; failures are checked below.
        unsafe {
            if IsIconic(hwnd).as_bool() {
                let _ = ShowWindow(hwnd, SW_RESTORE);
            }
        }
        let _ = crate::keys::combo(&keys.alt);
        sleep_ms(t.alt_settle_ms);
        // SAFETY: plain call; the result is checked through the foreground below.
        let _ = unsafe { SetForegroundWindow(hwnd) };
        sleep_ms(t.front_wait_ms);
    }
    spike_core::window::foreground() == hwnd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn poll_until_returns_the_first_value_at_once() {
        let mut calls = 0;
        let got = poll_until(0, 0, || {
            calls += 1;
            Some(7)
        });
        assert_eq!((got, calls), (Some(7), 1));
    }

    #[test]
    fn poll_until_gives_up_after_the_timeout_without_waiting() {
        let mut calls = 0;
        let got = poll_until::<()>(0, u64::MAX, || {
            calls += 1;
            None
        });
        assert_eq!((got, calls), (None, 1));
    }
}
