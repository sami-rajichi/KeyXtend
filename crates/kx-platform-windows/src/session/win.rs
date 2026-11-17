//! The watch window: hidden and top-level on its own thread (`kx-session`), it registers for session and power
//! notes when made, unregisters when destroyed, and tells each change once. Dropping the run ends the thread.

use std::cell::{Cell, OnceCell};
use std::panic::{AssertUnwindSafe, catch_unwind};

use kx_module_api::ports::{SessionError, SessionNote, SessionRun, SessionWatch};
use windows::Win32::Foundation::{HANDLE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::Power::{
    HPOWERNOTIFY, RegisterSuspendResumeNotification, UnregisterSuspendResumeNotification,
};
use windows::Win32::System::RemoteDesktop::{
    NOTIFY_FOR_THIS_SESSION, WTSRegisterSessionNotification, WTSUnRegisterSessionNotification,
};
use windows::Win32::UI::WindowsAndMessaging::{DEVICE_NOTIFY_WINDOW_HANDLE, WM_CREATE, WM_DESTROY};

use super::{Last, table};
use crate::hidden::{Hidden, Kind, answer};
use crate::wide;

/// Class of the watch window, local to our process.
pub(crate) const CLASS: &str = "KeyXtend.SessionWatch";
/// The watch thread's name.
const THREAD: &str = "kx-session";
/// What `WM_CREATE` answers so the window is not made: Windows refused a registration.
const NOT_MADE: LRESULT = LRESULT(-1);
/// What a window answers a power note it took.
const TAKEN: LRESULT = LRESULT(1);

/// Where a watch tells its notes.
type Tell = Box<dyn Fn(SessionNote) + Send>;

/// What a watch thread holds.
struct Here {
    tell: Tell,
    last: Cell<Last>,
    power: Cell<Option<HPOWERNOTIFY>>,
}

thread_local! {
    /// Set once per watch thread before its window exists, and never borrowed mutably.
    static HERE: OnceCell<Here> = const { OnceCell::new() };
}

/// The Windows session watch.
#[derive(Clone, Copy, Debug, Default)]
pub struct WinSessionWatch;

impl SessionWatch for WinSessionWatch {
    fn watch(&self, tell: Tell) -> Result<Box<dyn SessionRun>, SessionError> {
        table().ok_or(SessionError::Unavailable)?;
        let here = Here {
            tell,
            last: Cell::default(),
            power: Cell::new(None),
        };
        let setup = move || drop(HERE.with(|h| h.set(here)));
        let hidden = Hidden::named(THREAD, wide(CLASS), Kind::TopLevel, Some(proc), setup);
        Ok(Box::new(Run(Some(
            hidden.map_err(|_| SessionError::Refused)?,
        ))))
    }
}

/// A running watch; dropping it unregisters and ends its thread.
struct Run(Option<Hidden>);

impl SessionRun for Run {}

impl Drop for Run {
    fn drop(&mut self) {
        if let Some(hidden) = self.0.take() {
            hidden.stop();
        }
    }
}

/// Registers from creation to destruction, and tells each change it hears.
#[allow(unsafe_code, reason = "The window procedure of our watch window.")]
unsafe extern "system" fn proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_CREATE if !register(hwnd) => return NOT_MADE,
        WM_DESTROY => unregister(hwnd),
        _ if table().is_some_and(|t| t.names(msg)) => {
            heard(msg, wparam.0);
            return TAKEN;
        }
        _ => {}
    }
    // SAFETY: our own window's message, with its own values.
    unsafe { answer(hwnd, msg, wparam, lparam) }
}

/// Registers our window for this session's changes and for sleep and wake; false when Windows refuses either.
#[allow(unsafe_code, reason = "Plain calls on our own window.")]
fn register(hwnd: HWND) -> bool {
    // SAFETY: our own live window, made on this thread.
    if unsafe { WTSRegisterSessionNotification(hwnd, NOTIFY_FOR_THIS_SESSION) }.is_err() {
        return false;
    }
    // SAFETY: our own live window, named as a window handle as the flag says.
    let power =
        unsafe { RegisterSuspendResumeNotification(HANDLE(hwnd.0), DEVICE_NOTIFY_WINDOW_HANDLE) };
    HERE.with(|h| {
        if let Some(here) = h.get() {
            here.power.set(power.as_ref().ok().copied());
        }
    });
    power.is_ok()
}

/// Unregisters our window as it goes; one that never registered is refused harmlessly.
#[allow(unsafe_code, reason = "Plain calls on our own window.")]
fn unregister(hwnd: HWND) {
    // SAFETY: our own window, still alive while it is destroyed.
    let _ = unsafe { WTSUnRegisterSessionNotification(hwnd) };
    if let Some(power) = HERE.with(|h| h.get().and_then(|here| here.power.take())) {
        // SAFETY: the registration made for this window, ended once, here.
        let _ = unsafe { UnregisterSuspendResumeNotification(power) };
    }
}

/// Tells the change `msg` with `wparam` tells, unless it repeats the last one; a panic must not cross into Windows.
fn heard(msg: u32, wparam: usize) {
    HERE.with(|h| {
        let (Some(here), Some(note)) = (h.get(), table().and_then(|t| t.note(msg, wparam))) else {
            return;
        };
        let mut last = here.last.take();
        let fresh = last.fresh(note);
        here.last.set(last);
        if fresh {
            let _ = catch_unwind(AssertUnwindSafe(|| (here.tell)(note)));
        }
    });
}

#[cfg(test)]
mod tests;
