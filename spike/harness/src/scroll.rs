//! G6 scroll routes: UI Automation's ScrollPattern, a posted wheel message, and a SendInput wheel.

use spike_core::hold::{Act, Pt};
use spike_core::inject::{self, TAG};
use spike_core::{capture, uia};
use windows::Win32::Foundation::{HWND, LPARAM, POINT, WPARAM};
use windows::Win32::UI::Accessibility::{
    IUIAutomationScrollPattern, ScrollAmount, ScrollAmount_NoAmount, ScrollAmount_SmallDecrement,
    ScrollAmount_SmallIncrement,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{MOUSEEVENTF_HWHEEL, MOUSEEVENTF_WHEEL};
use windows::Win32::UI::WindowsAndMessaging::{
    PostMessageW, WM_MOUSEHWHEEL, WM_MOUSEWHEEL, WindowFromPoint,
};

use crate::probecfg::G6;
use crate::{hookio, mouse, win};

/// Bytes per screen pixel in a picture.
const PIXEL: usize = capture::BYTES_PP as usize;
/// Bits of the low word in a message parameter.
const WORD_BITS: u32 = 16;

/// A scroll direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dir {
    /// Further down the document.
    Down,
    /// Back up.
    Up,
    /// Further right.
    Right,
    /// Back left.
    Left,
}

impl Dir {
    /// Every direction, in an order that ends near the start.
    pub const ALL: [Dir; 4] = [Dir::Down, Dir::Up, Dir::Right, Dir::Left];

    /// Name in the report.
    pub fn name(self) -> &'static str {
        match self {
            Dir::Down => "down",
            Dir::Up => "up",
            Dir::Right => "right",
            Dir::Left => "left",
        }
    }

    fn sideways(self) -> bool {
        matches!(self, Dir::Right | Dir::Left)
    }

    /// The wheel's sign: positive turns scroll up, or right on the sideways wheel.
    fn sign(self) -> i32 {
        match self {
            Dir::Down | Dir::Left => -1,
            Dir::Up | Dir::Right => 1,
        }
    }

    /// UIA small steps: sideways and downward.
    fn amounts(self) -> (ScrollAmount, ScrollAmount) {
        let (none, more, less) = (
            ScrollAmount_NoAmount,
            ScrollAmount_SmallIncrement,
            ScrollAmount_SmallDecrement,
        );
        match self {
            Dir::Down => (none, more),
            Dir::Up => (none, less),
            Dir::Right => (more, none),
            Dir::Left => (less, none),
        }
    }
}

/// A way to scroll a window that is not under the pointer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Route {
    /// UI Automation's ScrollPattern.
    Uia,
    /// A wheel message posted to the window at the target.
    Post,
    /// A SendInput wheel, with the pointer moved to the target and back.
    Send,
}

impl Route {
    /// Every route.
    pub const ALL: [Route; 3] = [Route::Uia, Route::Post, Route::Send];

    /// Name in the report.
    pub fn name(self) -> &'static str {
        match self {
            Route::Uia => "uia",
            Route::Post => "post",
            Route::Send => "send",
        }
    }
}

/// A wheel message's wParam: the turn in the high word, no keys held in the low word.
pub fn wheel_wparam(delta: i32) -> usize {
    usize::from(delta as i16 as u16) << WORD_BITS
}

/// A screen point as a message lParam: x in the low word, y in the high word.
pub fn point_lparam(p: Pt) -> isize {
    // Each word keeps the low 16 bits of its coordinate, as the message wants.
    ((p.y as u16 as isize) << WORD_BITS) | (p.x as u16 as isize)
}

/// Posts wheel messages to the window at `p`, which must be `ours` or inside it; the pointer stays put.
fn post(ours: HWND, p: Pt, dir: Dir, g: &G6) -> Result<(), String> {
    // SAFETY: a plain query.
    let hwnd = unsafe { WindowFromPoint(POINT { x: p.x, y: p.y }) };
    if !win::contains(ours, hwnd) {
        return Err(format!(
            "{} is at the target, not ours",
            win::describe(hwnd)
        ));
    }
    let msg = if dir.sideways() {
        WM_MOUSEHWHEEL
    } else {
        WM_MOUSEWHEEL
    };
    let (w, l) = (wheel_wparam(dir.sign() * g.wheel_delta), point_lparam(p));
    for _ in 0..g.notches {
        // SAFETY: posting to a live window of ours, checked above.
        unsafe { PostMessageW(Some(hwnd), msg, WPARAM(w), LPARAM(l)) }
            .map_err(|e| format!("post wheel to {}: {e}", win::describe(hwnd)))?;
    }
    Ok(())
}

