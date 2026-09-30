//! The guard on real, hidden windows of the test thread; no test shows a window or changes the focus.
#![cfg(windows)]
#![allow(
    unsafe_code,
    reason = "The tests make, guard and destroy their own hidden windows."
)]

use std::cell::Cell;

use kx_platform::ui::{FocusToken, UiError, WindowGuard};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, GWL_EXSTYLE, GetDesktopWindow,
    GetWindowLongPtrW, MA_NOACTIVATE, RegisterClassW, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
    SWP_NOZORDER, SendMessageW, SetWindowLongPtrW, WINDOW_EX_STYLE, WINDOWPOS, WM_MOUSEACTIVATE,
    WM_NCDESTROY, WM_WINDOWPOSCHANGING, WNDCLASSW, WS_POPUP,
};
use windows::core::{PCWSTR, w};

use super::guard::{guard, guarded};
use super::win::{token_of, window_of};
use super::{
    KEPT, THROUGH, WinGuard, foreground, from_raw, give_back, is_ours, own_windows, title,
};
use crate::wide;

/// Class of the test windows.
const CLASS: PCWSTR = w!("KeyXtend.GuardTest");

thread_local! {
    /// Whether the guard was still hooked when this thread's last test window got `WM_NCDESTROY`.
    static HOOKED_AT_END: Cell<Option<bool>> = const { Cell::new(None) };
}

