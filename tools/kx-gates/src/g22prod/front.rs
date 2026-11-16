//! Who holds the foreground while a clip is dictated: a thread samples the window in front until it finishes, keeping
//! each window's key once. A moment with no window in front is skipped, as Windows shows one while focus moves.
#![cfg(windows)]

use std::collections::BTreeSet;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use kx_platform_windows::window::foreground;

use crate::win;

/// What the samples saw: how many, and each window that was in front.
#[derive(Debug, Default)]
pub(super) struct Fronts {
    pub(super) samples: u32,
    pub(super) seen: BTreeSet<isize>,
}

/// The samples' count against the target and the bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct FrontCount {
    /// Samples taken.
    pub(super) samples: u32,
    /// Windows other than the target that were in front.
    pub(super) away: usize,
    /// The bar was in front.
    pub(super) bar: bool,
}

impl Fronts {
    /// The count against `target`'s key and `bar`'s.
    pub(super) fn count(&self, target: isize, bar: isize) -> FrontCount {
        let away = self.seen.iter().filter(|&&k| k != target).count();
        FrontCount {
            samples: self.samples,
            away,
            bar: self.seen.contains(&bar),
        }
    }
}

impl FrontCount {
    /// The target kept the foreground: it was sampled, and no other window was ever in front.
    pub(super) fn kept(&self) -> bool {
        self.samples > 0 && self.away == 0
    }
}

/// The sampling thread.
pub(super) struct Watch {
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<Fronts>>,
}

impl Watch {
    /// Samples the window in front every `poll` until `finish`.
    pub(super) fn start(poll: Duration) -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let halt = Arc::clone(&stop);
        let thread = thread::spawn(move || {
            let mut f = Fronts::default();
            while !halt.load(Ordering::Acquire) {
                let key = win::key(foreground());
                if key != 0 {
                    f.samples += 1;
                    f.seen.insert(key);
                }
                thread::sleep(poll);
            }
            f
        });
        Self {
            stop,
            thread: Some(thread),
        }
    }

    /// Stops sampling: what it saw.
    pub(super) fn finish(mut self) -> Fronts {
        self.halt().unwrap_or_default()
    }

    /// Stops the thread and takes what it saw.
    fn halt(&mut self) -> Option<Fronts> {
        self.stop.store(true, Ordering::Release);
        self.thread.take().and_then(|t| t.join().ok())
    }
}

impl Drop for Watch {
    fn drop(&mut self) {
        let _ = self.halt();
    }
}
