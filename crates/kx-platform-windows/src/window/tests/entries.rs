//! The entry watch on hidden windows of the test thread: the guard keeps the newest history point outside the watched
//! window, once per entry. The history, the pointer's place and the capture come from the test; none moves the pointer.

use std::sync::{Mutex, PoisonError};

use kx_module_api::ports::{Point, Rect};
use kx_platform::ui::{Screens, UiError, WindowGuard};
use windows::Win32::Foundation::{LPARAM, WPARAM};
use windows::Win32::UI::Controls::WM_MOUSELEAVE;
use windows::Win32::UI::WindowsAndMessaging::{
    MSG, PM_REMOVE, PeekMessageW, SWP_NOACTIVATE, SWP_NOZORDER, SendMessageW, SetWindowPos,
    WM_CAPTURECHANGED, WM_MOUSEMOVE,
};

use super::super::entry::{enter, kept, mark, mark_with, see, watched};
use super::super::guard::guard;
use super::super::watch::Seen;
use super::{Hidden, WinGuard};
use crate::WinScreens;
use crate::scroll::point_lparam;

/// Keeps these tests apart: the kept entry is one per process.
static ENTRIES: Mutex<()> = Mutex::new(());

/// A box on the main screen's work area, 100 pixels in from its corner.
fn spot_on_main() -> Rect {
    let main = WinScreens
        .work_areas()
        .into_iter()
        .find(|s| s.primary)
        .unwrap();
    Rect {
        x: main.rect.x + 100,
        y: main.rect.y + 100,
        w: 300,
        h: 200,
    }
}

/// Moves and sizes the hidden `window` to `r`, never showing or activating it.
#[allow(unsafe_code, reason = "Moves the test's own hidden window.")]
fn place(window: &Hidden, r: Rect) {
    let flags = SWP_NOZORDER | SWP_NOACTIVATE;
    // SAFETY: a move of the test's own hidden window.
    unsafe { SetWindowPos(window.0, None, r.x, r.y, r.w, r.h, flags) }.unwrap();
}

/// Tells `window` the pointer moved to its client point `p`, as Windows does.
#[allow(unsafe_code, reason = "Sends a message to the test's own window.")]
fn move_to(window: &Hidden, p: Point) {
    let at = LPARAM(point_lparam(p));
    // SAFETY: the message carries no pointer.
    unsafe { SendMessageW(window.0, WM_MOUSEMOVE, Some(WPARAM(0)), Some(at)) };
}

/// Tells `window` the pointer left it, as Windows does.
#[allow(unsafe_code, reason = "Sends a message to the test's own window.")]
fn leave(window: &Hidden) {
    // SAFETY: the message carries no pointer.
    unsafe { SendMessageW(window.0, WM_MOUSELEAVE, None, None) };
}

#[test]
fn the_guard_keeps_the_newest_history_point_outside_its_window() {
    let _e = ENTRIES.lock().unwrap_or_else(PoisonError::into_inner);
    let (window, kb) = (Hidden::new("KeyXtend test: keyboard"), spot_on_main());
    place(&window, kb);
    let entry = Point {
        x: kb.x + 10,
        y: kb.y + 1,
    };
    let before = Point {
        x: kb.x + 10,
        y: kb.y - 3,
    };
    let history = vec![(entry.x, entry.y), (kb.x + 10, kb.y), (before.x, before.y)];
    enter(window.0, entry, |from| {
        assert_eq!(from, entry, "the history starts at the entry");
        history
    });
    let kept = kept().unwrap();
    assert_eq!(
        (kept.outside, kept.entry, kept.keyboard),
        (Some(before), entry, kb)
    );
}

#[test]
fn a_watched_window_keeps_one_entry_until_the_pointer_leaves_it() {
    let _e = ENTRIES.lock().unwrap_or_else(PoisonError::into_inner);
    let (window, kb) = (Hidden::new("KeyXtend test: watched"), spot_on_main());
    place(&window, kb);
    guard(window.0).unwrap();
    mark(window.0).unwrap();
    assert!(watched(window.0));
    let screen = |x, y| Point {
        x: kb.x + x,
        y: kb.y + y,
    };
    move_to(&window, Point { x: 5, y: 6 });
    assert_eq!(kept().map(|k| k.entry), Some(screen(5, 6)));
    move_to(&window, Point { x: 7, y: 8 });
    assert_eq!(kept().map(|k| k.entry), Some(screen(5, 6)), "still inside");
    leave(&window);
    move_to(&window, Point { x: 9, y: 10 });
    assert_eq!(kept().map(|k| k.entry), Some(screen(9, 10)), "a new entry");
    assert_eq!(kept().map(|k| k.keyboard), Some(kb));
}

#[test]
fn an_unwatched_window_keeps_nothing_and_a_hidden_one_cannot_be_watched() {
    let _e = ENTRIES.lock().unwrap_or_else(PoisonError::into_inner);
    let (watched_one, plain) = (
        Hidden::new("KeyXtend test: noted"),
        Hidden::new("KeyXtend test: unwatched"),
    );
    guard(watched_one.0).unwrap();
    mark(watched_one.0).unwrap();
    move_to(&watched_one, Point { x: 1, y: 1 });
    let was = kept();
    guard(plain.0).unwrap();
    move_to(&plain, Point { x: 2, y: 2 });
    assert_eq!(kept(), was, "only the watched window counts");
    assert!(!watched(plain.0));
    let hidden = WinGuard.watch_entries("KeyXtend test: unwatched");
    assert_eq!(hidden, Err(UiError::NoWindow));
}

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

/// A hidden guarded window at `kb`, watched with the pointer `on` it or not.
fn watched_at(title: &str, kb: Rect, on: bool) -> Hidden {
    let window = Hidden::new(title);
    place(&window, kb);
    guard(window.0).unwrap();
    mark_with(window.0, on).unwrap();
    window
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

#[test]
fn the_kept_entry_goes_with_the_watched_window() {
    let _e = ENTRIES.lock().unwrap_or_else(PoisonError::into_inner);
    let window = watched_at("KeyXtend test: going", spot_on_main(), false);
    move_to(&window, Point { x: 1, y: 1 });
    assert!(kept().is_some());
    drop(window);
    assert_eq!(kept(), None);
}

#[test]
fn only_a_guarded_window_can_be_watched() {
    let _e = ENTRIES.lock().unwrap_or_else(PoisonError::into_inner);
    let plain = Hidden::new("KeyXtend test: not guarded");
    assert!(matches!(mark(plain.0), Err(UiError::Refused(_))));
    assert!(!watched(plain.0));
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
