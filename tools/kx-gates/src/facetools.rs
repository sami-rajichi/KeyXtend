//! Our windows found by title, and clicks only where the window is on top: G7's panel and G25's keys.
#![cfg(windows)]

use windows::Win32::Foundation::{HWND, POINT};

use crate::{mouse, win};

/// Clicks `p` only when `owner` is the window on top there, so no other window is ever clicked.
pub fn click_on(owner: HWND, p: POINT) -> Result<(), String> {
    let top = win::root_at(p);
    if top != owner {
        return Err(format!("{} covers {},{}", win::describe(top), p.x, p.y));
    }
    mouse::click_at(p)
}

/// Process `pid`'s window called `title` while it shows; windows of other programs never count.
pub fn shown_by(pid: u32, title: &str) -> Option<HWND> {
    win::seen_windows()
        .into_iter()
        .find(|&w| win::owner(w).0 == pid && win::title(w) == title)
}
