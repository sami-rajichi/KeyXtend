//! UI Automation on hidden test windows: a read within its cap, steps that move the bars, nothing that scrolls, and a
//! window that stopped reading its messages, which a read gives up on at its cap.

use super::work::{amounts, client, look, step};
use super::*;
use crate::com::Com;
use crate::scroll::testwin::{TestWin, WAIT, one_client};
use kx_module_api::ports::{Point, Rect};
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
/// A wide table in the test window's client, as in a long page.
const TABLE: Rect = Rect {
    x: 10,
    y: 10,
    w: 200,
    h: 100,
};

/// A cap no test should reach.
const LONG_MS: u32 = 1000;

/// `READ_MS` as a cap.
fn cap() -> Duration {
    Duration::from_millis(u64::from(READ_MS))
}

/// The thread once its client has started: the first read at `win` may wait for it.
fn started(win: &TestWin) -> Uia {
    let uia = Uia::start(READ_MS);
    let _ = uia.read(win.seek(), Dir::Down, WAIT);
    uia
}

/// A body that takes no job until its handle drops, as a client slow to start.
fn slow(inbox: &mpsc::Receiver<Job>, _: &AtomicUsize, _: u32) {
    let _ = inbox.recv();
}

#[test]
fn a_read_finds_the_box_and_percents_within_its_cap() {
    let _one = one_client();
    let win = TestWin::start(BOX, false);
    let uia = started(&win);
    let read = uia.read(win.seek(), Dir::Down, cap()).expect("a read");
    assert_eq!(read.area, Some(crate::screen::edges(BOX)));
    assert_eq!(read.pair, Some((50.0, 50.0)), "both bars half way");
    assert!(uia.idle(), "nothing left running");
}

/// A seek at a point in a top window reads from that window, as for a target ours covers: down there, the page. A
/// hidden window shows UI Automation no children, so the walk down to the point is tested on a fake tree in `down`.
#[test]
fn a_seek_at_a_point_in_a_window_reads_from_that_window() {
    let _one = one_client();
    let win = TestWin::split(BOX, TABLE);
    let uia = started(&win);
    let table = win.bounds();
    let mid = Point {
        x: table.x + table.w / 2,
        y: table.y + table.h / 2,
    };
    let down = uia.read(Seek::Window(win.top_id(), mid), Dir::Down, cap());
    let page = Some(crate::screen::edges(BOX));
    assert_eq!(down.map(|r| r.area), Some(page));
}

#[test]
fn steps_move_the_bars_and_a_read_behind_them_sees_it() {
    let _one = one_client();
    let win = TestWin::start(BOX, false);
    let uia = started(&win);
    let seek = win.seek();
    assert!(uia.steps(seek, Dir::Down, TENTH));
    let down = uia.read(seek, Dir::Down, cap()).and_then(|r| r.pair);
    assert_eq!(down, Some((50.0, 60.0)));
    assert!(uia.steps(seek, Dir::Left, TENTH), "free again once read");
    let left = uia.read(seek, Dir::Left, cap()).and_then(|r| r.pair);
    assert_eq!(left, Some((40.0, 60.0)));
}

/// Over a wide table in a long page, each way reads and steps the first that scrolls along its axis: down the page,
/// sideways the table, so down never reads the table's "cannot scroll".
#[test]
fn each_way_reads_and_steps_the_first_that_scrolls_along_its_axis() {
    let _one = one_client();
    let win = TestWin::split(BOX, TABLE);
    let uia = started(&win);
    let seek = win.seek();
    let (page, table) = (
        crate::screen::edges(BOX),
        crate::screen::edges(win.bounds()),
    );
    let down = uia.read(seek, Dir::Down, cap()).expect("a read down");
    assert_eq!(
        (down.area, down.pair.map(|p| p.1)),
        (Some(page), Some(50.0))
    );
    let right = uia.read(seek, Dir::Right, cap()).expect("a read sideways");
    assert_eq!(
        (right.area, right.pair.map(|p| p.0)),
        (Some(table), Some(50.0))
    );
    assert!(uia.steps(seek, Dir::Down, TENTH));
    let page_pct = uia.read(seek, Dir::Up, cap()).and_then(|r| r.pair);
    assert_eq!(page_pct.map(|p| p.1), Some(60.0), "the page moved");
    assert!(uia.steps(seek, Dir::Left, TENTH));
    let table_pct = uia.read(seek, Dir::Left, cap()).and_then(|r| r.pair);
    assert_eq!(table_pct.map(|p| p.0), Some(40.0), "the table moved");
}

#[test]
fn a_window_with_nothing_that_scrolls_reads_no_box() {
    let _one = one_client();
    let win = TestWin::plain(BOX);
    let uia = started(&win);
    let read = uia.read(win.seek(), Dir::Down, cap()).expect("a read");
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
    assert!(uia.read(stuck.seek(), Dir::Down, SHORT).is_none());
    assert!(started.elapsed() >= SHORT, "it waited its cap");
    let after = uia
        .read(fine.seek(), Dir::Down, WAIT)
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
        uia.read(seek, Dir::Down, SHORT).is_none(),
        "behind the steps, it waits its cap"
    );
    assert!(started.elapsed() >= SHORT);
    let quick = Instant::now();
    assert!(
        uia.read(seek, Dir::Down, WAIT).is_none(),
        "behind two, it gives up at once"
    );
    assert!(quick.elapsed() < WAIT);
}

#[test]
fn starting_never_waits_for_the_client() {
    let begun = Instant::now();
    let uia = Uia::spawn(LONG_MS, slow);
    assert!(begun.elapsed() < Duration::from_millis(u64::from(LONG_MS)));
    let seek = Seek::At(Point::default());
    assert!(
        uia.read(seek, Dir::Down, SHORT).is_none(),
        "it waits its cap at most"
    );
}

#[test]
fn steps_past_their_time_look_nothing_up() {
    let _one = one_client();
    let (a, b) = (TestWin::start(BOX, false), TestWin::start(BOX, false));
    let _com = Com::start().unwrap();
    let auto = client(READ_MS).unwrap();
    let (at_a, at_b) = (a.seek(), b.seek());
    let mut held = None;
    step(&auto, (at_a, Dir::Down, 0), deadline(WAIT), &mut held);
    let held_a = |k: &work::Held| k.as_ref().is_some_and(|h| h.0 == at_a);
    assert!(held_a(&held), "a lookup in time");
    let past = Instant::now().checked_sub(SHORT).unwrap();
    step(&auto, (at_b, Dir::Down, TENTH), past, &mut held);
    assert!(held_a(&held), "none past its time");
}

/// A walk up that cannot finish, as one past its time, reads nothing and keeps the pattern held for steps.
#[test]
fn a_walk_that_cannot_finish_reads_nothing_and_keeps_the_held_pattern() {
    let _one = one_client();
    let (a, b) = (TestWin::start(BOX, false), TestWin::start(BOX, false));
    let _com = Com::start().unwrap();
    let auto = client(READ_MS).unwrap();
    let (at_a, at_b) = (a.seek(), b.seek());
    let mut held = None;
    step(&auto, (at_a, Dir::Down, 0), deadline(WAIT), &mut held);
    let past = Instant::now().checked_sub(SHORT).unwrap();
    assert!(
        look(&auto, (at_b, Dir::Down), past, &mut held).is_none(),
        "no read"
    );
    let held_a = held.as_ref().is_some_and(|h| h.0 == at_a);
    assert!(held_a, "the pattern stays");
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
