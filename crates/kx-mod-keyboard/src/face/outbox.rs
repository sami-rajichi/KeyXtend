//! One sender at a time for the face's bus events: a request made while another thread sends is left to that thread,
//! which sends again before it stops, so the bus hears the newest state last and no caller waits.

use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::SeqCst;

/// Requests to send, taken by one thread at a time.
#[derive(Default)]
pub(super) struct Outbox {
    /// A thread asks for a send.
    due: AtomicBool,
    /// One thread is sending; the others leave their requests to it.
    sending: AtomicBool,
}

/// Clears the `sending` flag when the sending thread stops, even by a panic, so later sends still go out.
struct Sending<'a>(&'a AtomicBool);

impl Drop for Sending<'_> {
    fn drop(&mut self) {
        self.0.store(false, SeqCst);
    }
}

impl Outbox {
    /// Asks for a send: runs `send` once for each request made before it stops, unless another thread is sending,
    /// which then takes this request too. `send` may ask again, as a bus handler may.
    pub(super) fn post(&self, send: impl Fn()) {
        self.due.store(true, SeqCst);
        loop {
            if self
                .sending
                .compare_exchange(false, true, SeqCst, SeqCst)
                .is_err()
            {
                return;
            }
            let sending = Sending(&self.sending);
            while self.due.swap(false, SeqCst) {
                send();
            }
            drop(sending);
            // A request made after the last check but before the stop found the sender busy, so it is ours.
            if !self.due.load(SeqCst) {
                return;
            }
        }
    }
}

#[cfg(test)]
mod tests;
