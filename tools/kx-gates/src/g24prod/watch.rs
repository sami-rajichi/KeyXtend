//! A watch over one snip, on a thread of its own: every poll it reads which window is in front and which windows of
//! the copy show, kept out of captures or not, and counts what `judge::faults` finds. It records counts, never pixels.
#![cfg(windows)]

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;

use kx_module_api::ports::SnipKeyboard;
use kx_platform_windows::window::foreground;
use serde::Serialize;

use super::judge::{self, Kind};
use super::snip::Snip;
use super::wins::{self, Titles};
use crate::win::{self, sleep_ms};

/// What a watch saw: its reads, those taken during a capture, those with target-window not in front, each fault once,
/// and whether the bubble showed.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Watched {
    pub reads: u32,
    pub judged: u32,
    pub away: u32,
    pub faults: Vec<&'static str>,
    pub bubble: bool,
}

impl Watched {
    /// True when it judged a capture, target-window stayed in front, nothing faulted, and with the keyboard `way` a
    /// bubble, the bubble showed.
    pub fn ok(&self, way: SnipKeyboard) -> bool {
        let bubble = way != SnipKeyboard::Bubble || self.bubble;
        self.judged > 0 && self.away == 0 && self.faults.is_empty() && bubble
    }

    /// Adds one read: whether target-window was in front, and the faults and kinds of the copy's windows.
    fn add(&mut self, front: bool, way: SnipKeyboard, seen: &[judge::Seen]) {
        self.reads += 1;
        self.away += u32::from(!front);
        let capture = seen
            .iter()
            .any(|s| matches!(s.kind, Kind::Frozen | Kind::Capture));
        self.judged += u32::from(capture);
        self.bubble |= capture && seen.iter().any(|s| s.kind == Kind::Bubble);
        for f in judge::faults(way, seen) {
            if !self.faults.contains(&f) {
                self.faults.push(f);
            }
        }
    }
}

/// A running watch.
pub struct Watch {
    stop: Arc<AtomicBool>,
    thread: JoinHandle<Watched>,
}

/// What the watch thread needs, all of it sendable.
struct Spec {
    pid: u32,
    titles: Titles,
    target: isize,
    way: SnipKeyboard,
    poll_ms: u64,
}

impl Watch {
    /// Starts watching the copy and target-window of `s`.
    pub fn start(s: &Snip) -> Self {
        let spec = Spec {
            pid: s.pid,
            titles: s.titles.clone(),
            target: win::key(s.app.hwnd),
            way: s.way,
            poll_ms: s.poll(),
        };
        let stop = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&stop);
        let thread = std::thread::spawn(move || run(&spec, &flag));
        Self { stop, thread }
    }

    /// Stops the watch and gives what it saw; a watch that failed saw nothing.
    pub fn stop(self) -> Watched {
        self.stop.store(true, Ordering::Relaxed);
        self.thread.join().unwrap_or_default()
    }
}

/// Reads every `poll_ms` until `stop` is set.
fn run(spec: &Spec, stop: &AtomicBool) -> Watched {
    let mut w = Watched::default();
    while !stop.load(Ordering::Relaxed) {
        let front = win::key(foreground()) == spec.target;
        w.add(front, spec.way, &wins::seen(spec.pid, &spec.titles));
        sleep_ms(spec.poll_ms);
    }
    w
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::g24prod::judge::Seen;

    #[test]
    fn a_watch_passes_only_with_a_capture_judged_the_front_kept_and_no_fault() {
        let (cap, kb) = (
            Seen {
                kind: Kind::Capture,
                shielded: true,
            },
            Seen {
                kind: Kind::Keyboard,
                shielded: true,
            },
        );
        let bubble = Seen {
            kind: Kind::Bubble,
            shielded: true,
        };
        let mut w = Watched::default();
        w.add(true, SnipKeyboard::Hidden, &[]);
        assert!(!w.ok(SnipKeyboard::Hidden), "nothing judged");
        w.add(true, SnipKeyboard::Hidden, &[cap]);
        assert!(w.ok(SnipKeyboard::Hidden));
        assert!(!w.ok(SnipKeyboard::Bubble), "no bubble seen");
        w.add(true, SnipKeyboard::Bubble, &[cap, bubble]);
        assert!(w.ok(SnipKeyboard::Bubble));
        w.add(false, SnipKeyboard::Bubble, &[cap, kb]);
        assert_eq!(
            (w.reads, w.judged, w.away, w.faults.clone()),
            (4, 3, 1, vec![judge::KEYBOARD])
        );
        assert!(!w.ok(SnipKeyboard::Bubble));
    }
}
