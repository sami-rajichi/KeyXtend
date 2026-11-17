//! When the typed text is done: it showed in the box, then stayed the same for the settle wait. It keeps the last read
//! in wiped memory, as it is the user's dictated text.
#![cfg(windows)]

use std::time::{Duration, Instant};

use kx_module_api::Zeroizing;

/// The box's text as read after Stop, until it settles.
pub(super) struct Settle {
    wait: Duration,
    last: Zeroizing<String>,
    since: Option<Instant>,
    first: Option<Instant>,
}

impl Settle {
    /// Text counts as settled once it stays the same for `wait`.
    pub(super) fn new(wait: Duration) -> Self {
        Self {
            wait,
            last: Zeroizing::new(String::new()),
            since: None,
            first: None,
        }
    }

    /// Feeds the box's text read at `at`: true once it is not empty and has stayed the same for the wait.
    pub(super) fn feed(&mut self, text: &str, at: Instant) -> bool {
        if text != self.last.as_str() {
            self.last = Zeroizing::new(text.to_owned());
            self.since = Some(at);
            if !text.is_empty() {
                self.first.get_or_insert(at);
            }
            return false;
        }
        let held = self
            .since
            .is_some_and(|s| at.saturating_duration_since(s) >= self.wait);
        !text.is_empty() && held
    }

    /// When text first showed.
    pub(super) fn first(&self) -> Option<Instant> {
        self.first
    }
}
