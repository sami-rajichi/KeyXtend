//! One wake that waits for QML's read, so a run of changes costs one wake per read.

use std::sync::atomic::{AtomicBool, Ordering};

/// Whether a wake waits for QML to read the state it is about.
#[derive(Default)]
pub(super) struct Unread(AtomicBool);

impl Unread {
    /// True when a change should wake, as no wake waits yet; from now on one does.
    pub fn arm(&self) -> bool {
        !self.0.swap(true, Ordering::SeqCst)
    }

    /// QML reads the state, so the next change wakes again.
    pub fn read(&self) {
        self.0.store(false, Ordering::SeqCst);
    }

    /// True while a wake waits for QML's read.
    pub fn waits(&self) -> bool {
        self.0.load(Ordering::SeqCst)
    }
}
