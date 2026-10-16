//! G6 scroll routes: UI Automation's `ScrollPattern`, a posted wheel message, and a `SendInput` wheel. The route
//! enum, the wheel's sign and words, and the percent judge are the product's (`kx_platform_windows::scroll`).
#![cfg(windows)]

use kx_module_api::ports::{self, Point};
use kx_platform_windows::keys::TAG;
use kx_platform_windows::scroll::{point_lparam, sign, wheel_wparam};
use spike_core::pt::Pt;
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
use crate::simin::{self, Act};
use crate::stats;
use crate::{keys, mouse, win};

pub use kx_platform_windows::scroll::{Route, percent_moved};

/// Bytes per screen pixel in a picture.
const PIXEL: usize = capture::BYTES_PP as usize;

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

    /// The product's direction.
    pub fn port(self) -> ports::Dir {
        match self {
            Dir::Down => ports::Dir::Down,
            Dir::Up => ports::Dir::Up,
            Dir::Right => ports::Dir::Right,
            Dir::Left => ports::Dir::Left,
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

/// Posts wheel messages to the window at `p`, which must be `ours` or inside it; the pointer stays put.
#[allow(unsafe_code, reason = "A plain query.")]
fn post(ours: HWND, p: Pt, dir: Dir, g: &G6) -> Result<(), String> {
    // SAFETY: a plain query.
    let hwnd = unsafe { WindowFromPoint(POINT { x: p.x, y: p.y }) };
    if !win::contains(ours, hwnd) {
        return Err(format!(
            "{} is at the target, not ours",
            win::describe(hwnd)
        ));
    }
    let msg = if dir.port().sideways() {
        WM_MOUSEHWHEEL
    } else {
        WM_MOUSEWHEEL
    };
    let at = Point { x: p.x, y: p.y };
    let (w, l) = (
        wheel_wparam(sign(dir.port()) * g.wheel_delta),
        point_lparam(at),
    );
    for _ in 0..g.notches {
        // SAFETY: posting to a live window of ours, checked above.
        unsafe { PostMessageW(Some(hwnd), msg, WPARAM(w), LPARAM(l)) }
            .map_err(|e| format!("post wheel to {}: {e}", win::describe(hwnd)))?;
    }
    Ok(())
}

/// Turns the wheel at `p` with `SendInput`: the pointer visits `p`, then goes back to `rest`.
fn send(p: Pt, rest: Pt, dir: Dir, g: &G6) -> Result<(), String> {
    let desk = win::desktop();
    let flags = if dir.port().sideways() {
        MOUSEEVENTF_HWHEEL
    } else {
        MOUSEEVENTF_WHEEL
    };
    let mut inputs = simin::inputs(&[Act::Move(p)], desk, TAG);
    for _ in 0..g.notches {
        inputs.push(mouse::wheel(sign(dir.port()) * g.wheel_delta, flags, TAG));
    }
    inputs.extend(simin::inputs(&[Act::Move(rest)], desk, TAG));
    keys::send(&inputs)
}

/// Scrolls window `ours` by `route` in `dir` at `p`; `sp` is the `ScrollPattern` there, if any.
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
    Some(stats::ratio(changed, a.len() / PIXEL))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_gate_direction_is_the_products_of_the_same_name() {
        for dir in Dir::ALL {
            let name = serde_json::to_value(dir.port()).unwrap();
            assert_eq!(name, dir.name(), "{dir:?}");
        }
    }

    #[test]
    fn directions_take_the_right_uia_steps() {
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
}
