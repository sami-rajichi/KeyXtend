//! The page mark (ADR-0029): our windows that page, as the open list does, which the scroll target takes, while it
//! passes over our others. The UI names them by title as they open; the guard holds each marked window, so the mark
//! goes with it.

use kx_module_api::ports::Point;
use kx_platform::ui::UiError;
use windows::Win32::Foundation::HWND;
use windows::core::{PCWSTR, w};

use super::entry::watched;
use super::guard::{guard, guarded};
use super::prop::{clear, put, value};
use super::win::{is_ours, own_titled, own_windows, window_at};
use crate::failed;
use crate::scroll::Whose;

/// The window property that marks our window that pages.
const PAGES: PCWSTR = w!("KeyXtend.Pages");
/// What `PAGES` holds.
const MARK: usize = 1;

/// Marks our shown window titled `title` as one that pages, guarding it first if the guard does not hold it yet.
///
/// # Errors
/// `NoWindow` while it is not shown, `Refused` when the guard or Windows says no.
pub(super) fn mark_pages(title: &str) -> Result<(), UiError> {
    mark(own_titled(title).ok_or(UiError::NoWindow)?)
}

/// Marks our window `hwnd` as one that pages; the guard holds it first, so its unhook drops the mark as it goes.
///
/// # Errors
/// `Refused` for another program's window, or when the guard or Windows says no, as from another thread.
pub(super) fn mark(hwnd: HWND) -> Result<(), UiError> {
    if !is_ours(hwnd) {
        return Err(failed());
    }
    if !guarded(hwnd) {
        guard(hwnd)?;
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

/// Our shown window that pages, front first; none while none shows. While one shows, the scroll target aims at it.
pub(crate) fn paging() -> Option<HWND> {
    own_windows()
        .into_iter()
        .find(|&w| whose(w) == Whose::Pages)
}

/// True when the window at screen point `p` is one of ours that does not page, which the scroll target passes over.
pub(super) fn passed_at(p: Point) -> bool {
    whose(window_at(p).1).passed()
}
