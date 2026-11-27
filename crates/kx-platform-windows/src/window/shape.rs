//! The shape that takes clicks: a window cut to its rects, so clicks elsewhere reach the app below.

use kx_module_api::ports::Rect;
use kx_platform::ui::UiError;
use windows::Win32::Graphics::Gdi::{
    CombineRgn, CreateRectRgn, DeleteObject, HRGN, RGN_ERROR, RGN_OR, SetWindowRgn,
};

use super::win::own_titled;
use crate::failed;
use crate::screen::edges;

/// Cuts our window titled `title_is` to `rects` (physical pixels from its top-left corner), so clicks
/// elsewhere reach the app below; Windows also draws nothing outside them.
///
/// # Errors
/// `NoWindow` for a window not shown, `Refused` when Windows says no.
#[allow(unsafe_code, reason = "Hands a region to our own window.")]
pub fn shape(title_is: &str, rects: &[Rect]) -> Result<(), UiError> {
    let hwnd = own_titled(title_is).ok_or(UiError::NoWindow)?;
    let all = region(rects)?;
    // SAFETY: `all` is ours until SetWindowRgn takes it; `hwnd` is one of our live windows.
    if unsafe { SetWindowRgn(hwnd, Some(all), true) } == 0 {
        // SAFETY: Windows did not take the region, so we delete it, once.
        let _ = unsafe { DeleteObject(all.into()) };
        return Err(failed());
    }
    Ok(())
}

/// One region covering every rect; the caller owns it.
#[allow(unsafe_code, reason = "Makes a region and deletes it on failure.")]
pub(super) fn region(rects: &[Rect]) -> Result<HRGN, UiError> {
    // SAFETY: makes an empty region, which the caller owns.
    let all = unsafe { CreateRectRgn(0, 0, 0, 0) };
    if all.is_invalid() {
        return Err(failed());
    }
    for r in rects {
        if let Err(e) = add(all, *r) {
            // SAFETY: deletes the region made above, once.
            let _ = unsafe { DeleteObject(all.into()) };
            return Err(e);
        }
    }
    Ok(all)
}

/// Adds rect `r` to region `all`.
#[allow(unsafe_code, reason = "Makes, joins and deletes one region.")]
fn add(all: HRGN, r: Rect) -> Result<(), UiError> {
    let [left, top, right, bottom] = edges(r);
    // SAFETY: makes one region, deleted below.
    let one = unsafe { CreateRectRgn(left, top, right, bottom) };
    if one.is_invalid() {
        return Err(failed());
    }
    // SAFETY: both regions are ours and live.
    let joined = unsafe { CombineRgn(Some(all), Some(all), Some(one), RGN_OR) };
    let result = if joined == RGN_ERROR {
        Err(failed())
    } else {
        Ok(())
    };
    // SAFETY: deletes the region made above, once.
    let _ = unsafe { DeleteObject(one.into()) };
    result
}