/// Turns the wheel at `p` with SendInput: the pointer visits `p`, then goes back to `rest`.
fn send(p: Pt, rest: Pt, dir: Dir, g: &G6) -> Result<(), String> {
    let desk = win::desktop();
    let flags = if dir.sideways() {
        MOUSEEVENTF_HWHEEL
    } else {
        MOUSEEVENTF_WHEEL
    };
    let mut inputs = hookio::inputs(&[Act::Move(p)], desk, TAG);
    for _ in 0..g.notches {
        inputs.push(mouse::wheel(dir.sign() * g.wheel_delta, flags, TAG));
    }
    inputs.extend(hookio::inputs(&[Act::Move(rest)], desk, TAG));
    inject::send(&inputs)
}

/// Scrolls window `ours` by `route` in `dir` at `p`; `sp` is the ScrollPattern there, if any.
pub fn act(
    ours: HWND,
    route: Route,
    dir: Dir,
    (p, rest): (Pt, Pt),
    sp: Option<&IUIAutomationScrollPattern>,
    g: &G6,
) -> Result<(), String> {
    match route {
        Route::Uia => {
            let sp = sp.ok_or("no ScrollPattern at the target")?;
            let (h, v) = dir.amounts();
            uia::scroll(sp, h, v, g.notches)
        }
        Route::Post => post(ours, p, dir, g),
        Route::Send => send(p, rest, dir, g),
    }
}

/// The pixels of window `hwnd` as the screen shows them, if they fit in `cap` bytes.
pub fn picture(hwnd: HWND, cap: usize) -> Result<Vec<u8>, String> {
    let r = win::rect(hwnd)?;
    capture::grab(r.left, r.top, r.right - r.left, r.bottom - r.top, cap)
}

/// The share of pixels that differ between two pictures; `None` when their sizes differ.
pub fn changed_share(a: &[u8], b: &[u8]) -> Option<f64> {
    if a.len() != b.len() || a.is_empty() {
        return None;
    }
    let pairs = a
        .as_chunks::<PIXEL>()
        .0
        .iter()
        .zip(b.as_chunks::<PIXEL>().0);
    let changed = pairs.filter(|(x, y)| x != y).count();
    Some(changed as f64 / (a.len() / PIXEL) as f64)
}

/// True when UIA's percents moved the way `dir` scrolls; a missing or "cannot scroll" read never counts.
pub fn percent_moved(dir: Dir, before: Option<(f64, f64)>, after: Option<(f64, f64)>) -> bool {
    let (Some(b), Some(a)) = (before, after) else {
        return false;
    };
    let (from, to) = if dir.sideways() {
        (b.0, a.0)
    } else {
        (b.1, a.1)
    };
    if from < 0.0 || to < 0.0 {
        return false;
    }
    // Down and right grow a percent; up and left shrink it.
    if matches!(dir, Dir::Down | Dir::Right) {
        to > from
    } else {
        to < from
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wheel_params_pack_signed_words() {
        assert_eq!(wheel_wparam(-120), 0xFF88_0000);
        assert_eq!(wheel_wparam(120), 0x0078_0000);
        assert_eq!(point_lparam(Pt { x: -1, y: 2 }), 0x0002_FFFF);
    }

    #[test]
    fn directions_turn_the_right_wheel_the_right_way() {
        assert_eq!((Dir::Down.sign(), Dir::Down.sideways()), (-1, false));
        assert_eq!((Dir::Right.sign(), Dir::Right.sideways()), (1, true));
        assert_eq!(
            Dir::Up.amounts(),
            (ScrollAmount_NoAmount, ScrollAmount_SmallDecrement)
        );
        assert_eq!(
            Dir::Left.amounts(),
            (ScrollAmount_SmallDecrement, ScrollAmount_NoAmount)
        );
    }

    #[test]
    fn changed_share_counts_whole_pixels() {
        let a = [0u8; 16];
        let mut b = a;
        b[5] = 1;
        assert_eq!(changed_share(&a, &b), Some(0.25));
        assert_eq!(changed_share(&a, &a), Some(0.0));
        assert_eq!(changed_share(&a, &b[..8]), None);
    }

    #[test]
    fn percents_count_only_when_they_move_the_scroll_way() {
        let (top, lower) = (Some((0.0, 0.0)), Some((0.0, 4.0)));
        assert!(percent_moved(Dir::Down, top, lower));
        assert!(!percent_moved(Dir::Up, top, lower));
        assert!(percent_moved(Dir::Up, lower, top));
        assert!(!percent_moved(Dir::Right, top, lower));
        assert!(!percent_moved(Dir::Down, top, None));
        assert!(!percent_moved(
            Dir::Down,
            Some((0.0, -1.0)),
            Some((0.0, 3.0))
        ));
    }
}
