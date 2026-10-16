//! The run's ping thread: it asks the target window for a `WM_NULL` answer within `read_ms`, one ask at a time, so a
//! window that stopped reading its messages gets no turn long before Windows calls it hung.

use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use windows::Win32::Foundation::{ERROR_ACCESS_DENIED, GetLastError, LPARAM, WIN32_ERROR, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    SMTO_ABORTIFHUNG, SMTO_ERRORONEXIT, SendMessageTimeoutW, WM_NULL,
};

use super::handle;
use crate::lock;
use crate::scroll::Pings;

/// The ping thread's name.
const THREAD: &str = "kx-scroll-ping";

/// The pings, and the signal that one ended.
type Shared = Arc<(Mutex<Pings>, Condvar)>;

/// The ping thread; dropping it ends the thread once its running ask returns.
pub(super) struct Pinger {
    /// The thread's inbox; none when it did not start, which holds no window back.
    asks: Option<Sender<u64>>,
    shared: Shared,
    /// How long a ping may stay out: `check_ms`.
    wait: Duration,
}

impl Pinger {
    /// Starts the thread: each ask waits `read_ms` at most for its answer, and one out past `check_ms` makes its
    /// window busy.
    pub(super) fn start(read_ms: u32, check_ms: u32) -> Self {
        let shared = Shared::default();
        let (asks, inbox) = mpsc::channel::<u64>();
        let theirs = Arc::clone(&shared);
        let body = move || {
            for window in inbox {
                let answered = answers(window, read_ms);
                let (pings, ended) = &*theirs;
                lock(pings).done(answered);
                ended.notify_all();
            }
        };
        let asks = match thread::Builder::new().name(THREAD.to_owned()).spawn(body) {
            Ok(_) => Some(asks),
            Err(e) => {
                tracing::warn!(error = %e, "no pings: their thread did not start");
                None
            }
        };
        let wait = Duration::from_millis(u64::from(check_ms));
        Self { asks, shared, wait }
    }

    /// Pings `window`, unless a ping is out.
    pub(super) fn ask(&self, window: u64) {
        let Some(asks) = &self.asks else {
            return;
        };
        let mut pings = lock(&self.shared.0);
        if pings.ask(window, Instant::now()) && asks.send(window).is_err() {
            pings.done(true);
        }
    }

    /// True while `window` owes a ping past `check_ms`, or left its last one unanswered.
    pub(super) fn busy(&self, window: u64) -> bool {
        lock(&self.shared.0).busy(window, Instant::now(), self.wait)
    }

    /// Waits up to `limit` for the ping out to end.
    #[cfg(test)]
    pub(super) fn settle(&self, limit: Duration) {
        let (pings, ended) = &*self.shared;
        let _ = ended.wait_timeout_while(lock(pings), limit, |p| p.out());
    }
}

/// True when `window` answered a `WM_NULL` within `read_ms`; a hung or closed window says no at once.
#[allow(
    unsafe_code,
    reason = "Sends a message with no data, for a bounded time."
)]
fn answers(window: u64, read_ms: u32) -> bool {
    let flags = SMTO_ABORTIFHUNG | SMTO_ERRORONEXIT;
    let (w, l) = (WPARAM(0), LPARAM(0));
    // SAFETY: the message carries no data, and the wait ends after `read_ms` at most.
    let sent = unsafe { SendMessageTimeoutW(handle(window), WM_NULL, w, l, flags, read_ms, None) };
    // SAFETY: reads this thread's last error, set by the call just made.
    answered(sent.0 != 0, unsafe { GetLastError() })
}

/// Whether a ping that was `sent`, or failed with `error`, counts as answered: one that Windows blocks (UIPI) does,
/// so the turn's own failure tells instead.
pub(super) fn answered(sent: bool, error: WIN32_ERROR) -> bool {
    sent || error == ERROR_ACCESS_DENIED
}
