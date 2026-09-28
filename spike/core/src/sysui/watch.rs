//! A hidden top-level window that hears Windows' look change and reports each real change once.

use std::cell::{OnceCell, RefCell};
use std::sync::mpsc;
use std::thread::JoinHandle;

use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetMessageW, MSG,
    PostThreadMessageW, RegisterClassW, WM_DWMCOLORIZATIONCOLORCHANGED, WM_QUIT, WM_SETTINGCHANGE,
    WM_SYSCOLORCHANGE, WM_THEMECHANGED, WNDCLASSW, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_POPUP,
};
use windows::core::{PCWSTR, w};

use super::{SystemLook, read};
use crate::com::Com;

/// Class of the hidden watcher window; broadcasts reach only top-level windows, never message-only ones.
const CLASS: PCWSTR = w!("KeyXtendSpikeLookWatch");

/// The callback for a changed look.
type OnChange = Box<dyn Fn(SystemLook)>;

thread_local! {
    /// The last look seen, on the watcher thread only.
    static LAST: RefCell<Option<SystemLook>> = const { RefCell::new(None) };
    /// The callback, set once and never borrowed mutably, so it may run while another message arrives.
    static ON_CHANGE: OnceCell<OnChange> = const { OnceCell::new() };
}

/// Messages Windows sends when the mode, accent or high contrast may have changed.
const CHANGES: [u32; 4] = [
    WM_SETTINGCHANGE,
    WM_SYSCOLORCHANGE,
    WM_THEMECHANGED,
    WM_DWMCOLORIZATIONCOLORCHANGED,
];

/// `now` when it differs from `last`, which then becomes `now`; a first look is always new.
pub fn changed(last: &mut Option<SystemLook>, now: SystemLook) -> Option<SystemLook> {
    if last.as_ref() == Some(&now) {
        return None;
    }
    *last = Some(now.clone());
    Some(now)
}

unsafe extern "system" fn proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if CHANGES.contains(&msg) {
        // The borrow ends before the callback runs.
        let new = LAST.with(|l| changed(&mut l.borrow_mut(), read()));
        if let Some(look) = new {
            ON_CHANGE.with(|f| f.get().map(|f| f(look)));
        }
    }
    // SAFETY: default handling of a message for our own window.
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

/// The hidden watcher window; it is never shown and never takes focus.
fn window() -> Result<HWND, String> {
    // SAFETY: plain window calls with our own module and class; a second register fails harmlessly.
    unsafe {
        let inst = GetModuleHandleW(None).map_err(|e| e.to_string())?;
        let wc = WNDCLASSW {
            lpfnWndProc: Some(proc),
            hInstance: inst.into(),
            lpszClassName: CLASS,
            ..Default::default()
        };
        RegisterClassW(&wc);
        let ex = WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE;
        CreateWindowExW(
            ex,
            CLASS,
            w!(""),
            WS_POPUP,
            0,
            0,
            0,
            0,
            None,
            None,
            Some(inst.into()),
            None,
        )
        .map_err(|e| e.to_string())
    }
}

/// The watcher thread: calls `on_change` with each new look until told to quit.
pub struct Watch {
    thread: u32,
    join: Option<JoinHandle<()>>,
}

/// Starts watching; `on_change` runs on the watcher thread, once per real change.
pub fn watch(on_change: impl Fn(SystemLook) + Send + 'static) -> Result<Watch, String> {
    let (tx, rx) = mpsc::channel();
    let join = std::thread::spawn(move || {
        let hwnd = match Com::start().and_then(|c| window().inspect(|_| c.keep())) {
            Ok(h) => h,
            Err(e) => return drop(tx.send(Err(e))),
        };
        LAST.with(|l| *l.borrow_mut() = Some(read()));
        ON_CHANGE.with(|f| f.set(Box::new(on_change)).ok());
        // SAFETY: plain query of this thread's id.
        let _ = tx.send(Ok(unsafe { GetCurrentThreadId() }));
        let mut msg = MSG::default();
        // SAFETY: a standard message loop on the thread that owns the window.
        while unsafe { GetMessageW(&mut msg, None, 0, 0) }.as_bool() {
            // SAFETY: dispatches a message this thread received.
            unsafe { DispatchMessageW(&msg) };
        }
        // SAFETY: the window belongs to this thread.
        let _ = unsafe { DestroyWindow(hwnd) };
    });
    let thread = rx.recv().map_err(|e| e.to_string())??;
    Ok(Watch {
        thread,
        join: Some(join),
    })
}

impl Drop for Watch {
    fn drop(&mut self) {
        // SAFETY: posts WM_QUIT to our own watcher thread's queue.
        let _ = unsafe { PostThreadMessageW(self.thread, WM_QUIT, WPARAM(0), LPARAM(0)) };
        if let Some(j) = self.join.take() {
            let _ = j.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_real_change_is_reported_once() {
        let dark = SystemLook {
            dark: true,
            ..SystemLook::default()
        };
        let mut last = Some(SystemLook::default());
        assert_eq!(changed(&mut last, SystemLook::default()), None, "no change");
        assert_eq!(changed(&mut last, dark.clone()), Some(dark.clone()));
        assert_eq!(changed(&mut last, dark), None, "the same change again");
    }
}
