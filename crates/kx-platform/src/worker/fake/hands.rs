//! The tests' clock, moved by hand.

use kx_module_api::{Clock, Mono};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

/// A clock that moves only when the test says; clones share it.
#[derive(Clone, Default)]
pub struct Hands(Arc<AtomicU64>);

impl Hands {
    /// Moves the time on by `by`.
    pub fn advance(&self, by: Duration) {
        let us = u64::try_from(by.as_micros()).unwrap_or(u64::MAX);
        self.0.fetch_add(us, Ordering::SeqCst);
    }
}

impl Clock for Hands {
    fn now(&self) -> Mono {
        Mono::from_us(self.0.load(Ordering::SeqCst))
    }
}
