//! A hidden window on its own thread that hears messages until it is stopped; it is never shown and never takes the focus.

use std::sync::mpsc;
use std::thread::JoinHandle;

use kx_platform::ui::UiError;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetMessageW, HWND_MESSAGE,
    MSG, PostThreadMessageW, RegisterClassW, WM_CLOSE, WM_QUIT, WNDCLASSW, WNDPROC,
    WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_POPUP,
};
use windows::core::PCWSTR;

use crate::{failed, refused};

/// Where the hidden window sits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    /// A top-level window, which hears Windows' broadcasts.
    TopLevel,
    /// A message-only window, which only a sender that finds it by class can reach.
    MessageOnly,
}

/// A running hidden window; `stop` ends it and its thread.
#[derive(Debug)]
pub(crate) struct Hidden {
    thread: u32,
    join: JoinHandle<()>,
}

impl Hidden {
    /// Starts a thread that runs `setup`, then makes a window of class `class` (UTF-16 with a null) with `proc`.
    ///
    /// # Errors
    /// `Refused` when Windows will not make the window.
    pub(crate) fn start(
        class: Vec<u16>,
        kind: Kind,
        proc: WNDPROC,
        setup: impl FnOnce() + Send + 'static,
    ) -> Result<Self, UiError> {
        let (tx, rx) = mpsc::channel();
        let join = std::thread::spawn(move || {
            setup();
            match window(PCWSTR(class.as_ptr()), kind, proc) {
                Ok(hwnd) => {
                    let _ = tx.send(Ok(this_thread()));
                    pump();
                    destroy(hwnd);
                }
                Err(e) => drop(tx.send(Err(e))),
            }
        });
        let thread = rx.recv().map_err(|_| failed())??;
        Ok(Self { thread, join })
    }

    /// Ends the window and its thread and waits for it; from that thread itself it does not wait.
    #[allow(unsafe_code, reason = "Posts a quit message to our own thread.")]
    pub(crate) fn stop(self) {
        // SAFETY: posts WM_QUIT to our own window thread; a thread already gone just refuses it.
        let _ = unsafe { PostThreadMessageW(self.thread, WM_QUIT, WPARAM(0), LPARAM(0)) };
        if self.join.thread().id() != std::thread::current().id() {
            let _ = self.join.join();
        }
    }
}

/// This thread's id.
#[allow(unsafe_code, reason = "A plain thread query.")]
fn this_thread() -> u32 {
    // SAFETY: plain query of the calling thread's id.
    unsafe { GetCurrentThreadId() }
}

/// Makes the hidden window; a class registered before, by this or an earlier thread, is reused.
#[allow(
    unsafe_code,
    reason = "Registers our class and makes a window that is never shown."
)]
fn window(class: PCWSTR, kind: Kind, proc: WNDPROC) -> Result<HWND, UiError> {
    // SAFETY: a plain query of our own program module.
    let module = unsafe { GetModuleHandleW(None) }.map_err(|e| refused(&e))?;
    let wc = WNDCLASSW {
        lpfnWndProc: proc,
        hInstance: module.into(),
        lpszClassName: class,
        ..WNDCLASSW::default()
    };
    // SAFETY: `wc` and its class name outlive the call; a class that exists already gives 0 and still serves.
    unsafe { RegisterClassW(&raw const wc) };
    let parent = (kind == Kind::MessageOnly).then_some(HWND_MESSAGE);
    let ex = WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE;
    // SAFETY: a window of our own class with no size, never shown; the class name outlives the call.
    unsafe {
        CreateWindowExW(
            ex,
            class,
            PCWSTR::null(),
            WS_POPUP,
            0,
            0,
            0,
            0,
            parent,
            None,
            Some(module.into()),
            None,
        )
    }
    .map_err(|e| refused(&e))
}

/// Windows' default handling for a message to one of our hidden windows, except `WM_CLOSE`, which is
/// ignored so another program cannot end the window; `stop` ends it.
#[allow(
    unsafe_code,
    reason = "Default handling in our hidden windows' procedures."
)]
pub(crate) unsafe extern "system" fn answer(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if msg == WM_CLOSE {
        return LRESULT::default();
    }
    // SAFETY: the caller passes a message for our own window, as Windows gave it.
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

/// Hears this thread's messages until `stop`'s `WM_QUIT` or an error.
#[allow(unsafe_code, reason = "The standard message loop.")]
fn pump() {
    let mut msg = MSG::default();
    loop {
        // SAFETY: the loop of the thread that owns the window; `msg` is a local it fills.
        let got = unsafe { GetMessageW(&raw mut msg, None, 0, 0) }.0;
        if ends(got, !msg.hwnd.is_invalid()) {
            break;
        }
        if got > 0 {
            // SAFETY: dispatches a message this thread received.
            unsafe { DispatchMessageW(&raw const msg) };
        }
    }
}

/// Whether `GetMessageW`'s answer `got` ends the loop: an error (-1), or a `WM_QUIT` (0) that names no
/// window, as `stop` posts it; a `WM_QUIT` another program posts to the window is ignored.
fn ends(got: i32, to_window: bool) -> bool {
    got < 0 || (got == 0 && !to_window)
}

/// Destroys our hidden window on its own thread.
#[allow(unsafe_code, reason = "Destroys a window this thread made.")]
fn destroy(hwnd: HWND) {
    // SAFETY: the window belongs to this thread, which made it.
    let _ = unsafe { DestroyWindow(hwnd) };
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows::Win32::UI::WindowsAndMessaging::{
        FindWindowExW, IsWindow, SendMessageW, WM_CLOSE,
    };

    #[test]
    fn only_a_thread_quit_or_an_error_ends_the_loop() {
        assert!(ends(0, false), "stop's WM_QUIT names no window");
        assert!(!ends(0, true), "a WM_QUIT posted to the window");
        assert!(ends(-1, false) && ends(-1, true), "an error");
        assert!(!ends(1, true) && !ends(1, false), "any other message");
    }

    #[test]
    #[allow(
        unsafe_code,
        reason = "Finds and messages the test's own hidden window."
    )]
    fn another_program_cannot_close_a_hidden_window() {
        let class = crate::wide(&format!("KeyXtendTest.{}.close", std::process::id()));
        let hidden = Hidden::start(class.clone(), Kind::TopLevel, Some(answer), || ()).unwrap();
        // SAFETY: the class name ends with a null; WM_CLOSE carries no pointers.
        let open = unsafe {
            let window = FindWindowExW(None, None, PCWSTR(class.as_ptr()), PCWSTR::null()).unwrap();
            SendMessageW(window, WM_CLOSE, None, None);
            IsWindow(Some(window)).as_bool()
        };
        hidden.stop();
        assert!(open, "the window ignored WM_CLOSE");
    }
}
