//! G9's drops as kx-gates makes them: another thread turns topmost off on the keyboard, then on its shadow, while the
//! windows' own thread answers their messages.

use std::cell::Cell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, PoisonError};
use std::time::Instant;

use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    CallWindowProcW, DispatchMessageW, GWLP_WNDPROC, HWND_NOTOPMOST, InSendMessage, MSG, PM_REMOVE,
    PeekMessageW, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER, SetWindowLongPtrW,
    SetWindowPos, TranslateMessage, WINDOWPOS, WM_WINDOWPOSCHANGING,
};

use super::super::guard::{Proc, next};
use super::super::top::lifting;
use super::super::win::guard_these;
use super::z_order::{Z_ORDER, above, listed, put_after, topmost};
use super::{Hidden, WAIT};

/// The most messages one `pump` answers, so a message loop fails a test instead of hanging it.
const CAP: usize = 1000;

thread_local! {
    /// The guard procedure `stuck` hands messages to.
    static STUCK_BEHIND: Cell<isize> = const { Cell::new(0) };
    /// True when `stuck` keeps its window where it is on every lift, not only inside another thread's call.
    static STUCK_ALWAYS: Cell<bool> = const { Cell::new(false) };
}

/// A subclass in front of the guard that keeps its window where it is when the guard lifts it while another thread's
/// call is still running, as G9 suggests the installed shadow does: a lift made there does not land.
unsafe extern "system" fn stuck(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    // SAFETY: a plain query of this thread's state.
    let held = STUCK_ALWAYS.get() || unsafe { InSendMessage() }.as_bool();
    if msg == WM_WINDOWPOSCHANGING && lifting() && held {
        let pos = std::ptr::with_exposed_provenance_mut::<WINDOWPOS>(lparam.0.cast_unsigned());
        // SAFETY: for this message `lparam` points to a live WINDOWPOS.
        if let Some(pos) = unsafe { pos.as_mut() } {
            pos.flags |= SWP_NOZORDER;
        }
    }
    let behind = STUCK_BEHIND.get().cast_unsigned();
    // SAFETY: hands every message to the guard this subclass replaced, as a subclass must.
    unsafe { CallWindowProcW(next(behind), hwnd, msg, wparam, lparam) }
}

/// Puts `stuck` in front of the guard on `window`.
fn stall(window: &Hidden) {
    let ours: Proc = stuck;
    // SAFETY: puts the test subclass in front of the guard on the test's own window; it hands every message on.
    STUCK_BEHIND
        .set(unsafe { SetWindowLongPtrW(window.0, GWLP_WNDPROC, (ours as usize).cast_signed()) });
}

/// Answers this thread's waiting messages, at most `CAP`; says how many.
fn pump() -> usize {
    let mut msg = MSG::default();
    let mut answered = 0;
    // SAFETY: the test thread's own messages, taken and handed on one at a time.
    while answered < CAP && unsafe { PeekMessageW(&raw mut msg, None, 0, 0, PM_REMOVE) }.as_bool() {
        // SAFETY: hands on a message this thread just took.
        unsafe {
            let _ = TranslateMessage(&raw const msg);
            DispatchMessageW(&raw const msg);
        }
        answered += 1;
    }
    answered
}

/// Turns topmost off on `windows`, one after another, from another thread, while this thread answers their messages.
/// A thread still busy after `WAIT` fails the test and is left behind, as joining it could hang.
fn drop_elsewhere(windows: &[HWND]) {
    let raw: Vec<usize> = windows.iter().map(|w| w.0.addr()).collect();
    let done = Arc::new(AtomicBool::new(false));
    let flag = Arc::clone(&done);
    let thread = std::thread::spawn(move || {
        let flags = SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE;
        for w in raw {
            let w = HWND(std::ptr::without_provenance_mut(w));
            // SAFETY: a z-order change on the test's own hidden window, as another program could make it.
            let _ = unsafe { SetWindowPos(w, Some(HWND_NOTOPMOST), 0, 0, 0, 0, flags) };
        }
        flag.store(true, Ordering::SeqCst);
    });
    let start = Instant::now();
    while !done.load(Ordering::SeqCst) && start.elapsed() < WAIT {
        pump();
    }
    assert!(
        done.load(Ordering::SeqCst),
        "the other thread's drops ended"
    );
    thread.join().unwrap();
    pump();
}

/// G9's own drops, keyboard then shadow, from another thread: the guard's repair inside each call holds.
#[test]
fn a_keyboard_and_its_shadow_dropped_in_turn_from_another_thread_both_come_back() {
    let _z = Z_ORDER.lock().unwrap_or_else(PoisonError::into_inner);
    let shadow = Hidden::tip("KeyXtend test: shadow dropped from afar", None);
    let kb = Hidden::topmost("KeyXtend test: keyboard dropped from afar", Some(shadow.0));
    guard_these(listed(&[&kb, &shadow]), None, None).unwrap();
    drop_elsewhere(&[kb.0, shadow.0]);
    assert!(topmost(&kb), "the keyboard stays");
    assert!(topmost(&shadow), "and its shadow");
    assert!(above(&kb, &shadow), "above its shadow");
}

/// A model of one possible G9 cause: the guard's lift of the shadow does not land while the other program's drop is
/// still running, so the guard checks again after the window has answered the move.
#[test]
fn a_lift_that_does_not_land_during_another_threads_drop_is_made_again_after_it() {
    let _z = Z_ORDER.lock().unwrap_or_else(PoisonError::into_inner);
    let shadow = Hidden::tip("KeyXtend test: stuck shadow", None);
    let kb = Hidden::topmost("KeyXtend test: stuck shadow's keyboard", Some(shadow.0));
    guard_these(listed(&[&kb, &shadow]), None, None).unwrap();
    stall(&shadow);
    drop_elsewhere(&[kb.0, shadow.0]);
    assert!(topmost(&kb), "the keyboard stays");
    assert!(topmost(&shadow), "and its shadow comes back after the drop");
    assert!(above(&kb, &shadow), "above its shadow");
}

/// A lift that never lands is checked again a few times, never over and over: a posted check posts none.
#[test]
fn the_checks_end_when_a_lift_never_lands() {
    let _z = Z_ORDER.lock().unwrap_or_else(PoisonError::into_inner);
    let shadow = Hidden::tip("KeyXtend test: shadow never lifted", None);
    let kb = Hidden::topmost("KeyXtend test: keyboard over it", Some(shadow.0));
    guard_these(listed(&[&kb, &shadow]), None, None).unwrap();
    STUCK_ALWAYS.set(true);
    stall(&shadow);
    put_after(&kb, HWND_NOTOPMOST);
    let answered = pump();
    STUCK_ALWAYS.set(false);
    assert!(answered < CAP, "the checks ended after {answered} messages");
    assert!(!topmost(&shadow), "the shadow's lift never landed");
}
