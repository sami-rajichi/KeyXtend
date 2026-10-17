//! The page mark (ADR-0029): our windows that page, as the open list does, which the scroll target takes, while it
//! passes over our others. The UI names them by title; a mark goes with its window.

use kx_module_api::ports::Point;
use kx_platform::ui::UiError;
use windows::Win32::Foundation::HWND;
use windows::core::{PCWSTR, w};

use super::entry::watched;
use super::guard::guarded;
use super::prop::{clear, put, value};
use super::win::{is_ours, own_titled, window_at};
use crate::failed;
use crate::scroll::Whose;

/// The window property that marks our window that pages.
const PAGES: PCWSTR = w!("KeyXtend.Pages");
/// What `PAGES` holds.
const MARK: usize = 1;

/// Marks our shown window titled `title` as one that pages.
///
/// # Errors
/// `NoWindow` while it is not shown, `Refused` for a window the guard does not hold, or when Windows says no.
pub(super) fn mark_pages(title: &str) -> Result<(), UiError> {
    mark(own_titled(title).ok_or(UiError::NoWindow)?)
}

/// Marks our guarded window `hwnd` as one that pages.
///
/// # Errors
/// `Refused` for a window the guard does not hold, or when Windows says no.
pub(super) fn mark(hwnd: HWND) -> Result<(), UiError> {
    if !guarded(hwnd) {
        return Err(failed());
    }
    put(hwnd, PAGES, MARK)
}

/// True when `hwnd` carries the page mark; any thread may ask.
fn pages(hwnd: HWND) -> bool {
    value(hwnd, PAGES) != 0
}

/// Drops the page mark from `hwnd` as it goes.
pub(super) fn forget(hwnd: HWND) {
    clear(hwnd, PAGES);
}

/// Whose top-level window `root` is: the keyboard the guard watches, another of ours that pages or not, or another
/// app's, whose marks count for nothing.
pub(crate) fn whose(root: HWND) -> Whose {
    Whose::of(is_ours(root), watched(root), pages(root))
}

/// True when the window at screen point `p` is one of ours that does not page, which the scroll target passes over.
pub(super) fn passed_at(p: Point) -> bool {
    whose(window_at(p).1).passed()
}
