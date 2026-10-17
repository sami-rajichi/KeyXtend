//! Scrolling other apps (ADR-0029): wheel deltas from lines, the route each tick takes, UI Automation's percents, the
//! fallback to its steps and the point before the keyboard. All pure and tested everywhere; the OS parts call Windows.

use kx_module_api::ports::{Dir, Point, Reach, Rect};

mod course;
pub mod entry;
mod ping;
mod run;
#[cfg(test)]
#[cfg(windows)]
pub(crate) mod testwin;
#[cfg(windows)]
mod uia;
#[cfg(windows)]
mod win;

pub use course::{Course, MAX_TURN, Turn};
pub use ping::Pings;
pub use run::{Desk, Read, Run, Spot, Whose};
#[cfg(windows)]
pub use win::WinScroll;

/// `WHEEL_PAGESCROLL` (`UINT_MAX` in `WinUser.h`; the bindings lack it): the wheel setting that scrolls a page a notch.
pub const PAGE_SCROLL: u32 = u32::MAX;
/// UI Automation's scroll percents run from 0 to this.
pub const FULL: f64 = 100.0;
/// How far from 0 or `FULL` a percent may round and still be there.
pub const END_SLACK: f64 = 1e-9;

/// The user's wheel, as Windows reports it.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Wheel {
    /// One notch's delta: `WHEEL_DELTA`.
    pub notch: u32,
    /// Lines a notch scrolls up or down (`SPI_GETWHEELSCROLLLINES`).
    pub lines: u32,
    /// Characters a notch scrolls sideways (`SPI_GETWHEELSCROLLCHARS`).
    pub chars: u32,
    /// True when Windows gives a wheel turn to the window under the pointer (`SPI_GETMOUSEWHEELROUTING`), not to the
    /// focus, so a sent turn reaches the target.
    pub at_pointer: bool,
}

impl Wheel {
    /// The delta one line takes towards `dir`; none when that setting is 0 or a page, as the wheel cannot glide then.
    #[must_use]
    pub fn per_line(self, dir: Dir) -> Option<f64> {
        let setting = if dir.sideways() {
            self.chars
        } else {
            self.lines
        };
        let glides = self.notch > 0 && setting > 0 && setting != PAGE_SCROLL;
        glides.then(|| f64::from(self.notch) / f64::from(setting))
    }
}

/// A way to scroll a target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Route {
    /// UI Automation's `ScrollPattern` line steps: the fallback, and the way when the wheel cannot glide.
    Uia,
    /// A wheel message posted to the window at the target, while the pointer is elsewhere.
    Post,
    /// A `SendInput` wheel at the pointer, while the pointer is on the target.
    Send,
}

/// A tick's route: UI Automation's steps when the wheel cannot glide, else a sent wheel while the pointer is on the
/// target, else a posted one.
#[must_use]
pub const fn route(on_pointer: bool, wheel: bool) -> Route {
    match (wheel, on_pointer) {
        (false, _) => Route::Uia,
        (true, true) => Route::Send,
        (true, false) => Route::Post,
    }
}

/// The wheel's sign towards `dir`: positive scrolls up, or right on the sideways wheel.
#[must_use]
pub const fn sign(dir: Dir) -> i32 {
    match dir {
        Dir::Up | Dir::Right => 1,
        Dir::Down | Dir::Left => -1,
    }
}

/// The low 16 bits of `n`, which is all a message word keeps.
fn low_word(n: i32) -> u16 {
    let [lo, hi, ..] = n.to_le_bytes();
    u16::from_le_bytes([lo, hi])
}

/// A wheel message's wParam: the turn in the high word, no keys held in the low word.
#[must_use]
pub fn wheel_wparam(delta: i32) -> usize {
    usize::from(low_word(delta)) << u16::BITS
}

/// A screen point as a message lParam: x in the low word, y in the high word.
#[must_use]
pub fn point_lparam(p: Point) -> isize {
    let words = (usize::from(low_word(p.y)) << u16::BITS) | usize::from(low_word(p.x));
    words.cast_signed()
}

/// A message lParam as a screen or client point: x in the low word, y in the high word, each signed.
#[must_use]
pub fn lparam_point(lparam: isize) -> Point {
    let [x0, x1, y0, y1, ..] = lparam.to_le_bytes();
    let word = |lo, hi| i32::from(i16::from_le_bytes([lo, hi]));
    Point {
        x: word(x0, x1),
        y: word(y0, y1),
    }
}

/// The percent towards `dir` from UI Automation's (horizontal, vertical) pair, a rounding past its range kept in it;
/// none when that way cannot scroll (-1) or the value is no percent.
#[must_use]
pub fn along(dir: Dir, (h, v): (f64, f64)) -> Option<f64> {
    let pct = if dir.sideways() { h } else { v };
    let range = -END_SLACK..=FULL + END_SLACK;
    range.contains(&pct).then(|| pct.clamp(0.0, FULL))
}

/// True when the percent grows towards `dir`: down and right.
const fn grows(dir: Dir) -> bool {
    matches!(dir, Dir::Down | Dir::Right)
}

/// True when UI Automation's percents moved the way `dir` scrolls; a missing or "cannot scroll" read never counts.
#[must_use]
pub fn percent_moved(dir: Dir, before: Option<(f64, f64)>, after: Option<(f64, f64)>) -> bool {
    let read = |pair: Option<(f64, f64)>| pair.and_then(|p| along(dir, p));
    let (Some(from), Some(to)) = (read(before), read(after)) else {
        return false;
    };
    if grows(dir) { to > from } else { to < from }
}

/// The box UI Automation gave (left, top, right, bottom), checked against `whole`, the box around every screen: none
/// when it is empty, turned inside out, or reaches past `whole` by more than `whole`'s own size.
#[must_use]
pub fn area(edges: [i32; 4], whole: Rect) -> Option<Rect> {
    let [left, top, right, bottom] = edges.map(i64::from);
    let [d_left, d_top, d_right, d_bottom] = crate::screen::edges(whole).map(i64::from);
    let (wide, high) = (d_right - d_left, d_bottom - d_top);
    let near = left >= d_left - wide && top >= d_top - high;
    let far = right <= d_right + wide && bottom <= d_bottom + high;
    (right > left && bottom > top && near && far).then(|| crate::screen::rect_of(edges))
}

/// How far `dir` can still go at percent `pct`: its end at 0 or `FULL` (within `END_SLACK`), unknown without one.
#[must_use]
pub fn reach(dir: Dir, pct: Option<f64>) -> Reach {
    let Some(pct) = pct else {
        return Reach::Unknown;
    };
    let end = if grows(dir) {
        pct >= FULL - END_SLACK
    } else {
        pct <= END_SLACK
    };
    if end { Reach::End } else { Reach::More }
}

#[cfg(test)]
mod tests;
