//! The Windows desk over hidden test windows: a posted turn reaches its window in its own pixels, closed, hung and
//! silent windows read as such, a sent turn is one tagged wheel, and the window behind ours is never ours. No test
//! moves the pointer or sends real input.
#![allow(
    unsafe_code,
    reason = "The tests read the union of the input they build."
)]

use super::*;
use crate::screen::contains;
use crate::scroll::lparam_point;
use crate::scroll::testwin::{TestWin, WAIT, one_client};
use crate::window::{THROUGH, is_ours, is_through};
use kx_module_api::ports::{Aim, TargetId};
use pinger::answered;
use windows::Win32::Foundation::{ERROR_ACCESS_DENIED, ERROR_INVALID_WINDOW_HANDLE, ERROR_TIMEOUT};
use windows::Win32::UI::HiDpi::LogicalToPhysicalPointForPerMonitorDPI;
use windows::Win32::UI::WindowsAndMessaging::{
    GA_ROOT, GWL_EXSTYLE, GetAncestor, SetWindowLongPtrW,
};

/// The test window's box.
const BOX: Rect = Rect {
    x: 100,
    y: 120,
    w: 300,
    h: 200,
};
/// A point on it at every common display scale, whose pixels divide evenly by each.
const AT: Point = Point { x: 210, y: 315 };
/// A third of a notch.
const THIRD: i32 = 40;
/// A notch at Windows' default of three lines.
const LINES: f64 = 3.0;
/// `[scroll]` with a short read, so a window that stopped reading costs the tests little.
const QUICK: ScrollConfig = ScrollConfig {
    check_ms: 250,
    read_ms: 100,
    edge_px: 8,
};

/// Point `p` in the pixels of `hwnd` back on the screen's physical pixels.
fn physical(hwnd: HWND, p: Point) -> Point {
    let mut at = POINT { x: p.x, y: p.y };
    // SAFETY: a local point and the test's own window.
    assert!(unsafe { LogicalToPhysicalPointForPerMonitorDPI(Some(hwnd), &raw mut at) }.as_bool());
    Point { x: at.x, y: at.y }
}

#[test]
fn a_posted_turn_reaches_the_window_at_its_point() {
    let win = TestWin::start(BOX, false);
    let window = id_of(win.hwnd());
    post(window, AT, Dir::Down, -THIRD).unwrap();
    post(window, AT, Dir::Right, THIRD).unwrap();
    let got = win.wheels();
    let raw = win.raw();
    let turns: Vec<_> = got.iter().map(|s| (s.0, s.1, s.2)).collect();
    let down = (raw, WM_MOUSEWHEEL, wheel_wparam(-THIRD));
    assert_eq!(turns, [down, (raw, WM_MOUSEHWHEEL, wheel_wparam(THIRD))]);
    for (.., l) in got {
        assert_eq!(physical(win.hwnd(), lparam_point(l)), AT, "its own pixels");
    }
}

#[test]
fn a_window_that_stopped_reading_gets_no_post_after_its_first_unanswered_ping() {
    let _one = one_client();
    let mut stuck = TestWin::start(BOX, true);
    let window = id_of(stuck.hwnd());
    let desk = WinDesk::new(QpcClock::new().unwrap(), QUICK);
    desk.ping(window);
    desk.pinger.settle(WAIT);
    assert!(
        desk.hung(window) && !hung(window),
        "busy before Windows says hung"
    );
    let mut run = Run::new(desk, QUICK);
    let id = TargetId::new(window);
    let (on_pointer, area, ours) = (false, None, false);
    let aim = Aim {
        at: AT,
        on_pointer,
        area,
        ours,
        id,
    };
    assert_eq!(run.turn(&aim, Dir::Down, LINES), Ok(()));
    stuck.release();
    assert!(stuck.wheels().is_empty(), "nothing was posted");
}

#[test]
fn a_window_that_answers_its_ping_is_not_busy() {
    let win = TestWin::start(BOX, false);
    let window = id_of(win.hwnd());
    let pinger = Pinger::start(QUICK.read_ms, QUICK.check_ms);
    pinger.ask(window);
    pinger.settle(WAIT);
    assert!(!pinger.busy(window));
}