/// The test windows' own procedure; it runs after the guard's, so it sees what the guard left.
unsafe extern "system" fn plain(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if msg == WM_NCDESTROY {
        HOOKED_AT_END.set(Some(guarded(hwnd)));
    }
    // SAFETY: default handling of a message for the test's own window.
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

/// A hidden window of the test thread, destroyed when dropped; it is never shown.
struct Hidden(HWND);

impl Hidden {
    fn new(title_is: &str) -> Self {
        let name = wide(title_is);
        // SAFETY: registers the test class, made again harmlessly, and a window of it that is never shown.
        let hwnd = unsafe {
            let module = GetModuleHandleW(None).unwrap();
            let wc = WNDCLASSW {
                lpfnWndProc: Some(plain),
                hInstance: module.into(),
                lpszClassName: CLASS,
                ..WNDCLASSW::default()
            };
            RegisterClassW(&raw const wc);
            let (ex, style) = (WINDOW_EX_STYLE::default(), WS_POPUP);
            let name = PCWSTR(name.as_ptr());
            CreateWindowExW(
                ex,
                CLASS,
                name,
                style,
                0,
                0,
                1,
                1,
                None,
                None,
                Some(module.into()),
                None,
            )
        };
        Self(hwnd.unwrap())
    }

    fn ex_style(&self) -> u32 {
        // SAFETY: a plain query of the test's own window.
        u32::try_from(unsafe { GetWindowLongPtrW(self.0, GWL_EXSTYLE) }).unwrap()
    }
}

impl Drop for Hidden {
    fn drop(&mut self) {
        // SAFETY: destroys the test's own window on its own thread, once.
        let _ = unsafe { DestroyWindow(self.0) };
    }
}

#[test]
fn the_style_table_matches_windows() {
    use windows::Win32::UI::WindowsAndMessaging::{
        WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_EX_TRANSPARENT,
    };
    assert_eq!(
        KEPT,
        (WS_EX_NOACTIVATE | WS_EX_TOPMOST | WS_EX_TOOLWINDOW).0
    );
    assert_eq!(THROUGH, (WS_EX_LAYERED | WS_EX_TRANSPARENT).0);
}

#[test]
fn a_guarded_window_keeps_its_styles_and_never_activates() {
    let window = Hidden::new("KeyXtend test: guarded");
    guard(window.0).unwrap();
    assert_eq!(window.ex_style() & KEPT, KEPT);
    // SAFETY: tries to clear the test window's extended style.
    unsafe { SetWindowLongPtrW(window.0, GWL_EXSTYLE, 0) };
    assert_eq!(window.ex_style() & KEPT, KEPT, "the guard put them back");
    // SAFETY: asks the test window how a click would activate it.
    let answer = unsafe { SendMessageW(window.0, WM_MOUSEACTIVATE, None, None) };
    assert_eq!(answer.0, isize::try_from(MA_NOACTIVATE).unwrap());
}

#[test]
fn every_move_or_show_stays_passive() {
    let window = Hidden::new("KeyXtend test: passive");
    guard(window.0).unwrap();
    let mut pos = WINDOWPOS {
        hwnd: window.0,
        flags: SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER,
        ..WINDOWPOS::default()
    };
    let lparam = LPARAM(
        std::ptr::from_mut(&mut pos)
            .expose_provenance()
            .cast_signed(),
    );
    // SAFETY: `pos` outlives the call, as WM_WINDOWPOSCHANGING needs.
    unsafe { SendMessageW(window.0, WM_WINDOWPOSCHANGING, None, Some(lparam)) };
    assert!(pos.flags.contains(SWP_NOACTIVATE));
}

#[test]
fn guarding_again_only_renews_the_guard() {
    let window = Hidden::new("KeyXtend test: again");
    guard(window.0).unwrap();
    guard(window.0).unwrap();
    assert!(guarded(window.0));
    // SAFETY: the window still answers through one guard and its own procedure.
    let answer = unsafe { SendMessageW(window.0, WM_MOUSEACTIVATE, None, None) };
    assert_eq!(answer.0, isize::try_from(MA_NOACTIVATE).unwrap());
}

#[test]
fn the_guard_unhooks_itself_when_its_window_is_destroyed() {
    let window = Hidden::new("KeyXtend test: unhook");
    guard(window.0).unwrap();
    assert!(guarded(window.0));
    drop(window);
    assert_eq!(HOOKED_AT_END.get(), Some(false), "unhooked on WM_NCDESTROY");
}

#[test]
fn an_unguarded_window_ends_with_no_guard() {
    drop(Hidden::new("KeyXtend test: plain"));
    assert_eq!(HOOKED_AT_END.get(), Some(false));
}

#[test]
fn a_window_of_another_thread_is_refused() {
    let window = Hidden::new("KeyXtend test: other thread");
    let raw = window.0.0.addr().cast_signed();
    let tried = std::thread::spawn(move || guard(from_raw(raw)))
        .join()
        .unwrap();
    assert!(matches!(tried, Err(UiError::Refused(_))), "{tried:?}");
    assert!(!guarded(window.0));
}

#[test]
fn hidden_windows_are_ours_but_never_guarded_or_shaped() {
    let window = Hidden::new("KeyXtend test: hidden");
    assert!(is_ours(window.0));
    assert_eq!(title(window.0), "KeyXtend test: hidden");
    assert!(
        !own_windows().contains(&window.0),
        "only visible windows count"
    );
    assert_eq!(
        WinGuard.guard_own(None),
        Ok(0),
        "the test program shows no window"
    );
    let rect = kx_module_api::ports::Rect::default();
    let shaped = WinGuard.shape("KeyXtend test: hidden", &[rect]);
    assert_eq!(shaped, Err(UiError::NoWindow));
}

#[test]
fn the_desktop_and_no_window_are_not_ours() {
    // SAFETY: a plain query.
    assert!(!is_ours(unsafe { GetDesktopWindow() }));
    assert!(!is_ours(HWND::default()));
}

#[test]
fn a_raw_handle_and_a_focus_token_read_back() {
    let window = Hidden::new("KeyXtend test: raw");
    assert_eq!(from_raw(window.0.0.addr().cast_signed()), window.0);
    let token = FocusToken::new(u64::try_from(window.0.0.addr()).unwrap());
    assert_eq!(window_of(token), Some(window.0));
    assert_eq!(token_of(window.0), token);
}

#[test]
fn giving_back_is_a_no_op_while_none_of_ours_is_in_front() {
    let window = Hidden::new("KeyXtend test: give back");
    assert!(!is_ours(foreground()), "the test program is never in front");
    give_back(window.0);
    WinGuard.give_back(FocusToken::new(0));
}
