//! The Windows desk over hidden test windows: a posted turn reaches its window, closed and hung windows read as such,
//! and a sent turn is one tagged wheel. No test moves the pointer or sends real input.
#![allow(
    unsafe_code,
    reason = "The tests read the union of the input they build."
)]

use super::*;
use crate::screen::contains;
use crate::scroll::testwin::TestWin;

/// The test window's box.
const BOX: Rect = Rect {
    x: 100,
    y: 120,
    w: 300,
    h: 200,
};
/// A point on it.
const AT: Point = Point { x: 150, y: 160 };
/// A third of a notch.
const THIRD: i32 = 40;

#[test]
fn a_posted_turn_reaches_the_window_at_its_point() {
    let win = TestWin::start(BOX, false);
    let window = id_of(win.hwnd());
    post(window, AT, Dir::Down, -THIRD).unwrap();
    post(window, AT, Dir::Right, THIRD).unwrap();
    let want = [
        (
            win.raw(),
            WM_MOUSEWHEEL,
            wheel_wparam(-THIRD),
            point_lparam(AT),
        ),
        (
            win.raw(),
            WM_MOUSEHWHEEL,
            wheel_wparam(THIRD),
            point_lparam(AT),
        ),
    ];
    assert_eq!(win.wheels(), want);
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
