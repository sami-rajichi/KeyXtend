//! Clipboard history listener for G20: a hidden window told of every clipboard change.
#![cfg(windows)]

use std::cell::RefCell;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::JoinHandle;
use std::time::Duration;

use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::DataExchange::{
    AddClipboardFormatListener, RemoveClipboardFormatListener,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetMessageW, HWND_MESSAGE,
    MSG, PM_NOREMOVE, PeekMessageW, PostThreadMessageW, RegisterClassW, WINDOW_EX_STYLE,
    WINDOW_STYLE, WM_CLIPBOARDUPDATE, WM_QUIT, WNDCLASSW,
};
use windows::core::{PCWSTR, w};

use crate::{clip, cliptext};

/// Formats whose presence means "leave this copy out" (Windows clipboard-history conventions).
pub const EXCLUDE: [&str; 2] = [
    "ExcludeClipboardContentFromMonitorProcessing",
    "Clipboard Viewer Ignore",
];
/// This format holding 0 also means "leave it out".
pub const HISTORY_FLAG: &str = "CanIncludeInClipboardHistory";
/// Every exclusion marker, for the test copies.
pub const MARKS: [&str; 3] = [EXCLUDE[0], EXCLUDE[1], HISTORY_FLAG];
/// Class of the hidden listener window.
const CLASS: PCWSTR = w!("KeyXtendSpikeClipListener");

/// True when the formats on the clipboard ask history tools to leave the copy out.
pub fn excluded(names: &[String], history_flag: Option<u32>) -> bool {
    names.iter().any(|n| EXCLUDE.contains(&n.as_str())) || history_flag == Some(0)
}

/// One clipboard change, as the listener saw it.
pub enum Update {
    /// It carried an exclusion marker, so its text was never read.
    Skipped,
    /// It held no text.
    NoText,
    /// Its text, capped.
    Text(String),
}

/// What the listener saw: every change, the skipped ones, and the kept texts in order.
#[derive(Default)]
pub struct Tally {
    /// Changes seen.
    pub seen: usize,
    /// Changes left out for an exclusion marker.
    pub skipped: usize,
    /// Kept texts, oldest first.
    pub entries: Vec<String>,
}

impl Tally {
    /// Counts `u`, and keeps its text if it has one.
    pub fn add(&mut self, u: Update) {
        self.seen += 1;
        match u {
            Update::Skipped => self.skipped += 1,
            Update::NoText => {}
            Update::Text(t) => self.entries.push(t),
        }
    }
}

/// How the listener opens and reads the clipboard.
#[derive(Clone, Copy)]
pub struct Reads {
    /// Longest text kept, in characters.
    pub max_chars: usize,
    /// Longest wait to open the clipboard, in ms.
    pub open_ms: u64,
    /// Poll step while waiting, in ms.
    pub poll_ms: u64,
}

/// One report from the listener: an update, or why the clipboard could not be read.
type Report = Result<Update, String>;

thread_local! {
    /// Where the window procedure sends each report, on the listener thread only.
    static SINK: RefCell<Option<(Sender<Report>, Reads)>> = const { RefCell::new(None) };
}

/// Reads the clipboard once for window `hwnd`: markers first, and the text only when none says to leave it out.
fn read(hwnd: HWND, r: Reads) -> Report {
    let open = clip::open_for(hwnd, r.open_ms, r.poll_ms)?;
    if excluded(&clip::names(&open), clip::dword(&open, HISTORY_FLAG)) {
        return Ok(Update::Skipped);
    }
    Ok(match cliptext::unicode(&open, r.max_chars)? {
        Some(t) => Update::Text(t),
        None => Update::NoText,
    })
}

#[allow(unsafe_code, reason = "Default handling for our own window.")]
unsafe extern "system" fn proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if msg == WM_CLIPBOARDUPDATE {
        SINK.with(|s| {
            if let Some((tx, r)) = s.borrow().as_ref() {
                let _ = tx.send(read(hwnd, *r));
            }
        });
        return LRESULT(0);
    }
    // SAFETY: default handling of a message for our own window.
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

/// The hidden message-only window that receives clipboard updates.
#[allow(unsafe_code, reason = "Window calls with our own module and class.")]
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
        RegisterClassW(&raw const wc);
        let style = (WINDOW_EX_STYLE(0), WINDOW_STYLE(0));
        CreateWindowExW(
            style.0,
            CLASS,
            w!(""),
            style.1,
            0,
            0,
            0,
            0,
            Some(HWND_MESSAGE),
            None,
            Some(inst.into()),
            None,
        )
        .map_err(|e| format!("listener window: {e}"))
    }
}

