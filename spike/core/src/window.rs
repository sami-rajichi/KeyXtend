//! Keeps our windows on top and never focused, like osk.exe.

use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::Threading::GetCurrentProcessId;
use windows::Win32::UI::Shell::{DefSubclassProc, SetWindowSubclass};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GWL_EXSTYLE, GetForegroundWindow, GetWindowLongPtrW, GetWindowThreadProcessId,
    HWND_TOPMOST, IsWindowVisible, MA_NOACTIVATE, STYLESTRUCT, SWP_NOACTIVATE, SWP_NOMOVE,
    SWP_NOSIZE, SetForegroundWindow, SetWindowLongPtrW, SetWindowPos, WM_MOUSEACTIVATE,
    WM_STYLECHANGING, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST,
};
use windows::core::BOOL;

/// Our subclass id on each guarded window.
const GUARD_ID: usize = 0x4B58;

/// Extended styles every keyboard window keeps.
fn wanted() -> u32 {
    (WS_EX_NOACTIVATE | WS_EX_TOPMOST | WS_EX_TOOLWINDOW).0
}

/// Makes `hwnd` non-activating and topmost, and keeps it that way.
pub fn guard(hwnd: HWND) -> Result<(), String> {
    // SAFETY: `hwnd` is one of our own live windows; the subclass proc is `'static`.
    unsafe {
        let ex = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
        SetWindowLongPtrW(hwnd, GWL_EXSTYLE, ex | wanted() as isize);
        let flags = SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE;
        SetWindowPos(hwnd, Some(HWND_TOPMOST), 0, 0, 0, 0, flags).map_err(|e| e.to_string())?;
        if !SetWindowSubclass(hwnd, Some(guard_proc), GUARD_ID, 0).as_bool() {
            return Err("SetWindowSubclass failed".to_string());
        }
    }
    Ok(())
}

unsafe extern "system" fn guard_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    _id: usize,
    _data: usize,
) -> LRESULT {
    match msg {
        WM_MOUSEACTIVATE => return LRESULT(MA_NOACTIVATE as isize),
        WM_STYLECHANGING if wparam.0 as i32 == GWL_EXSTYLE.0 => {
            // SAFETY: for WM_STYLECHANGING, lparam points to a live STYLESTRUCT.
            unsafe { (*(lparam.0 as *mut STYLESTRUCT)).styleNew |= wanted() };
        }
        _ => {}
    }
    // SAFETY: forwards the same message to the next handler.
    unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) }
}

/// True if `hwnd` belongs to this process.
pub fn is_ours(hwnd: HWND) -> bool {
    let mut pid = 0;
    // SAFETY: plain queries.
    unsafe {
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        pid == GetCurrentProcessId()
    }
}

/// The visible top-level windows of this process.
pub fn own_windows() -> Vec<HWND> {
    unsafe extern "system" fn collect(hwnd: HWND, found: LPARAM) -> BOOL {
        // SAFETY: `found` is the `Vec` passed below, alive for the whole call.
        unsafe {
            if IsWindowVisible(hwnd).as_bool() && is_ours(hwnd) {
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

/// Guards every visible window of this process; returns how many.
pub fn guard_own_windows() -> Result<usize, String> {
    let windows = own_windows();
    for hwnd in &windows {
        guard(*hwnd)?;
    }
    Ok(windows.len())
}

/// The window in front right now.
pub fn foreground() -> HWND {
    // SAFETY: plain query.
    unsafe { GetForegroundWindow() }
}

/// Hands the foreground back to `previous` if one of our windows took it at start-up.
pub fn give_back(previous: HWND) {
    if !previous.is_invalid() && is_ours(foreground()) && !is_ours(previous) {
        // SAFETY: plain call; we are in front, so Windows allows it.
        let _ = unsafe { SetForegroundWindow(previous) };
    }
}
