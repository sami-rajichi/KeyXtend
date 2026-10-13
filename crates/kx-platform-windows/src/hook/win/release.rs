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
    use super::*;

    #[test]
    fn release_now_without_a_live_hook_returns_at_once() {
        let start = Instant::now();
        release_now();
        assert!(start.elapsed() < RELEASE_WAIT);
    }

    #[test]
    fn only_an_ask_moves_the_answer_and_a_start_voids_old_asks() {
        forget_asks();
        let before = DONE.load(Ordering::SeqCst);
        assert_eq!(let_go(), None);
        assert_eq!(DONE.load(Ordering::SeqCst), before, "nobody asked");
        let ticket = ASKED.fetch_add(1, Ordering::SeqCst) + 1;
        let _ = let_go();
        assert!(DONE.load(Ordering::SeqCst) >= ticket, "the ask is answered");
        ASKED.fetch_add(1, Ordering::SeqCst);
        forget_asks();
        assert_eq!(DONE.load(Ordering::SeqCst), ASKED.load(Ordering::SeqCst));
    }
}
