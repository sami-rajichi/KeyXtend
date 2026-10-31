//! The hidden window: which messages end its loop, when a stop waits, and that only `stop` closes it.

use std::sync::{Mutex, PoisonError};
use std::time::{Duration, Instant};

use super::*;
use windows::Win32::UI::WindowsAndMessaging::{
    FindWindowExW, IsWindow, PostMessageW, SendMessageW, WM_APP, WM_CLOSE,
};

#[test]
fn only_stop_or_an_error_ends_the_loop() {
    assert!(ends(0, true), "stop's WM_QUIT");
    assert!(!ends(0, false), "a WM_QUIT another program posts");
    assert!(
        ends(1, true),
        "any message once stopping, so a refused quit still ends it"
    );
    assert!(!ends(1, false), "any other message");
    assert!(ends(-1, false) && ends(-1, true), "an error");
}

#[test]
fn stop_waits_only_for_a_quit_it_posted_and_never_from_the_window_thread() {
    assert!(waits(true, false), "a posted quit ends the loop");
    assert!(!waits(false, false), "a full queue refused the quit");
    assert!(!waits(true, true) && !waits(false, true), "its own thread");
}

#[test]
#[allow(
    unsafe_code,
    reason = "Finds and messages the test's own hidden window."
)]
fn another_program_cannot_close_a_hidden_window() {
    let class = crate::wide(&format!("KeyXtendTest.{}.close", std::process::id()));
    let hidden = Hidden::start(class.clone(), Kind::TopLevel, Some(answer), || ()).unwrap();
    // SAFETY: the class name ends with a null; WM_CLOSE carries no pointers.
    let open = unsafe {
        let window = FindWindowExW(None, None, PCWSTR(class.as_ptr()), PCWSTR::null()).unwrap();
        SendMessageW(window, WM_CLOSE, None, None);
        IsWindow(Some(window)).as_bool()
    };
    hidden.stop();
    assert!(open, "the window ignored WM_CLOSE");
}

/// Holds a test window's thread inside its procedure while a test keeps it locked.
static GATE: Mutex<()> = Mutex::new(());
/// Long enough for an idle window's thread to end.
const IDLE_WAIT: Duration = Duration::from_secs(5);
/// Too short for a thread the gate holds.
const BUSY_WAIT: Duration = Duration::from_millis(50);

/// Hears that a test window's thread reached the gate.
static INSIDE: Mutex<Option<mpsc::Sender<()>>> = Mutex::new(None);

/// On `WM_APP` says it is inside, then waits at the gate; else answers as our windows do.
#[allow(unsafe_code, reason = "The procedure of a test window.")]
unsafe extern "system" fn gated(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if msg == WM_APP {
        let inside = INSIDE.lock().unwrap_or_else(PoisonError::into_inner).take();
        if let Some(tx) = inside {
            let _ = tx.send(());
        }
        drop(GATE.lock());
        return LRESULT::default();
    }
    // SAFETY: our own window's message, with its own values.
    unsafe { answer(hwnd, msg, wparam, lparam) }
}

#[test]
#[allow(
    unsafe_code,
    reason = "Finds and messages the test's own hidden window."
)]
fn a_bounded_stop_waits_for_an_idle_thread_and_leaves_a_held_one() {
    let class = |what: &str| crate::wide(&format!("KeyXtendTest.{}.{what}", std::process::id()));
    let idle = Hidden::start(class("idle"), Kind::TopLevel, Some(gated), || ()).unwrap();
    assert!(idle.stop_within(IDLE_WAIT), "an idle thread ends in time");
    let gate = GATE.lock().unwrap_or_else(PoisonError::into_inner);
    let (tx, inside) = mpsc::channel();
    *INSIDE.lock().unwrap_or_else(PoisonError::into_inner) = Some(tx);
    let held = class("held");
    let busy = Hidden::start(held.clone(), Kind::TopLevel, Some(gated), || ()).unwrap();
    // SAFETY: the class name ends with a null; WM_APP carries no pointers.
    unsafe {
        let window = FindWindowExW(None, None, PCWSTR(held.as_ptr()), PCWSTR::null()).unwrap();
        PostMessageW(Some(window), WM_APP, WPARAM(0), LPARAM(0)).unwrap();
    }
    inside
        .recv_timeout(IDLE_WAIT)
        .expect("the thread reached the gate");
    let started = Instant::now();
    let ended = busy.stop_within(BUSY_WAIT);
    let waited = started.elapsed();
    drop(gate);
    assert!(!ended, "the held thread is left to end alone");
    assert!(waited < IDLE_WAIT, "the stop came back after its wait");
}
