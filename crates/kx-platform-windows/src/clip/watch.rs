//! The watch: a message-only window on its own thread (`kx-clip-watch`) hears each clipboard change through
//! `AddClipboardFormatListener` and tells its note there. Dropping the run removes the listener and ends the thread,
//! waiting `STOP_WAIT` at most.

use std::cell::OnceCell;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::PathBuf;
use std::time::Duration;

use kx_module_api::Redacted;
use kx_module_api::ports::{ClipConfig, ClipError, ClipNote, ClipRun, Clipboard, FileFact};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::DataExchange::{
    AddClipboardFormatListener, RemoveClipboardFormatListener,
};
use windows::Win32::UI::WindowsAndMessaging::{WM_CLIPBOARDUPDATE, WM_CREATE, WM_DESTROY};

use super::parts::Formats;
use super::{Marks, facts, read, table};
use crate::hidden::{Hidden, Kind, answer};
use crate::wide;

/// Class of the watch window, local to our process.
const CLASS: &str = "KeyXtend.ClipWatch";
/// The watch thread's name.
const THREAD: &str = "kx-clip-watch";
/// What `WM_CREATE` answers so the window is not made: Windows refused the listener.
const NOT_MADE: LRESULT = LRESULT(-1);
/// How long dropping a watch waits for its thread before it leaves the thread to end alone.
const STOP_WAIT: Duration = Duration::from_secs(1);

/// Where a watch tells its notes.
type Tell = Box<dyn Fn(ClipNote) + Send>;

/// What a watch thread holds.
struct Here {
    cfg: ClipConfig,
    marks: Marks,
    formats: Formats,
    tell: Tell,
}

thread_local! {
    /// Set once per watch thread before its window exists, and never borrowed mutably.
    static HERE: OnceCell<Here> = const { OnceCell::new() };
}

/// The Windows clipboard: it watches copies and checks copied paths.
#[derive(Debug, Default)]
pub struct WinClipboard {
    /// The markers to skip in place of the shipped ones, for tests.
    marks: Option<Marks>,
}

impl WinClipboard {
    /// A clipboard whose watch skips the copies the shipped table marks.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// A clipboard whose watch skips only `marks`, so a test can hear its own writes.
    #[cfg(test)]
    pub(crate) fn with_marks(marks: Marks) -> Self {
        Self { marks: Some(marks) }
    }
}

impl Clipboard for WinClipboard {
    fn watch(&self, cfg: ClipConfig, tell: Tell) -> Result<Box<dyn ClipRun>, ClipError> {
        let table = table()?;
        let marks = self.marks.clone().unwrap_or_else(|| table.marks());
        let formats = read::ids(table)?;
        let here = Here {
            cfg,
            marks,
            formats,
            tell,
        };
        let setup = move || drop(HERE.with(|h| h.set(here)));
        let hidden = Hidden::named(THREAD, wide(CLASS), Kind::MessageOnly, Some(proc), setup);
        Ok(Box::new(Run(Some(hidden.map_err(|_| ClipError::Refused)?))))
    }

    fn files(&self, paths: &Redacted<Vec<PathBuf>>) -> Vec<FileFact> {
        facts(paths)
    }
}

/// A running watch; dropping it removes the listener and ends its thread.
struct Run(Option<Hidden>);

impl ClipRun for Run {}

impl Drop for Run {
    /// Waits `STOP_WAIT` at most. A thread an app's late rendering holds ends alone once it is let go, and may tell
    /// that one copy's note after the drop.
    fn drop(&mut self) {
        let Some(hidden) = self.0.take() else {
            return;
        };
        if !hidden.stop_within(STOP_WAIT) {
            let wait_ms = STOP_WAIT.as_millis();
            tracing::warn!(wait_ms, "the clipboard watch did not stop in time");
        }
    }
}

/// Listens from creation to destruction, and reads the clipboard on each change.
#[allow(unsafe_code, reason = "The window procedure of our watch window.")]
unsafe extern "system" fn proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_CREATE => {
            if !listen(hwnd) {
                return NOT_MADE;
            }
        }
        WM_DESTROY => unlisten(hwnd),
        WM_CLIPBOARDUPDATE => {
            heard(hwnd);
            return LRESULT::default();
        }
        _ => {}
    }
    // SAFETY: our own window's message, with its own values.
    unsafe { answer(hwnd, msg, wparam, lparam) }
}

/// Starts hearing clipboard changes in our window; false when Windows refuses.
#[allow(unsafe_code, reason = "A plain call on our own window.")]
fn listen(hwnd: HWND) -> bool {
    // SAFETY: our own live window, made on this thread.
    unsafe { AddClipboardFormatListener(hwnd) }.is_ok()
}

/// Stops hearing clipboard changes as our window goes.
#[allow(unsafe_code, reason = "A plain call on our own window.")]
fn unlisten(hwnd: HWND) {
    // SAFETY: our own window, still alive while it is destroyed; one that never listened is refused harmlessly.
    let _ = unsafe { RemoveClipboardFormatListener(hwnd) };
}

/// Reads the clipboard once and tells its note; a panic must not cross into Windows.
fn heard(hwnd: HWND) {
    HERE.with(|h| {
        if let Some(here) = h.get() {
            let tell = || (here.tell)(read::note(hwnd, &here.cfg, &here.marks, &here.formats));
            let _ = catch_unwind(AssertUnwindSafe(tell));
        }
    });
}
