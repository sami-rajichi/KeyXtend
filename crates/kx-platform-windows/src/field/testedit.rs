//! A hidden test edit box on a thread of its own, holding a text with the caret at its end, and a UI Automation client
//! to read it. It is never shown and never takes the focus.
#![allow(
    unsafe_code,
    reason = "The tests make, fill and destroy their own hidden edit boxes."
)]

use std::sync::mpsc;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::Accessibility::{IUIAutomation, IUIAutomationElement};
use windows::Win32::UI::Controls::EM_SETSEL;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DestroyWindow, DispatchMessageW, ES_AUTOHSCROLL, GetMessageW, MSG,
    PostThreadMessageW, SendMessageW, WINDOW_EX_STYLE, WINDOW_STYLE, WM_QUIT, WS_POPUP,
};
use windows::core::{PCWSTR, w};

use kx_module_api::ports::FieldConfig;

use crate::com::Com;
use crate::scroll::testwin::WAIT;
use crate::uia::client;

/// The test box's place and size; it is never shown.
const BOX: (i32, i32, i32, i32) = (40, 40, 600, 40);
/// A `read_ms` no test read reaches.
pub(super) const LONG_MS: u32 = 5000;
/// A `read_chars` that holds every test box's text.
pub(super) const CHARS: u16 = 256;
/// A watch config no test read reaches.
pub(super) const CONFIG: FieldConfig = FieldConfig {
    read_ms: LONG_MS,
    read_chars: CHARS,
};

/// A hidden edit box on a thread of its own; dropping it destroys both.
pub(super) struct TestEdit {
    hwnd: isize,
    thread: u32,
    join: Option<JoinHandle<()>>,
}

impl TestEdit {
    /// A box holding `text`, its caret at the end, with the edit style bits `style` (`ES_PASSWORD`, `ES_READONLY`).
    pub(super) fn start(text: &str, style: i32) -> Self {
        let (made, ready) = mpsc::channel();
        let text = crate::wide(text);
        let join = thread::spawn(move || {
            let hwnd = make(&text, style);
            // SAFETY: a plain query of this thread's id.
            let thread = unsafe { GetCurrentThreadId() };
            made.send((hwnd.0.addr().cast_signed(), thread)).unwrap();
            let mut msg = MSG::default();
            // SAFETY: reads this thread's messages until the test posts the quit message.
            while unsafe { GetMessageW(&raw mut msg, None, 0, 0) }.0 > 0 {
                // SAFETY: hands a message this thread read to its own window.
                unsafe { DispatchMessageW(&raw const msg) };
            }
            // SAFETY: destroys the box this thread made.
            let _ = unsafe { DestroyWindow(hwnd) };
        });
        let (hwnd, thread) = ready.recv_timeout(WAIT).unwrap();
        Self {
            hwnd,
            thread,
            join: Some(join),
        }
    }

    /// The box's element in `auto`.
    pub(super) fn element(&self, auto: &IUIAutomation) -> IUIAutomationElement {
        element(auto, self.hwnd)
    }

    /// The box's raw handle, for a thread that makes its own client.
    pub(super) fn raw(&self) -> isize {
        self.hwnd
    }
}

impl Drop for TestEdit {
    fn drop(&mut self) {
        // SAFETY: asks the box's own thread to stop reading messages.
        let _ = unsafe { PostThreadMessageW(self.thread, WM_QUIT, WPARAM(0), LPARAM(0)) };
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
    }
}

/// The element of the test box with raw handle `raw` in `auto`.
pub(super) fn element(auto: &IUIAutomation, raw: isize) -> IUIAutomationElement {
    let hwnd = crate::window::from_raw(raw);
    // SAFETY: a plain lookup on a live client, for the test's own window.
    unsafe { auto.ElementFromHandle(hwnd) }.unwrap()
}

/// Makes the hidden edit box holding `text` (with its ending null), with `style` added, and puts the caret at its end.
fn make(text: &[u16], style: i32) -> HWND {
    let bits = ES_AUTOHSCROLL | style;
    let style = WS_POPUP | WINDOW_STYLE(bits.cast_unsigned());
    let (x, y, cx, cy) = BOX;
    let ex = WINDOW_EX_STYLE::default();
    // SAFETY: makes a hidden window of the system's edit class; `text` outlives the call.
    let hwnd = unsafe {
        let title = PCWSTR(text.as_ptr());
        CreateWindowExW(
            ex,
            w!("EDIT"),
            title,
            style,
            x,
            y,
            cx,
            cy,
            None,
            None,
            None,
            None,
        )
    }
    .unwrap();
    let end = text.len() - 1;
    let at = (Some(WPARAM(end)), Some(LPARAM(end.cast_signed())));
    // SAFETY: a plain message to the box this thread owns.
    unsafe { SendMessageW(hwnd, EM_SETSEL, at.0, at.1) };
    hwnd
}

/// A client for the tests, with COM on this thread while the guard lives.
pub(super) fn started() -> (Com, IUIAutomation) {
    let com = Com::start().unwrap();
    (com, client(LONG_MS).unwrap())
}

/// A deadline no test read reaches.
pub(super) fn far() -> Instant {
    Instant::now() + Duration::from_millis(u64::from(LONG_MS))
}
