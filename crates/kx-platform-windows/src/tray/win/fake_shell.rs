//! A fake taskbar for the tray tests: it records each call, refuses what a test names, and can run the thread's other
//! messages inside a call, as the real one can.

use super::*;
use std::sync::Condvar;
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
}

/// A taskbar that records each call and refuses the kinds the test names.
#[derive(Debug, Default)]
pub(super) struct FakeShell {
    calls: Mutex<Vec<Call>>,
    heard: Condvar,
    refuse: Mutex<Vec<NOTIFY_ICON_MESSAGE>>,
    /// A window that gets the taskbar's restart message inside the next call, as a real call can run other messages.
    pub(super) restart_in: Mutex<Option<usize>>,
    /// An icon state the owner changes during the next add, as `set` can on its own thread.
    pub(super) change_in: Mutex<Option<Arc<Mutex<State>>>>,
}

impl FakeShell {
    /// The kinds of the calls heard since the last take, and the calls.
    pub(super) fn take(&self) -> (Vec<u32>, Vec<Call>) {
        let calls = std::mem::take(&mut *self.calls.lock().unwrap());
        (calls.iter().map(|c| c.what).collect(), calls)
    }

    /// Waits until `n` calls came from the icon's queue, then takes them.
    pub(super) fn wait(&self, n: usize) -> (Vec<u32>, Vec<Call>) {
        let calls = self.calls.lock().unwrap();
        let (calls, waited) = self
            .heard
            .wait_timeout_while(calls, PATIENCE, |c| c.len() < n)
            .unwrap();
        assert!(!waited.timed_out(), "{} calls, not {n}", calls.len());
        drop(calls);
        self.take()
    }

    pub(super) fn refuse(&self, kinds: &[NOTIFY_ICON_MESSAGE]) {
        *self.refuse.lock().unwrap() = kinds.to_vec();
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
        };
        self.calls.lock().unwrap().push(call);
        self.heard.notify_all();
        let change = self.change_in.lock().unwrap().take();
        if let Some(state) = change.filter(|_| what == NIM_ADD) {
            let mut s = lock(&state);
            s.tip = crate::tray::tip("newer");
            s.changes += 1;
        }
        let restart = self.restart_in.lock().unwrap().take();
        if let Some(window) = restart {
            send(
                HWND(std::ptr::without_provenance_mut(window)),
                restart_msg(),
                0,
                0,
            );
        }
        !self.refuse.lock().unwrap().contains(&what)
    }
}

/// Sends the icon's window a message and waits until its thread handled it.
#[allow(unsafe_code, reason = "Messages the test's own window.")]
pub(super) fn send(window: HWND, msg: u32, wparam: usize, lparam: isize) {
    // SAFETY: plain numbers to our own window; nothing points into memory.
    unsafe { SendMessageW(window, msg, Some(WPARAM(wparam)), Some(LPARAM(lparam))) };
}
