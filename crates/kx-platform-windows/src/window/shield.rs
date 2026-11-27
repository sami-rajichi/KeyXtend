//! The shield (ADR-0056 decision 8): our windows kept out of every screen capture by title, shown or not, so one
//! shielded before it first shows is in no capture.

use kx_platform::ui::UiError;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{
    GetWindowDisplayAffinity, SetWindowDisplayAffinity, WDA_EXCLUDEFROMCAPTURE, WDA_NONE,
    WINDOW_DISPLAY_AFFINITY,
};

use super::win::{is_ours, title, top_level};
use crate::{failed, refused};

/// Keeps our windows titled `title_is`, shown or not, out of every capture while `on`, or lets them back in.
///
/// # Errors
/// `NoWindow` without such a window, `Refused` when Windows will not set the shield or does not keep it; then every
/// such window is let back in, so none stays half shielded.
pub fn shield(title_is: &str, on: bool) -> Result<(), UiError> {
    let mut ours = top_level();
    ours.retain(|&w| is_ours(w) && title(w) == title_is);
    if ours.is_empty() {
        return Err(UiError::NoWindow);
    }
    all_or_none(&ours, on, set)
}

/// Shields each of `ours` while `on`, or lets it in, through `set`; when one refuses, lets every one back in, then says
/// why with the first refusal.
pub(super) fn all_or_none<W: Copy>(
    ours: &[W],
    on: bool,
    mut set: impl FnMut(W, WINDOW_DISPLAY_AFFINITY) -> Result<(), UiError>,
) -> Result<(), UiError> {
    let affinity = if on { WDA_EXCLUDEFROMCAPTURE } else { WDA_NONE };
    let all = ours.iter().try_for_each(|&w| set(w, affinity));
    if all.is_err() {
        for &w in ours {
            let _ = set(w, WDA_NONE);
        }
    }
    all
}

/// Sets `affinity` on our window `hwnd` and reads it back, as an older Windows may keep another one.
#[allow(
    unsafe_code,
    reason = "Sets and reads our own window's capture affinity."
)]
fn set(hwnd: HWND, affinity: WINDOW_DISPLAY_AFFINITY) -> Result<(), UiError> {
    // SAFETY: one of this process's top-level windows, as the call needs.
    unsafe { SetWindowDisplayAffinity(hwnd, affinity) }.map_err(|e| refused(&e))?;
    let mut kept = 0;
    // SAFETY: a plain query of the same window into a local.
    unsafe { GetWindowDisplayAffinity(hwnd, &raw mut kept) }.map_err(|e| refused(&e))?;
    (kept == affinity.0).then_some(()).ok_or_else(failed)
}