/// The listener thread: makes the window, reports its thread id, and loops until told to quit.
#[allow(unsafe_code, reason = "Creates our queue before anyone posts to it.")]
fn listen(tx: Sender<Report>, r: Reads, ready: &Sender<Result<u32, String>>) {
    let mut msg = MSG::default();
    // SAFETY: creates this thread's queue before anyone posts to it.
    let tid = unsafe {
        let _ = PeekMessageW(&raw mut msg, None, 0, 0, PM_NOREMOVE);
        GetCurrentThreadId()
    };
    SINK.with(|s| *s.borrow_mut() = Some((tx, r)));
    let hwnd = match window() {
        Ok(h) => h,
        Err(e) => return drop(ready.send(Err(e))),
    };
    // SAFETY: our live window; removed below before it is destroyed.
    if let Err(e) = unsafe { AddClipboardFormatListener(hwnd) } {
        let _ = ready.send(Err(format!("AddClipboardFormatListener: {e}")));
        // SAFETY: our own window.
        return drop(unsafe { DestroyWindow(hwnd) });
    }
    let _ = ready.send(Ok(tid));
    // SAFETY: a plain message loop on this thread; it ends on WM_QUIT (0) or an error (-1).
    while unsafe { GetMessageW(&raw mut msg, None, 0, 0) }.0 > 0 {
        // SAFETY: a message from our own queue.
        unsafe { DispatchMessageW(&raw const msg) };
    }
    // SAFETY: our own window, listened to above.
    unsafe {
        let _ = RemoveClipboardFormatListener(hwnd);
        let _ = DestroyWindow(hwnd);
    }
}

/// A running listener; it stops when dropped.
pub struct Listener {
    tid: u32,
    thread: Option<JoinHandle<()>>,
    rx: Receiver<Report>,
}

impl Listener {
    /// Starts listening on its own thread.
    pub fn start(r: Reads) -> Result<Self, String> {
        let (tx, rx) = mpsc::channel();
        let (ready_tx, ready_rx) = mpsc::channel();
        let thread = std::thread::spawn(move || listen(tx, r, &ready_tx));
        let tid = ready_rx
            .recv()
            .map_err(|_| "the listener thread ended".to_string())??;
        Ok(Self {
            tid,
            thread: Some(thread),
            rx,
        })
    }

    /// The next update, if one comes within `wait_ms`.
    pub fn next(&self, wait_ms: u64) -> Option<Report> {
        self.rx.recv_timeout(Duration::from_millis(wait_ms)).ok()
    }
}

impl Drop for Listener {
    #[allow(unsafe_code, reason = "A plain post to our listener thread's queue.")]
    fn drop(&mut self) {
        // SAFETY: a plain post to our listener thread's queue.
        let _ = unsafe { PostThreadMessageW(self.tid, WM_QUIT, WPARAM(0), LPARAM(0)) };
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(n: &[&str]) -> Vec<String> {
        n.iter().map(ToString::to_string).collect()
    }

    #[test]
    fn plain_text_is_kept() {
        assert!(!excluded(&names(&["Rich Text Format"]), None));
    }

    #[test]
    fn a_monitor_or_viewer_exclusion_skips_the_copy_even_with_text() {
        for name in EXCLUDE {
            assert!(excluded(&names(&["HTML Format", name]), None), "{name}");
        }
    }

    #[test]
    fn the_history_flag_skips_only_when_it_is_zero() {
        assert!(excluded(&names(&[HISTORY_FLAG]), Some(0)));
        assert!(!excluded(&names(&[HISTORY_FLAG]), Some(1)));
    }

    #[test]
    fn two_quick_updates_are_both_counted() {
        let mut t = Tally::default();
        t.add(Update::Text("a".to_string()));
        t.add(Update::Text("a".to_string()));
        t.add(Update::Skipped);
        t.add(Update::NoText);
        assert_eq!((t.seen, t.skipped, t.entries.len()), (4, 1, 2));
    }

    #[test]
    fn marks_cover_every_exclusion() {
        assert!(MARKS.iter().all(|m| excluded(&names(&[m]), Some(0))));
    }
}
