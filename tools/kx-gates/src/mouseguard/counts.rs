//! What the guard counted, as numbers only: they show in each run's result that its hook was live.

use std::sync::atomic::{AtomicU64, Ordering};

/// The hand's events a guard dropped and kx-gates' own events it passed; never a position, button or key.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Counts {
    /// Events dropped that were not kx-gates' own: the hand's, or other programs' injected input.
    pub dropped: u64,
    /// kx-gates' own injected events passed.
    pub passed: u64,
}

/// A run's counts as its hooks keep them: the hook callback adds with lock-free atomics, and the gate reads them live.
#[derive(Debug, Default)]
pub struct Tally {
    dropped: AtomicU64,
    passed: AtomicU64,
}

impl Tally {
    /// One of the hand's events dropped.
    pub fn drop_one(&self) {
        self.dropped.fetch_add(1, Ordering::Relaxed);
    }

    /// One own event passed.
    pub fn pass_one(&self) {
        self.passed.fetch_add(1, Ordering::Relaxed);
    }

    /// Own events passed so far.
    pub fn passed(&self) -> u64 {
        self.passed.load(Ordering::Relaxed)
    }

    /// Both counts now.
    pub fn counts(&self) -> Counts {
        Counts {
            dropped: self.dropped.load(Ordering::Relaxed),
            passed: self.passed(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;

    #[test]
    fn a_tally_counts_on_the_hook_thread_and_the_gate_reads_it_live() {
        let tally = Arc::new(Tally::default());
        let hook = Arc::clone(&tally);
        std::thread::spawn(move || {
            hook.pass_one();
            hook.drop_one();
            hook.pass_one();
        })
        .join()
        .expect("the hook thread ends");
        assert_eq!(tally.passed(), 2);
        let counts = tally.counts();
        assert_eq!((counts.dropped, counts.passed), (1, 2));
    }
}
