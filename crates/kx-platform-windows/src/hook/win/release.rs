//! `release_now` for the app's panic hook: the hook thread lets go of what our input holds and keeps its hooks and its
//! run, so a panic the kernel catches never ends mouse assist.

use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use windows::Win32::UI::WindowsAndMessaging::WM_APP;

use super::os::{post, this_thread};
use super::{Host, LIVE, NONE, STARTING, with_host};

/// Asks the hook thread to let go.
pub(super) const LET_GO: u32 = WM_APP + 2;
/// How long `release_now` waits for the thread to let go.
const RELEASE_WAIT: Duration = Duration::from_millis(200);
/// How often `release_now` looks while it waits.
const RELEASE_POLL: Duration = Duration::from_millis(1);

/// How many let-gos `release_now` asked for, and up to which ask the hook thread has let go.
static ASKED: AtomicU64 = AtomicU64::new(0);
static DONE: AtomicU64 = AtomicU64::new(0);

/// Asks a live hook thread to let go of all our input holds, keeping its hooks and its run, and waits for that at most
/// `RELEASE_WAIT`. It never panics, and returns at once with no live hook, while one starts, or on the hook thread
/// itself, where the callback's own recovery lets go.
pub fn release_now() {
    let tid = LIVE.load(Ordering::SeqCst);
    if tid == NONE || tid == STARTING || tid == this_thread() {
        return;
    }
    let ticket = ASKED.fetch_add(1, Ordering::SeqCst) + 1;
    if !post(tid, LET_GO) {
        return;
    }
    let start = Instant::now();
    while DONE.load(Ordering::SeqCst) < ticket
        && LIVE.load(Ordering::SeqCst) == tid
        && start.elapsed() < RELEASE_WAIT
    {
        thread::sleep(RELEASE_POLL);
    }
}

/// On the hook thread: the let-go `release_now` asked for, then its answer; a `LET_GO` nobody asked for does nothing.
pub(super) fn let_go() -> Option<()> {
    let asked = ASKED.load(Ordering::SeqCst);
    if asked == DONE.load(Ordering::SeqCst) {
        return None;
    }
    let done = with_host(Host::release);
    DONE.store(asked, Ordering::SeqCst);
    done
}

/// Voids every ask no run answered, as a run starts, so another program's `LET_GO` never acts on its own.
pub(super) fn forget_asks() {
    DONE.store(ASKED.load(Ordering::SeqCst), Ordering::SeqCst);
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::AtomicUsize;

    use kx_module_api::Mono;
    use kx_module_api::ports::{HookConfig, PointerAct, PointerEngine, PointerEvent};

    use super::super::os::WinOs;
    use super::super::{HOST, drop_host};
    use super::*;
    use crate::QpcClock;
    use crate::hook::Source;
    use crate::hook::stats::Stats;

    /// The module's default checks; this host never installs its hooks.
    const CFG: HookConfig = HookConfig {
        health_ms: 1_000,
        health_slack_ms: 500,
        echo_ms: 250,
    };

    /// An engine that never acts and counts its releases.
    struct Count(Arc<AtomicUsize>);

    impl PointerEngine for Count {
        fn on(&mut self, _: PointerEvent, _: Mono, _: &mut Vec<PointerAct>) -> bool {
            false
        }

        fn settle(&mut self, _: Mono, _: &mut Vec<PointerAct>) {}

        fn deadline(&self) -> Option<Mono> {
            None
        }

        fn has_notes(&self) -> bool {
            false
        }

        fn release(&mut self, _: &mut Vec<PointerAct>) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }

        fn max_acts(&self) -> usize {
            1
        }
    }

    /// Puts a host with a counting engine on this thread, as the hook thread holds one; it sends nothing.
    fn seed() -> Arc<AtomicUsize> {
        let released = Arc::new(AtomicUsize::new(0));
        let os = WinOs::new(0, QpcClock::new().unwrap(), Arc::default());
        let engine = Box::new(Count(Arc::clone(&released)));
        let host = Host::new(os, engine, Source::Physical, &CFG, Arc::new(Stats::new()));
        HOST.with(|h| *h.borrow_mut() = Some(host));
        released
    }

    #[test]
    fn release_now_without_a_live_hook_returns_at_once() {
        let start = Instant::now();
        release_now();
        assert!(start.elapsed() < RELEASE_WAIT);
    }

    #[test]
    fn a_let_go_nobody_asked_for_runs_no_release_and_an_ask_runs_one() {
        let released = seed();
        forget_asks();
        assert_eq!(let_go(), None);
        assert_eq!(released.load(Ordering::SeqCst), 0, "nobody asked");
        let ticket = ASKED.fetch_add(1, Ordering::SeqCst) + 1;
        assert_eq!(let_go(), Some(()));
        assert_eq!(released.load(Ordering::SeqCst), 1, "the ask's one release");
        assert!(DONE.load(Ordering::SeqCst) >= ticket, "and its answer");
        assert_eq!(let_go(), None, "answered asks run no more");
        ASKED.fetch_add(1, Ordering::SeqCst);
        forget_asks();
        assert_eq!(let_go(), None, "a start voids unanswered asks");
        assert_eq!(released.load(Ordering::SeqCst), 1);
        drop_host();
    }
}
