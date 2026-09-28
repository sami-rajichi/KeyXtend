//! Keeps our windows on top and never focused, like osk.exe.

use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::Threading::GetCurrentProcessId;
use windows::Win32::UI::Shell::{DefSubclassProc, SetWindowSubclass};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GWL_EXSTYLE, GetForegroundWindow, GetWindowLongPtrW, GetWindowTextW,
    GetWindowThreadProcessId, HWND_TOPMOST, IsWindowVisible, LWA_ALPHA, MA_NOACTIVATE, STYLESTRUCT,
    SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SetForegroundWindow, SetLayeredWindowAttributes,
    SetWindowLongPtrW, SetWindowPos, WINDOWPOS, WM_MOUSEACTIVATE, WM_STYLECHANGING,
    WM_WINDOWPOSCHANGING, WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST,
    WS_EX_TRANSPARENT,
};
use windows::core::BOOL;

/// Our subclass id on each guarded window.
const GUARD_ID: usize = 0x4B58;
/// Alpha of a fully opaque layered window.
const OPAQUE: u8 = u8::MAX;

/// Extended styles every keyboard window keeps.
fn wanted() -> u32 {
    (WS_EX_NOACTIVATE | WS_EX_TOPMOST | WS_EX_TOOLWINDOW).0
}

/// Extended styles that let clicks pass through a window to the app below.
fn through() -> u32 {
    (WS_EX_LAYERED | WS_EX_TRANSPARENT).0
}

/// The extended style a guarded window may take: ours always, and click-through once it was set.
fn kept_ex(old: u32, new: u32) -> u32 {
    let keep = old & through() == through();
    new | wanted() | if keep { through() } else { 0 }
}

/// Lets clicks pass through our visible window `title_is`; false when it is not visible yet.
pub fn click_through(title_is: &str) -> Result<bool, String> {
    let Some(hwnd) = own_titled(title_is) else {
        return Ok(false);
    };
    // SAFETY: `hwnd` is one of our own live windows; a layered window shows only once its alpha is set.
    unsafe {
        let ex = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
        SetWindowLongPtrW(hwnd, GWL_EXSTYLE, ex | through() as isize);
        SetLayeredWindowAttributes(hwnd, COLORREF(0), OPAQUE, LWA_ALPHA)
            .map_err(|e| e.to_string())?;
    }
    Ok(true)
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
        // A plain show would take focus from the app in front; every move or show stays passive.
        WM_WINDOWPOSCHANGING => {
            // SAFETY: for WM_WINDOWPOSCHANGING, lparam points to a live WINDOWPOS.
            unsafe { (*(lparam.0 as *mut WINDOWPOS)).flags |= SWP_NOACTIVATE };
        }
        WM_STYLECHANGING if wparam.0 as i32 == GWL_EXSTYLE.0 => {
            // SAFETY: for WM_STYLECHANGING, lparam points to a live STYLESTRUCT.
            let s = unsafe { &mut *(lparam.0 as *mut STYLESTRUCT) };
            s.styleNew = kept_ex(s.styleOld, s.styleNew);
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

/// Guards every visible window of this process, keeping their order; returns how many.
pub fn guard_own_windows() -> Result<usize, String> {
    guard_all(guard_order(own_windows(), |_| false))
}

/// Guards every visible window of this process and puts the one called `top` above the others.
/// False when `top` is not visible yet: a toolkit may show a window a moment after it was asked to.
pub fn guard_with_top(top: &str) -> Result<bool, String> {
    let windows = own_windows();
    let found = windows.iter().any(|&w| title(w) == top);
    guard_all(guard_order(windows, |&w| title(w) == top))?;
    Ok(found)
}

/// Guards `windows` in turn; returns how many.
fn guard_all(windows: Vec<HWND>) -> Result<usize, String> {
    for hwnd in &windows {
        guard(*hwnd)?;
    }
    Ok(windows.len())
}

/// Windows listed front first, in guard order: each guard lifts its window to the top, so the front one comes last.
/// A window shown without activation may sit below the others; `is_top` marks it to come last of all.
fn guard_order<T>(mut front_first: Vec<T>, is_top: impl Fn(&T) -> bool) -> Vec<T> {
    front_first.reverse();
    let (mut rest, tops): (Vec<T>, Vec<T>) = front_first.into_iter().partition(|w| !is_top(w));
    rest.extend(tops);
    rest
}

/// The window in front right now.
pub fn foreground() -> HWND {
    // SAFETY: plain query.
    unsafe { GetForegroundWindow() }
}

/// Longest window title we read.
const TITLE_CAP: usize = 512;

/// The title of `hwnd`.
fn title(hwnd: HWND) -> String {
    let mut buf = [0u16; TITLE_CAP];
    // SAFETY: plain query into a local buffer.
    let n = unsafe { GetWindowTextW(hwnd, &mut buf) }.max(0) as usize;
    String::from_utf16_lossy(&buf[..n.min(TITLE_CAP)])
}

/// A window handle from its raw value, as passed between threads.
pub fn from_raw(v: isize) -> HWND {
    HWND(v as *mut _)
}

/// Our visible window called `title`.
pub fn own_titled(title_is: &str) -> Option<HWND> {
    own_windows().into_iter().find(|&w| title(w) == title_is)
}

/// Brings `target` back to the front if something else took it; true when it is in front after `settle_ms`.
pub fn bring_back(target: HWND, settle_ms: u64) -> bool {
    if foreground() != target {
        // SAFETY: plain call; a refused or dead window just stays behind, which the check below sees.
        let _ = unsafe { SetForegroundWindow(target) };
    }
    std::thread::sleep(std::time::Duration::from_millis(settle_ms));
    foreground() == target
}

/// Hands the foreground back to `previous` if one of our windows took it at start-up.
pub fn give_back(previous: HWND) {
    if !previous.is_invalid() && is_ours(foreground()) && !is_ours(previous) {
        // SAFETY: plain call; we are in front, so Windows allows it.
        let _ = unsafe { SetForegroundWindow(previous) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guarding_goes_back_to_front_so_the_front_window_stays_in_front() {
        assert_eq!(guard_order(vec![3, 2, 1], |_| false), vec![1, 2, 3]);
    }

    #[test]
    fn a_click_through_window_stays_click_through() {
        let through = through();
        assert_eq!(kept_ex(through, 0), wanted() | through);
        assert_eq!(kept_ex(0, 0), wanted(), "other windows keep taking clicks");
        assert_eq!(
            kept_ex(WS_EX_LAYERED.0, 0),
            wanted(),
            "layered alone is not click-through"
        );
        assert_eq!(
            kept_ex(WS_EX_TRANSPARENT.0, 0),
            wanted(),
            "transparent alone never makes a window layered"
        );
    }

    #[test]
    fn the_window_just_shown_is_guarded_last_so_it_ends_on_top() {
        assert_eq!(guard_order(vec![3, 2, 1], |&w| w == 2), vec![1, 3, 2]);
        assert_eq!(guard_order(vec![3, 2, 1], |&w| w == 1), vec![2, 3, 1]);
    }
}
