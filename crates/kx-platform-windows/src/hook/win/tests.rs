//! Tests of the posts to the hook thread, sent to this test's own thread; none installs a hook or starts a run.

use windows::Win32::UI::WindowsAndMessaging::{PM_REMOVE, PeekMessageW};

use super::*;

/// True when `msg` waited in this thread's queue; it is taken out.
#[allow(unsafe_code, reason = "Peeks into this thread's own queue.")]
fn queued(msg: u32) -> bool {
    let mut got = MSG::default();
    // SAFETY: a peek of this thread's own queue into a local it fills.
    unsafe { PeekMessageW(&raw mut got, None, msg, msg, PM_REMOVE) }.as_bool()
}

#[test]
fn a_post_reaches_the_runs_thread_only_while_live_holds_its_id() {
    let tid = own_queue();
    let live = AtomicU32::new(NONE);
    assert!(!post_live(&live, tid, WAKE), "the thread ended");
    assert!(!queued(WAKE));
    live.store(tid, Ordering::SeqCst);
    assert!(post_live(&live, tid, WAKE));
    assert!(queued(WAKE));
    live.store(NONE, Ordering::SeqCst);
    assert!(!post_live(&live, NONE, WAKE), "no id yet");
}

#[test]
fn asking_a_run_to_stop_wakes_its_thread_until_it_ends() {
    let ctl = Ctl::default();
    let live = AtomicU32::new(NONE);
    let (done, ended) = mpsc::channel();
    ask_stop(&ctl, &live, &ended);
    assert!(ctl.stop.load(Ordering::SeqCst));
    let tid = own_queue();
    ctl.tid.store(tid, Ordering::SeqCst);
    live.store(tid, Ordering::SeqCst);
    ask_stop(&ctl, &live, &ended);
    assert!(
        queued(WM_QUIT),
        "it answered late, so it may wait for a message"
    );
    drop(done);
    ask_stop(&ctl, &live, &ended);
    assert!(!queued(WM_QUIT), "it ended");
}
