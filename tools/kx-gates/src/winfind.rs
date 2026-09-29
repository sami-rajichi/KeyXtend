//! Finding windows: what a wanted window looks like, waiting for one to appear or go, and the one on top over a box.
#![cfg(windows)]

use std::collections::HashSet;

use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::UI::WindowsAndMessaging::IsWindow;

use crate::win::{class, key, poll_until, rect, seen_windows, title, top_windows};

/// What a wanted window looks like.
#[derive(Debug, Clone, Default)]
pub struct Match {
    /// Text the title must contain.
    pub title_has: Option<String>,
    /// Exact class name.
    pub class: Option<String>,
    /// Windows to ignore because they existed before.
    pub skip: HashSet<isize>,
}

/// The topmost visible top-level window that fits `m`.
pub fn find(m: &Match) -> Option<HWND> {
    top_windows().into_iter().find(|&w| {
        !m.skip.contains(&key(w))
            && m.class.as_ref().is_none_or(|c| class(w) == *c)
            && m.title_has
                .as_ref()
                .is_none_or(|t| title(w).contains(t.as_str()))
    })
}

/// Waits up to `timeout_ms` for a window that fits `m`.
pub fn wait_for(m: &Match, timeout_ms: u64, poll_ms: u64) -> Option<HWND> {
    poll_until(timeout_ms, poll_ms, || find(m))
}

/// True while `hwnd` is a live window.
#[allow(unsafe_code, reason = "Plain query.")]
pub fn exists(hwnd: HWND) -> bool {
    // SAFETY: plain query; any handle value is allowed.
    unsafe { IsWindow(Some(hwnd)).as_bool() }
}

/// Waits up to `timeout_ms` for `hwnd` to go away; true if it did.
pub fn wait_gone(hwnd: HWND, timeout_ms: u64, poll_ms: u64) -> bool {
    poll_until(timeout_ms, poll_ms, || (!exists(hwnd)).then_some(())).is_some()
}

/// Waits up to `timeout_ms` until no window fits `m`; returns one that still does.
pub fn wait_no_match(m: &Match, timeout_ms: u64, poll_ms: u64) -> Option<HWND> {
    match poll_until(timeout_ms, poll_ms, || find(m).is_none().then_some(())) {
        Some(()) => None,
        None => find(m),
    }
}

/// The seen window on top over `w`'s box, going by z-order; it finds click-through windows, which `root_at` skips.
pub fn over(w: HWND) -> Option<HWND> {
    let r = rect(w).ok()?;
    let boxes = seen_windows()
        .into_iter()
        .filter_map(|h| rect(h).ok().map(|b| (key(h), b)));
    first_over(boxes, &r).map(spike_core::window::from_raw)
}

/// The first window in `front_first` whose box overlaps `r`.
fn first_over(front_first: impl IntoIterator<Item = (isize, RECT)>, r: &RECT) -> Option<isize> {
    let overlaps =
        |b: &RECT| b.left < r.right && r.left < b.right && b.top < r.bottom && r.top < b.bottom;
    front_first
        .into_iter()
        .find(|(_, b)| overlaps(b))
        .map(|(k, _)| k)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_first_window_over_a_box_is_the_one_on_top_there() {
        let bx = |left, top, right, bottom| RECT {
            left,
            top,
            right,
            bottom,
        };
        let target = bx(100, 100, 200, 150);
        let list = [
            (1, bx(0, 0, 50, 50)),
            (2, bx(150, 140, 300, 300)),
            (3, target),
        ];
        assert_eq!(first_over(list, &target), Some(2), "a corner is enough");
        let beside = [(1, bx(200, 100, 300, 150)), (3, target)];
        assert_eq!(
            first_over(beside, &target),
            Some(3),
            "touching edges do not overlap"
        );
    }
}
