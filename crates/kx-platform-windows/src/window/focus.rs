//! The focus: which window is in front, the token that notes it, and giving it back when one of ours took it.

use kx_platform::ui::FocusToken;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, IsWindow, SetForegroundWindow};

use super::should_give_back;
use super::win::is_ours;

/// `hwnd` as a focus token.
pub(super) fn token_of(hwnd: HWND) -> FocusToken {
    FocusToken::new(u64::try_from(hwnd.0.addr()).unwrap_or_default())
}

/// The window `token` names; `None` when it does not fit a handle.
pub(super) fn window_of(token: FocusToken) -> Option<HWND> {
    let addr = usize::try_from(token.raw()).ok()?;
    Some(HWND(std::ptr::without_provenance_mut(addr)))
}

/// The window in front now; null when there is none.
#[must_use]
#[allow(unsafe_code, reason = "A plain window query.")]
pub fn foreground() -> HWND {
    // SAFETY: a plain query.
    unsafe { GetForegroundWindow() }
}

/// Gives the focus back to `noted` when one of ours took it and `noted` is another app's live window.
#[allow(unsafe_code, reason = "A focus change Windows allows us.")]
pub fn give_back(noted: HWND) {
    if gives_back(noted) {
        // SAFETY: one of our windows is in front, so Windows allows the change.
        let _ = unsafe { SetForegroundWindow(noted) };
    }
}

/// True when the focus should go back to `noted`: one of ours is in front and `noted` is another app's live window.
#[allow(unsafe_code, reason = "A plain window query.")]
pub(super) fn gives_back(noted: HWND) -> bool {
    // SAFETY: a plain query; a null or gone window reads as not a window.
    let alive = unsafe { IsWindow(Some(noted)) }.as_bool();
    should_give_back(alive, is_ours(foreground()), is_ours(noted))
}
