//! Between the app and the UI: the Qt thread's waker, a second copy's commands, Try again, and the one quit path.
//! Quit is the last request: the hub drops its sender after it, so the app's drain loop ends on its own.

use super::{Hub, Request, Wake, Waker};
use kx_platform::ui::Command;
use std::sync::Arc;

impl Hub {
    /// Registers the Qt thread's waker and hands it the `Show` and `End` wakes that came first.
    /// A second waker replaces the first, which then hears nothing; that is a bug, so it warns.
    pub fn set_waker(&self, waker: Waker) {
        let (pending, replaced) = {
            let mut live = self.lock();
            let replaced = live.waker.replace(Arc::new(waker)).is_some();
            (std::mem::take(&mut live.pending), replaced)
        };
        if replaced {
            tracing::warn!("a second Ui object took the hub's wakes; make one per app");
        }
        for wake in pending {
            self.wake(wake);
        }
    }

    /// A second copy's `command`: Show goes to the UI, Quit to the app, as the user's Quit does.
    pub fn command(&self, command: Command) {
        match command {
            Command::Show => self.wake_or_wait(Wake::Show),
            Command::Quit => {
                let _ = self.quit();
            }
        }
    }

    /// The app stopped the modules after Quit: the UI ends. It waits for the waker if none is set yet.
    pub fn end_ui(&self) {
        self.wake_or_wait(Wake::End);
    }

    /// Asks the app to quit and ends its queue; false when it was asked already or no longer listens.
    pub fn quit(&self) -> bool {
        let requests = self.lock().requests.take();
        requests.is_some_and(|r| r.send(Request::Quit).is_ok())
    }

    /// Try again on shown notice `id`: it closes, and the app restarts its module; false when it offers none.
    pub fn retry(&self, id: u64) -> bool {
        let (module, requests) = {
            let mut live = self.lock();
            (live.notices.close(id), live.requests.clone())
        };
        match (module, requests) {
            (Some(m), Some(r)) => r.send(Request::Retry(m)).is_ok(),
            _ => false,
        }
    }

    /// Wakes the UI with `what`, or keeps it until the waker is set.
    fn wake_or_wait(&self, what: Wake) {
        let waiting = {
            let mut live = self.lock();
            let none = live.waker.is_none();
            if none {
                live.pending.push(what);
            }
            none
        };
        if !waiting {
            self.wake(what);
        }
    }
}
