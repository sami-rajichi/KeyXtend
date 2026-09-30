//! A hidden window on its own thread that hears messages until it is stopped; it is never shown and never takes the focus.

use std::sync::mpsc;
use std::thread::JoinHandle;

use kx_platform::ui::UiError;
use windows::Win32::Foundation::{E_FAIL, HWND, LPARAM, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DestroyWindow, DispatchMessageW, GetMessageW, HWND_MESSAGE, MSG,
    PostThreadMessageW, RegisterClassW, WM_QUIT, WNDCLASSW, WNDPROC, WS_EX_NOACTIVATE,
    WS_EX_TOOLWINDOW, WS_POPUP,
};
use windows::core::PCWSTR;

use crate::refused;

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
        let thread = rx.recv().map_err(|_| UiError::Refused(E_FAIL.0))??;
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

/// Hears this thread's messages until `WM_QUIT` or an error.
#[allow(unsafe_code, reason = "The standard message loop.")]
fn pump() {
    let mut msg = MSG::default();
    // SAFETY: the loop of the thread that owns the window; 0 is WM_QUIT and -1 an error, and both end it.
    while unsafe { GetMessageW(&raw mut msg, None, 0, 0) }.0 > 0 {
        // SAFETY: dispatches a message this thread received.
        unsafe { DispatchMessageW(&raw const msg) };
    }
}

/// Destroys our hidden window on its own thread.
#[allow(unsafe_code, reason = "Destroys a window this thread made.")]
fn destroy(hwnd: HWND) {
    // SAFETY: the window belongs to this thread, which made it.
    let _ = unsafe { DestroyWindow(hwnd) };
}
