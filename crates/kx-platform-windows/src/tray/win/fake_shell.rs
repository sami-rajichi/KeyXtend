//! A fake taskbar for the tray tests: it records each call, refuses and holds what a test names, and can run the
//! thread's other messages inside a call, as the real one can.

use super::*;
use std::sync::Condvar;
use std::thread::ThreadId;
use std::time::Duration;
use windows::Win32::UI::WindowsAndMessaging::SendMessageW;

/// How long a test waits for calls its icon's thread makes, before it fails.
const PATIENCE: Duration = Duration::from_secs(10);

/// One call the fake taskbar heard.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Call {
    pub(super) what: u32,
    pub(super) window: usize,
    pub(super) id: u32,
    pub(super) flags: u32,
    pub(super) msg: u32,
    pub(super) tip: String,
    pub(super) icon: bool,
    pub(super) version: u32,
    /// The thread that made the call.
    pub(super) thread: ThreadId,
}

/// A taskbar that records each call and refuses the kinds the test names.
#[derive(Debug, Default)]
pub(super) struct FakeShell {
    calls: Mutex<Vec<Call>>,
    heard: Condvar,
    refuse: Mutex<Vec<NOTIFY_ICON_MESSAGE>>,
    /// A window that gets the taskbar's restart message inside the next call, as a real call can run other messages.
    pub(super) restart_in: Mutex<Option<usize>>,
    /// Calls of this kind wait inside the call until it changes, as a slow taskbar makes them.
    held: Mutex<Option<NOTIFY_ICON_MESSAGE>>,
    freed: Condvar,
}

impl FakeShell {
    /// The kinds of the calls heard since the last take, and the calls.
    pub(super) fn take(&self) -> (Vec<u32>, Vec<Call>) {
        let calls = std::mem::take(&mut *locked(&self.calls));
        (calls.iter().map(|c| c.what).collect(), calls)
    }

    /// Waits until `n` calls came from the icon's queue, then takes them; a failure frees the lock first, so the
    /// icon's thread never meets a poisoned one inside its window procedure.
    pub(super) fn wait(&self, n: usize) -> (Vec<u32>, Vec<Call>) {
        let calls = locked(&self.calls);
        let (calls, waited) = self
            .heard
            .wait_timeout_while(calls, PATIENCE, |c| c.len() < n)
            .unwrap_or_else(PoisonError::into_inner);
        let got = calls.len();
        drop(calls);
        assert!(!waited.timed_out(), "{got} calls, not {n}");
        self.take()
    }

    /// Refuses later calls of the kinds in `kinds`.
    pub(super) fn refuse(&self, kinds: &[NOTIFY_ICON_MESSAGE]) {
        *locked(&self.refuse) = kinds.to_vec();
    }

    /// Holds later calls of kind `what` inside the call, or lets every held call go with `None`.
    pub(super) fn hold(&self, what: Option<NOTIFY_ICON_MESSAGE>) {
        *locked(&self.held) = what;
        self.freed.notify_all();
    }
}

impl Shell for FakeShell {
    #[allow(unsafe_code, reason = "Reads the version from the record's union.")]
    fn notify(&self, what: NOTIFY_ICON_MESSAGE, data: &NOTIFYICONDATAW) -> bool {
        let len = data.szTip.iter().position(|u| *u == 0).unwrap_or(TIP_UNITS);
        let call = Call {
            what: what.0,
            window: data.hWnd.0.addr(),
            id: data.uID,
            flags: data.uFlags.0,
            msg: data.uCallbackMessage,
            tip: String::from_utf16_lossy(&data.szTip[..len]),
            icon: !data.hIcon.is_invalid(),
            // SAFETY: both union fields are a u32, so either reading is defined.
            version: unsafe { data.Anonymous.uVersion },
            thread: std::thread::current().id(),
        };
        locked(&self.calls).push(call);
        self.heard.notify_all();
        // A call held too long goes on, so the test that forgot it fails on its own asserts.
        let hold = locked(&self.held);
        let (hold, _) = self
            .freed
            .wait_timeout_while(hold, PATIENCE, |h| *h == Some(what))
            .unwrap_or_else(PoisonError::into_inner);
        drop(hold);
        let restart = locked(&self.restart_in).take();
        if let Some(window) = restart {
            send(
                HWND(std::ptr::without_provenance_mut(window)),
                restart_msg(),
                0,
                0,
            );
        }
        !locked(&self.refuse).contains(&what)
    }
}

/// Locks `m`, taking it even when a failed test poisoned it; the fake runs inside the icon's window procedure.
fn locked<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Sends the icon's window a message and waits until its thread handled it.
#[allow(unsafe_code, reason = "Messages the test's own window.")]
pub(super) fn send(window: HWND, msg: u32, wparam: usize, lparam: isize) {
    // SAFETY: plain numbers to our own window; nothing points into memory.
    unsafe { SendMessageW(window, msg, Some(WPARAM(wparam)), Some(LPARAM(lparam))) };
}
