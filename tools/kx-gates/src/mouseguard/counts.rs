//! What the guard counted, as numbers only: they show in each run's result that its hook was live.

/// The hand's events a guard dropped and kx-gates' own events it passed; never a position, button or key.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Counts {
    /// Physical events dropped.
    pub dropped: u32,
    /// kx-gates' own injected events passed.
    pub passed: u32,
}

impl Counts {
    /// One of the hand's events dropped.
    pub const DROP: Self = Self {
        dropped: 1,
        passed: 0,
    };
    /// One own event passed.
    pub const PASS: Self = Self {
        dropped: 0,
        passed: 1,
    };

    /// Adds what another hook counted.
    pub fn add(&mut self, more: Self) {
        self.dropped = self.dropped.saturating_add(more.dropped);
        self.passed = self.passed.saturating_add(more.passed);
    }
}
