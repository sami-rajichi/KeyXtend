//! Keeps our windows on top and never focused, like osk.exe.

use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    CombineRgn, CreateRectRgn, DeleteObject, HRGN, RGN_ERROR, RGN_OR, SetWindowRgn,
};
use windows::Win32::System::Console::GetConsoleWindow;
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

/// Cuts our visible window `title_is` down to `rects` (left, top, right, bottom in physical pixels), so a click
/// anywhere else reaches the app below; false when it is not visible yet.
pub fn shape(title_is: &str, rects: &[[i32; 4]]) -> Result<bool, String> {
    let Some(hwnd) = own_titled(title_is) else {
        return Ok(false);
    };
    let all = joined(rects)?;
    // SAFETY: `all` is ours until `SetWindowRgn` takes it; `hwnd` is one of our live windows.
    unsafe {
        if SetWindowRgn(hwnd, Some(all), true) == 0 {
            let _ = DeleteObject(all.into());
            return Err("Windows refused the window shape".into());
        }
    }
    Ok(true)
}

/// One region covering every rect; the caller owns it.
fn joined(rects: &[[i32; 4]]) -> Result<HRGN, String> {
    const FAILED: &str = "Windows could not build the window shape";
    // SAFETY: every region made here is deleted here, except `all`, which is returned to the caller.
    unsafe {
        let all = CreateRectRgn(0, 0, 0, 0);
        if all.is_invalid() {
            return Err(FAILED.into());
        }
        for &[l, t, r, b] in rects {
            let one = CreateRectRgn(l, t, r, b);
            let ok = !one.is_invalid()
                && CombineRgn(Some(all), Some(all), Some(one), RGN_OR) != RGN_ERROR;
            let _ = DeleteObject(one.into());
            if !ok {
                let _ = DeleteObject(all.into());
                return Err(FAILED.into());
            }
        }
        Ok(all)
    }
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
    let (me, console) = unsafe {
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        (GetCurrentProcessId(), GetConsoleWindow())
    };
    ours_by(hwnd, pid, me, console)
}

/// True if `hwnd`, which reports process `pid`, is ours (`me`); a console build's `console` window reports
/// our process but belongs to the console host.
fn ours_by(hwnd: HWND, pid: u32, me: u32, console: HWND) -> bool {
    pid == me && hwnd != console
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
    fn the_console_window_windows_attaches_is_not_ours_to_guard() {
        let (kb, console) = (HWND(1 as _), HWND(2 as _));
        assert!(ours_by(kb, 7, 7, console));
        assert!(
            !ours_by(console, 7, 7, console),
            "the console host's window"
        );
        assert!(!ours_by(kb, 8, 7, console), "another process");
        assert!(
            ours_by(kb, 7, 7, HWND::default()),
            "a build with no console"
        );
    }

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
