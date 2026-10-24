//! What G6 still reads and does itself: a window's picture and the share of it that changed, and one UI Automation
//! `ScrollPattern` step for the percent lag it records. The pad does all the scrolling; the percent rules are the
//! product's (`kx_platform_windows::scroll`).
#![cfg(windows)]

use kx_module_api::ports::Dir;
use kx_platform_windows::uia;
use spike_core::capture;
use windows::Win32::Foundation::RECT;
use windows::Win32::UI::Accessibility::{
    IUIAutomationScrollPattern, ScrollAmount, ScrollAmount_NoAmount, ScrollAmount_SmallDecrement,
    ScrollAmount_SmallIncrement,
};

use crate::stats;

pub use kx_platform_windows::scroll::{along, percent_moved, reach};

/// Bytes per screen pixel in a picture.
const PIXEL: usize = capture::BYTES_PP as usize;

/// UI Automation's small steps towards `dir`: sideways and downward.
fn amounts(dir: Dir) -> (ScrollAmount, ScrollAmount) {
    let (none, more, less) = (
        ScrollAmount_NoAmount,
        ScrollAmount_SmallIncrement,
        ScrollAmount_SmallDecrement,
    );
    match dir {
        Dir::Down => (none, more),
        Dir::Up => (none, less),
        Dir::Right => (more, none),
        Dir::Left => (less, none),
    }
}

/// One `ScrollPattern` small step of `sp` towards `dir`, as the product's fallback takes.
pub fn step(sp: &IUIAutomationScrollPattern, dir: Dir) -> Result<(), String> {
    let (h, v) = amounts(dir);
    uia::scroll(sp, h, v, 1).map_err(|e| e.to_string())
}

/// The pixels of screen box `r` as the screen shows them, if they fit in `cap` bytes.
pub fn picture(r: RECT, cap: usize) -> Result<Vec<u8>, String> {
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
    fn each_direction_takes_its_own_small_step() {
        assert_eq!(
            amounts(Dir::Up),
            (ScrollAmount_NoAmount, ScrollAmount_SmallDecrement)
        );
        assert_eq!(
            amounts(Dir::Down),
            (ScrollAmount_NoAmount, ScrollAmount_SmallIncrement)
        );
        assert_eq!(
            amounts(Dir::Left),
            (ScrollAmount_SmallDecrement, ScrollAmount_NoAmount)
        );
        assert_eq!(
            amounts(Dir::Right),
            (ScrollAmount_SmallIncrement, ScrollAmount_NoAmount)
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
