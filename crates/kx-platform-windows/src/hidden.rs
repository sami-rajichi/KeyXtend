//! A hidden window on its own thread that hears messages until it is stopped; it is never shown and never takes the focus.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::thread::{Builder, JoinHandle};

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
    /// Set by `stop` before its `WM_QUIT`, so a quit another program posts is ignored.
    stopping: Arc<AtomicBool>,
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
        Self::spawn(Builder::new(), class, kind, proc, setup)
    }

    /// Like `start`, on a thread named `name`.
    ///
    /// # Errors
    /// `Refused` when Windows will not make the window or the thread.
    pub(crate) fn named(
        name: &str,
        class: Vec<u16>,
        kind: Kind,
        proc: WNDPROC,
        setup: impl FnOnce() + Send + 'static,
    ) -> Result<Self, UiError> {
        Self::spawn(
            Builder::new().name(name.to_owned()),
            class,
            kind,
            proc,
            setup,
        )
    }

    /// Starts the window's thread from `builder`.
    fn spawn(
        builder: Builder,
        class: Vec<u16>,
        kind: Kind,
        proc: WNDPROC,
        setup: impl FnOnce() + Send + 'static,
    ) -> Result<Self, UiError> {
        let (tx, rx) = mpsc::channel();
        let stopping = Arc::new(AtomicBool::new(false));
        let asked = Arc::clone(&stopping);
        let spawned = builder.spawn(move || {
            setup();
            match window(PCWSTR(class.as_ptr()), kind, proc) {
                Ok(hwnd) => {
                    let _ = tx.send(Ok(this_thread()));
                    pump(&asked);
                    destroy(hwnd);
                }
                Err(e) => drop(tx.send(Err(e))),
            }
        });
        let join = spawned.map_err(|_| failed())?;
        let thread = rx.recv().map_err(|_| failed())??;
        Ok(Self {
            thread,
            join,
            stopping,
        })
    }

    /// Ends the window and its thread, and waits for it as `waits` says.
    #[allow(unsafe_code, reason = "Posts a quit message to our own thread.")]
    pub(crate) fn stop(self) {
        self.stopping.store(true, Ordering::SeqCst);
        // SAFETY: posts WM_QUIT to our own window thread; a thread already gone just refuses it.
        let posted =
            unsafe { PostThreadMessageW(self.thread, WM_QUIT, WPARAM(0), LPARAM(0)) }.is_ok();
        if waits(
            posted,
            self.join.thread().id() == std::thread::current().id(),
        ) {
            let _ = self.join.join();
        }
    }
}

/// Whether `stop` waits for the window's thread: only when its `WM_QUIT` was `posted`, so a full queue cannot hang
/// quitting, and never from that thread itself (`own`).
fn waits(posted: bool, own: bool) -> bool {
    posted && !own
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

/// Hears this thread's messages until `GetMessageW` fails or `stopping` is set; then any message ends it, `stop`'s
/// `WM_QUIT` or, when that post was refused, whatever comes next.
#[allow(unsafe_code, reason = "The standard message loop.")]
fn pump(stopping: &AtomicBool) {
    let mut msg = MSG::default();
    loop {
        // SAFETY: the loop of the thread that owns the window; `msg` is a local it fills.
        let got = unsafe { GetMessageW(&raw mut msg, None, 0, 0) }.0;
        if ends(got, stopping.load(Ordering::SeqCst)) {
            break;
        }
        if got > 0 {
            // SAFETY: dispatches a message this thread received.
            unsafe { DispatchMessageW(&raw const msg) };
        }
    }
}

/// Whether `GetMessageW`'s answer `got` ends the loop: an error (-1), or any message once `stopping` is set; a
/// `WM_QUIT` another program posts before then is ignored.
fn ends(got: i32, stopping: bool) -> bool {
    got < 0 || stopping
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
    fn only_stop_or_an_error_ends_the_loop() {
        assert!(ends(0, true), "stop's WM_QUIT");
        assert!(!ends(0, false), "a WM_QUIT another program posts");
        assert!(
            ends(1, true),
            "any message once stopping, so a refused quit still ends it"
        );
        assert!(!ends(1, false), "any other message");
        assert!(ends(-1, false) && ends(-1, true), "an error");
    }

    #[test]
    fn stop_waits_only_for_a_quit_it_posted_and_never_from_the_window_thread() {
        assert!(waits(true, false), "a posted quit ends the loop");
        assert!(!waits(false, false), "a full queue refused the quit");
        assert!(!waits(true, true) && !waits(false, true), "its own thread");
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
