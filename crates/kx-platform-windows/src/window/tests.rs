//! The guard on real, hidden windows of the test thread; no test shows a window or changes the focus.
#![cfg(windows)]
#![allow(
    unsafe_code,
    reason = "The tests make, guard and destroy their own hidden windows."
)]

mod calls;
mod cross;
mod entries;
mod owners;
mod pages;
mod under;
mod z_order;

use std::cell::{Cell, RefCell};

use kx_platform::ui::{UiError, WindowGuard};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CallWindowProcW, CreateWindowExW, DefWindowProcW, DestroyWindow, GWL_EXSTYLE, GWLP_WNDPROC,
    GetDesktopWindow, GetWindowLongPtrW, MA_NOACTIVATE, RegisterClassW, SWP_NOACTIVATE, SWP_NOMOVE,
    SWP_NOSIZE, SWP_NOZORDER, SendMessageW, SetWindowLongPtrW, WINDOW_EX_STYLE, WINDOWPOS,
    WM_MOUSEACTIVATE, WM_NCDESTROY, WM_WINDOWPOSCHANGING, WNDCLASSW, WS_POPUP,
};
use windows::core::{PCWSTR, w};

use super::WS_EX_TRANSPARENT as TRANSPARENT;
use super::guard::{Proc, guard, guarded, next};
use super::{KEPT, THROUGH, WinGuard, from_raw, is_ours, own_windows, title};
use crate::wide;

/// Class of the test windows.
const CLASS: PCWSTR = w!("KeyXtend.GuardTest");
/// How long a test waits for another thread; a wait, never a sleep.
const WAIT: std::time::Duration = std::time::Duration::from_secs(5);

thread_local! {
    /// Whether the guard was still hooked when this thread's last test window got `WM_NCDESTROY`.
    static HOOKED_AT_END: Cell<Option<bool>> = const { Cell::new(None) };
    /// Which procedures saw `WM_NCDESTROY`, in order.
    static SEEN: RefCell<Vec<&'static str>> = const { RefCell::new(Vec::new()) };
    /// The procedure the later test subclass hands messages to.
    static BEHIND: Cell<isize> = const { Cell::new(0) };
}

/// The test windows' own procedure; it runs after the guard's, so it sees what the guard left.
unsafe extern "system" fn plain(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if msg == WM_NCDESTROY {
        HOOKED_AT_END.set(Some(guarded(hwnd)));
        SEEN.with_borrow_mut(|s| s.push("own"));
    }
    // SAFETY: default handling of a message for the test's own window.
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

/// A subclass another part of the program puts in front after the guard, as a toolkit could.
unsafe extern "system" fn later(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if msg == WM_NCDESTROY {
        SEEN.with_borrow_mut(|s| s.push("later"));
    }
    let behind = BEHIND.get().cast_unsigned();
    // SAFETY: hands the message to the procedure this subclass replaced, as a subclass must.
    unsafe { CallWindowProcW(next(behind), hwnd, msg, wparam, lparam) }
}

/// A hidden window of the test thread, destroyed when dropped; it is never shown.
struct Hidden(HWND);

impl Hidden {
    fn new(title_is: &str) -> Self {
        Self::make(title_is, WINDOW_EX_STYLE::default(), None)
    }

    /// Topmost from the start with the keyboard's styles, as Qt makes it; owned by `owner` when given.
    fn topmost(title_is: &str, owner: Option<HWND>) -> Self {
        Self::make(title_is, WINDOW_EX_STYLE(KEPT), owner)
    }

    /// Topmost from the start with a tip's styles, click-through too, as Qt makes it; owned by `owner` when given.
    fn tip(title_is: &str, owner: Option<HWND>) -> Self {
        Self::make(title_is, WINDOW_EX_STYLE(KEPT | THROUGH), owner)
    }

    fn make(title_is: &str, ex: WINDOW_EX_STYLE, owner: Option<HWND>) -> Self {
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
            let name = PCWSTR(name.as_ptr());
            CreateWindowExW(
                ex,
                CLASS,
                name,
                WS_POPUP,
                0,
                0,
                1,
                1,
                owner,
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

    /// Asks for extended style `ex`, as another program could.
    fn set_ex(&self, ex: u32) {
        // SAFETY: a style change on the test's own window.
        unsafe { SetWindowLongPtrW(self.0, GWL_EXSTYLE, isize::try_from(ex).unwrap()) };
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

/// Only a window that let clicks through when first guarded keeps doing so, so no program can make the keyboard
/// click-through.
#[test]
fn click_through_is_kept_only_from_the_first_look() {
    let solid = Hidden::new("KeyXtend test: solid");
    let tip = Hidden::tip("KeyXtend test: see-through", None);
    guard(solid.0).unwrap();
    guard(tip.0).unwrap();
    solid.set_ex(solid.ex_style() | THROUGH);
    assert_eq!(solid.ex_style() & TRANSPARENT, 0, "a later click-through");
    tip.set_ex(0);
    assert_eq!(
        tip.ex_style() & THROUGH,
        THROUGH,
        "a tip stays click-through"
    );
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
fn a_later_subclass_stays_in_front_and_every_procedure_sees_the_end() {
    let window = Hidden::new("KeyXtend test: later subclass");
    guard(window.0).unwrap();
    let ours = later as Proc as usize;
    // SAFETY: puts the test's subclass in front of the guard on the test's own window.
    BEHIND.set(unsafe { SetWindowLongPtrW(window.0, GWLP_WNDPROC, ours.cast_signed()) });
    guard(window.0).unwrap();
    // SAFETY: a plain query of the test's own window.
    let front = unsafe { GetWindowLongPtrW(window.0, GWLP_WNDPROC) };
    assert_eq!(
        front,
        ours.cast_signed(),
        "guarding again keeps it in front"
    );
    SEEN.take();
    drop(window);
    assert_eq!(SEEN.take(), ["later", "own"]);
    assert_eq!(
        HOOKED_AT_END.get(),
        Some(false),
        "the guard unhooked between them"
    );
}

#[test]
fn a_title_reads_from_another_thread_while_the_owner_is_busy() {
    let window = Hidden::new("KeyXtend test: busy owner");
    let raw = window.0.0.addr().cast_signed();
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || tx.send(title(from_raw(raw))));
    let read = rx.recv_timeout(WAIT);
    assert_eq!(read.as_deref(), Ok("KeyXtend test: busy owner"));
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
fn hidden_windows_are_ours_but_never_guarded_shaped_or_lifted() {
    let window = Hidden::new("KeyXtend test: hidden");
    assert!(is_ours(window.0));
    assert_eq!(title(window.0), "KeyXtend test: hidden");
    assert!(
        !own_windows().contains(&window.0),
        "only visible windows count"
    );
    assert_eq!(
        WinGuard.guard_own(None, None),
        Ok(0),
        "the test program shows no window"
    );
    let rect = kx_module_api::ports::Rect::default();
    let shaped = WinGuard.shape("KeyXtend test: hidden", &[rect]);
    assert_eq!(shaped, Err(UiError::NoWindow));
    let lifted = WinGuard.lift_alone("KeyXtend test: hidden");
    assert_eq!(lifted, Err(UiError::NoWindow));
}

#[test]
fn the_desktop_and_no_window_are_not_ours() {
    // SAFETY: a plain query.
    assert!(!is_ours(unsafe { GetDesktopWindow() }));
    assert!(!is_ours(HWND::default()));
}
