//! UI Automation on hidden test windows: a read within its cap, steps that move the bars, nothing that scrolls, and a
//! window that stopped reading its messages, which a read gives up on at its cap.

use super::work::amounts;
use super::*;
use crate::scroll::testwin::{TestWin, WAIT, one_client};
use kx_module_api::ports::Rect;
use windows::Win32::UI::Accessibility::{
    ScrollAmount_NoAmount, ScrollAmount_SmallDecrement, ScrollAmount_SmallIncrement,
};

/// The test window's box.
const BOX: Rect = Rect {
    x: 100,
    y: 120,
    w: 300,
    h: 200,
};
/// The `[scroll] read_ms` default.
const READ_MS: u32 = 300;
/// A short cap for the stuck window, so the test ends soon.
const SHORT: Duration = Duration::from_millis(100);
/// Steps that move a bar of 90 positions by a tenth.
const TENTH: u16 = 9;

/// `READ_MS` as a cap.
fn cap() -> Duration {
    Duration::from_millis(u64::from(READ_MS))
}

#[test]
fn a_read_finds_the_box_and_percents_within_its_cap() {
    let _one = one_client();
    let win = TestWin::start(BOX, false);
    let uia = Uia::start(READ_MS);
    let read = uia.read(Seek::Window(win.raw()), cap()).expect("a read");
    assert_eq!(read.area, Some(crate::screen::edges(BOX)));
    assert_eq!(read.pair, Some((50.0, 50.0)), "both bars half way");
    assert!(uia.idle(), "nothing left running");
}

#[test]
fn steps_move_the_bars_and_a_read_behind_them_sees_it() {
    let _one = one_client();
    let win = TestWin::start(BOX, false);
    let uia = Uia::start(READ_MS);
    let seek = Seek::Window(win.raw());
    assert!(uia.steps(seek, Dir::Down, TENTH));
    let down = uia.read(seek, cap()).and_then(|r| r.pair);
    assert_eq!(down, Some((50.0, 60.0)));
    assert!(uia.steps(seek, Dir::Left, TENTH), "free again once read");
    let left = uia.read(seek, cap()).and_then(|r| r.pair);
    assert_eq!(left, Some((40.0, 60.0)));
}

#[test]
fn a_window_with_nothing_that_scrolls_reads_no_box() {
    let _one = one_client();
    let win = TestWin::plain(BOX);
    let uia = Uia::start(READ_MS);
    let read = uia.read(Seek::Window(win.raw()), cap()).expect("a read");
    assert_eq!((read.area, read.pair), (None, None));
}

/// A call to a window that reads no messages holds every UI Automation call in the process until UI Automation's own
/// timeout, which the thread sets to `read_ms`; then reads work again.
#[test]
fn a_window_that_stopped_reading_costs_one_timeout_then_reads_work_again() {
    let _one = one_client();
    let (stuck, fine) = (TestWin::start(BOX, true), TestWin::start(BOX, false));
    let uia = Uia::start(READ_MS);
    let started = Instant::now();
    assert!(uia.read(Seek::Window(stuck.raw()), SHORT).is_none());
    assert!(started.elapsed() >= SHORT, "it waited its cap");
    let after = uia
        .read(Seek::Window(fine.raw()), WAIT)
        .expect("a read once it timed out");
    assert_eq!(after.area, Some(crate::screen::edges(BOX)));
    assert!(uia.idle(), "nothing piled up");
}

#[test]
fn a_read_waits_behind_one_job_at_most_and_steps_go_only_to_a_free_thread() {
    let (jobs, _held) = mpsc::channel();
    let uia = Uia {
        jobs,
        pending: Arc::new(AtomicUsize::new(0)),
        cap: SHORT,
    };
    let seek = Seek::At(Point::default());
    assert!(uia.steps(seek, Dir::Down, 1), "a free thread takes steps");
    assert!(!uia.idle() && !uia.steps(seek, Dir::Down, 1));
    let started = Instant::now();
    assert!(
        uia.read(seek, SHORT).is_none(),
        "behind the steps, it waits its cap"
    );
    assert!(started.elapsed() >= SHORT);
    let quick = Instant::now();
    assert!(
        uia.read(seek, WAIT).is_none(),
        "behind two, it gives up at once"
    );
    assert!(quick.elapsed() < WAIT);
}

#[test]
fn each_way_takes_its_own_small_step() {
    let (none, more, less) = (
        ScrollAmount_NoAmount,
        ScrollAmount_SmallIncrement,
        ScrollAmount_SmallDecrement,
    );
    assert_eq!(amounts(Dir::Down), (none, more));
    assert_eq!(amounts(Dir::Up), (none, less));
    assert_eq!(amounts(Dir::Right), (more, none));
    assert_eq!(amounts(Dir::Left), (less, none));
}
