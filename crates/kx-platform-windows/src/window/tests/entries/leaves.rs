//! When the pointer is in the watched window: a pointer there when marked, leaves under capture or not, each move's
//! ask for the next leave, and a lost capture. The messages come from the test; none moves the pointer.

use std::sync::PoisonError;

use kx_module_api::ports::Point;
use windows::Win32::UI::Controls::WM_MOUSELEAVE;
use windows::Win32::UI::WindowsAndMessaging::{
    MSG, PM_REMOVE, PeekMessageW, SendMessageW, WM_CAPTURECHANGED,
};

use super::super::super::entry::{kept, see};
use super::super::super::watch::Seen;
use super::super::Hidden;
use super::{ENTRIES, move_to, spot_on_main, watched_at};

/// Tells `window` it lost the capture, as Windows does; the hidden window is never under the pointer.
#[allow(unsafe_code, reason = "Sends a message to the test's own window.")]
fn lose_capture(window: &Hidden) {
    // SAFETY: the message carries no pointer.
    unsafe { SendMessageW(window.0, WM_CAPTURECHANGED, None, None) };
}

/// How many leaves Windows queued for `window`, taken from the queue.
#[allow(unsafe_code, reason = "Reads the test thread's own queue.")]
fn leaves(window: &Hidden) -> usize {
    let mut msg = MSG::default();
    let mut count = 0;
    // SAFETY: takes the test window's own queued leaves into a local.
    while unsafe {
        PeekMessageW(
            &raw mut msg,
            Some(window.0),
            WM_MOUSELEAVE,
            WM_MOUSELEAVE,
            PM_REMOVE,
        )
    }
    .as_bool()
    {
        count += 1;
    }
    count
}

#[test]
fn a_pointer_already_on_the_window_when_marked_is_no_entry() {
    let _e = ENTRIES.lock().unwrap_or_else(PoisonError::into_inner);
    let kb = spot_on_main();
    let window = watched_at("KeyXtend test: under the pointer", kb, true);
    let was = kept();
    move_to(&window, Point { x: 5, y: 6 });
    assert_eq!(kept(), was, "no entry yet");
    see(window.0, Seen::Leave { captured: false });
    move_to(&window, Point { x: 7, y: 8 });
    let entry = Point {
        x: kb.x + 7,
        y: kb.y + 8,
    };
    assert_eq!(kept().map(|k| k.entry), Some(entry));
}

#[test]
fn a_leave_under_capture_keeps_the_entry_and_any_other_ends_it() {
    let _e = ENTRIES.lock().unwrap_or_else(PoisonError::into_inner);
    let kb = spot_on_main();
    let window = watched_at("KeyXtend test: captured", kb, false);
    let screen = |x, y| Point {
        x: kb.x + x,
        y: kb.y + y,
    };
    move_to(&window, Point { x: 5, y: 6 });
    see(window.0, Seen::Leave { captured: true });
    move_to(&window, Point { x: 7, y: 8 });
    assert_eq!(kept().map(|k| k.entry), Some(screen(5, 6)), "still inside");
    see(window.0, Seen::Leave { captured: false });
    move_to(&window, Point { x: 9, y: 10 });
    assert_eq!(kept().map(|k| k.entry), Some(screen(9, 10)), "a new entry");
}

/// Windows answers a leave request for a window the pointer is not on at once, so each request queues one leave.
#[test]
fn each_move_inside_asks_for_the_leave_again_and_a_leave_under_capture_once() {
    let _e = ENTRIES.lock().unwrap_or_else(PoisonError::into_inner);
    let window = watched_at("KeyXtend test: asks", spot_on_main(), false);
    move_to(&window, Point { x: 1, y: 1 });
    assert_eq!(leaves(&window), 1, "the entry asks");
    move_to(&window, Point { x: 2, y: 2 });
    assert_eq!(leaves(&window), 1, "each move asks again");
    see(window.0, Seen::Leave { captured: true });
    assert_eq!(leaves(&window), 1, "a leave under capture asks again");
    see(window.0, Seen::Leave { captured: true });
    assert_eq!(leaves(&window), 0, "but once until the next move");
}

/// A press dragged off the keyboard and let go there leaves no leave to come, so the lost capture ends the entry.
#[test]
fn a_capture_lost_off_the_window_ends_the_entry_and_one_lost_on_it_keeps_it() {
    let _e = ENTRIES.lock().unwrap_or_else(PoisonError::into_inner);
    let kb = spot_on_main();
    let window = watched_at("KeyXtend test: released", kb, false);
    let screen = |x, y| Point {
        x: kb.x + x,
        y: kb.y + y,
    };
    move_to(&window, Point { x: 5, y: 6 });
    see(window.0, Seen::Lost { on: true });
    move_to(&window, Point { x: 7, y: 8 });
    assert_eq!(kept().map(|k| k.entry), Some(screen(5, 6)), "still inside");
    lose_capture(&window);
    move_to(&window, Point { x: 9, y: 10 });
    assert_eq!(kept().map(|k| k.entry), Some(screen(9, 10)), "a new entry");
}

#[test]
fn a_capture_lost_on_the_window_while_out_asks_nothing_and_the_next_move_enters() {
    let _e = ENTRIES.lock().unwrap_or_else(PoisonError::into_inner);
    let kb = spot_on_main();
    let window = watched_at("KeyXtend test: lost while out", kb, false);
    see(window.0, Seen::Lost { on: true });
    assert_eq!(leaves(&window), 0, "nothing to leave yet");
    move_to(&window, Point { x: 3, y: 4 });
    let entry = Point {
        x: kb.x + 3,
        y: kb.y + 4,
    };
    assert_eq!(kept().map(|k| k.entry), Some(entry));
}

#[test]
fn a_leave_without_capture_after_one_under_it_ends_the_entry() {
    let _e = ENTRIES.lock().unwrap_or_else(PoisonError::into_inner);
    let kb = spot_on_main();
    let window = watched_at("KeyXtend test: asked then left", kb, false);
    let screen = |x, y| Point {
        x: kb.x + x,
        y: kb.y + y,
    };
    move_to(&window, Point { x: 5, y: 6 });
    see(window.0, Seen::Leave { captured: true });
    see(window.0, Seen::Leave { captured: false });
    move_to(&window, Point { x: 9, y: 10 });
    assert_eq!(kept().map(|k| k.entry), Some(screen(9, 10)), "a new entry");
}
