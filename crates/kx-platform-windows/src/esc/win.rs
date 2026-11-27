//! The hook: a message-only window on its own thread (`kx-esc-watch`) installs a low-level keyboard hook as it is made
//! and removes it as it goes. The hook only counts a press and posts it to the window; the window's procedure then
//! calls back on this same thread, so every key waits while the callback runs. Nothing here logs.

use std::cell::{Cell, OnceCell};
use std::panic::{AssertUnwindSafe, catch_unwind};

use kx_platform::ui::{EscWatch, OnEsc, UiError, WatchGuard};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, HC_ACTION, HHOOK, KBDLLHOOKSTRUCT, PostMessageW, SetWindowsHookExW,
    UnhookWindowsHookEx, WH_KEYBOARD_LL, WM_APP, WM_CREATE, WM_DESTROY,
};

use super::pressed;
use crate::hidden::{Hidden, Kind, answer};
use crate::wide;

/// Class of the watch window, local to our process.
const CLASS: &str = "KeyXtend.EscWatch";
/// The watch thread's name.
const THREAD: &str = "kx-esc-watch";
/// What `WM_CREATE` answers so the window is not made: Windows refused the hook.
const NOT_MADE: LRESULT = LRESULT(-1);
/// The message the hook posts to the window for each press.
const PRESSED: u32 = WM_APP;
/// The hook `code` that carries an event.
const ACTION: i32 = HC_ACTION.cast_signed();

/// What a watch thread holds; the hook runs on this thread, called through its message loop.
struct Here {
    on: OnEsc,
    /// The window the hook posts to, once made.
    window: Cell<Option<HWND>>,
    /// The hook, while installed.
    hook: Cell<Option<HHOOK>>,
    /// Esc is down, so its repeats count once.
    held: Cell<bool>,
    /// Presses the hook counted and the window has not told, so a post from elsewhere tells nothing.
    pending: Cell<u32>,
}

thread_local! {
    /// Set once per watch thread before its window exists, and never borrowed mutably.
    static HERE: OnceCell<Here> = const { OnceCell::new() };
}

/// The Windows Esc watch: each watch is a thread with its own hook, which ends as the guard drops.
#[derive(Debug, Default)]
pub struct WinEscWatch;

impl WinEscWatch {
    /// A watch.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl EscWatch for WinEscWatch {
    fn watch(&self, on: OnEsc) -> Result<WatchGuard, UiError> {
        let setup = move || drop(HERE.with(|h| h.set(here(on))));
        let hidden = Hidden::named(THREAD, wide(CLASS), Kind::MessageOnly, Some(proc), setup)?;
        Ok(WatchGuard::new(move || hidden.stop()))
    }
}

/// A thread's state before its window is made.
fn here(on: OnEsc) -> Here {
    Here {
        on,
        window: Cell::new(None),
        hook: Cell::new(None),
        held: Cell::new(false),
        pending: Cell::new(0),
    }
}

/// Hooks from creation to destruction, and tells each press the hook posted.
#[allow(unsafe_code, reason = "The window procedure of our watch window.")]
unsafe extern "system" fn proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_CREATE if !hook(hwnd) => return NOT_MADE,
        WM_DESTROY => unhook(),
        PRESSED => {
            tell();
            return LRESULT::default();
        }
        _ => {}
    }
    // SAFETY: our own window's message, with its own values.
    unsafe { answer(hwnd, msg, wparam, lparam) }
}

/// Installs the hook for window `hwnd`'s thread; false when Windows refuses.
#[allow(unsafe_code, reason = "Installs our own hook procedure.")]
fn hook(hwnd: HWND) -> bool {
    HERE.with(|h| {
        let Some(here) = h.get() else {
            return false;
        };
        // SAFETY: a plain query of our own program module.
        let Ok(module) = (unsafe { GetModuleHandleW(None) }) else {
            return false;
        };
        // SAFETY: our own procedure, which lives as long as the program, in our own module.
        let hook =
            unsafe { SetWindowsHookExW(WH_KEYBOARD_LL, Some(key_proc), Some(module.into()), 0) };
        here.window.set(Some(hwnd));
        here.hook.set(hook.ok());
        here.hook.get().is_some()
    })
}

/// Removes the hook as the window goes.
#[allow(unsafe_code, reason = "Removes the hook this thread installed.")]
fn unhook() {
    HERE.with(|h| {
        if let Some(hook) = h.get().and_then(|here| here.hook.take()) {
            // SAFETY: the hook this thread installed, taken out of its slot so it is removed once.
            let _ = unsafe { UnhookWindowsHookEx(hook) };
        }
    });
}

#[allow(unsafe_code, reason = "Reads the struct Windows passes the hook.")]
unsafe extern "system" fn key_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code == ACTION {
        let ptr = std::ptr::with_exposed_provenance::<KBDLLHOOKSTRUCT>(lparam.0.cast_unsigned());
        // SAFETY: for HC_ACTION, `lparam` points to a KBDLLHOOKSTRUCT that lives for this call.
        if let Some(info) = unsafe { ptr.as_ref() } {
            let msg = u32::try_from(wparam.0).unwrap_or_default();
            if heard(msg, info.vkCode, info.dwExtraInfo) && post() {
                count();
            }
        }
    }
    // SAFETY: passes the event on unchanged; no key is ever dropped.
    unsafe { CallNextHookEx(None, code, wparam, lparam) }
}

/// Whether hook event `msg` for key `vk`, marked `extra`, is a new press of an Esc we did not send.
fn heard(msg: u32, vk: u32, extra: usize) -> bool {
    HERE.with(|h| {
        h.get().is_some_and(|here| {
            let mut held = here.held.get();
            let fresh = pressed(msg, vk, extra, &mut held);
            here.held.set(held);
            fresh
        })
    })
}

/// Counts a press posted to the window, which runs on this thread only after the hook returns.
fn count() {
    HERE.with(|h| {
        if let Some(here) = h.get() {
            here.pending.set(here.pending.get().saturating_add(1));
        }
    });
}

/// Posts a press to this thread's window; false when there is none or its queue is full, and the press is lost.
#[allow(unsafe_code, reason = "Posts to our own window.")]
fn post() -> bool {
    let Some(window) = HERE.with(|h| h.get().and_then(|here| here.window.get())) else {
        return false;
    };
    // SAFETY: our own live window on this thread.
    unsafe { PostMessageW(Some(window), PRESSED, WPARAM(0), LPARAM(0)) }.is_ok()
}

/// Calls back once for a press the hook counted; a panic must not cross into Windows.
fn tell() {
    HERE.with(|h| {
        let Some(here) = h.get() else {
            return;
        };
        let Some(left) = here.pending.get().checked_sub(1) else {
            return;
        };
        here.pending.set(left);
        let _ = catch_unwind(AssertUnwindSafe(|| (here.on)()));
    });
}

#[cfg(test)]
mod tests;