#[test]
fn a_live_window_is_alive_and_not_hung_and_a_closed_one_is_gone() {
    let win = TestWin::start(BOX, false);
    let window = id_of(win.hwnd());
    assert!(alive(window) && !hung(window));
    drop(win);
    assert!(!alive(window));
    assert_eq!(post(window, AT, Dir::Down, -THIRD), Err(ScrollError::Gone));
}

#[test]
fn the_wheel_has_windows_notch() {
    assert_eq!(wheel().notch, WHEEL_DELTA);
}

#[test]
fn a_sent_turn_is_one_wheel_at_the_pointer_tagged_ours() {
    for (dir, flags) in [
        (Dir::Down, MOUSEEVENTF_WHEEL),
        (Dir::Left, MOUSEEVENTF_HWHEEL),
    ] {
        let input = wheel_input(dir, -THIRD);
        assert_eq!(input.r#type, INPUT_MOUSE);
        // SAFETY: the input was built as a mouse input.
        let mi = unsafe { input.Anonymous.mi };
        assert_eq!((mi.dx, mi.dy, mi.dwFlags), (0, 0, flags), "no move");
        assert_eq!(mi.mouseData, (-THIRD).cast_unsigned());
        assert_eq!(mi.dwExtraInfo, TAG);
    }
}

#[test]
fn the_whole_desk_holds_every_screen() {
    let all = whole();
    for s in WinScreens.work_areas() {
        let [l, t, r, b] = crate::screen::edges(s.rect);
        assert!(contains(all, Point { x: l, y: t }), "{all:?} {s:?}");
        assert!(contains(all, Point { x: r - 1, y: b - 1 }), "{all:?} {s:?}");
    }
}

#[test]
fn a_raw_id_names_its_window() {
    let win = TestWin::start(BOX, false);
    assert_eq!(handle(id_of(win.hwnd())), win.hwnd());
}

#[test]
fn the_window_under_the_pointer_is_another_apps() {
    let Ok(p) = screen::cursor() else {
        return;
    };
    let found = spot(p).expect("a window is everywhere on the desk");
    assert_eq!(found.whose, Whose::Theirs, "the tests show no window");
    assert!(alive(found.window));
}

/// The rules on our own hidden windows, with no desktop needed: only a point inside fits the box, a hidden window is
/// neither shown nor cloaked, so it never fits, and a click-through style reads as such.
#[test]
fn a_window_fits_only_shown_holding_the_point_and_taking_clicks() {
    let win = TestWin::plain(BOX);
    let hwnd = win.hwnd();
    let outside = Point {
        x: BOX.x - 1,
        y: AT.y,
    };
    assert!(behind::holds(hwnd, AT) && !behind::holds(hwnd, outside));
    assert!(!behind::shown(hwnd) && !behind::cloaked(hwnd));
    assert!(!behind::fits(hwnd, AT), "hidden");
    assert!(!is_through(behind::ex_of(hwnd)));
    let through = isize::try_from(THROUGH).unwrap();
    // SAFETY: a style change on the test's own window.
    unsafe { SetWindowLongPtrW(hwnd, GWL_EXSTYLE, through) };
    assert!(is_through(behind::ex_of(hwnd)), "click-through");
    assert!(!behind::takes(hwnd, AT), "ours");
}

/// On a desk with a shell, the window found behind ours at the pointer is another app's that fits there.
#[test]
fn the_window_behind_ours_is_another_apps_shown_window_at_the_point() {
    let Ok(p) = screen::cursor() else {
        return;
    };
    let Some(found) = behind::behind(p) else {
        return;
    };
    // SAFETY: a plain query; a gone window has no root.
    let root = unsafe { GetAncestor(found, GA_ROOT) };
    assert!(behind::takes(root, p) && !is_ours(found));
}

#[test]
fn a_ping_windows_blocks_counts_as_answered_and_a_late_one_does_not() {
    assert!(answered(true, ERROR_TIMEOUT));
    assert!(
        answered(false, ERROR_ACCESS_DENIED),
        "the turn's own failure tells"
    );
    assert!(!answered(false, ERROR_TIMEOUT));
    assert!(!answered(false, ERROR_INVALID_WINDOW_HANDLE));
}
