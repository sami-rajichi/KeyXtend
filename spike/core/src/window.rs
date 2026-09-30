//! Spike glue over the product's window guard (`kx_platform_windows::window`, ADR-0014): the spared Settings
//! title, errors as text for the face, and quick-fill's focus request, the one Win32 call made here.

use std::sync::OnceLock;

use kx_module_api::ports::Rect;
use kx_platform::ui::UiError;
use kx_platform_windows::window::{self, foreground};
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::SetForegroundWindow;

/// Our one window that may take focus, as Settings does; the guard leaves it alone.
static SPARED: OnceLock<String> = OnceLock::new();

/// Lets our window `title_is` take focus: no guard touches it. Only the first call counts.
pub fn spare(title_is: &str) {
    let _ = SPARED.set(title_is.to_string());
}

/// Guards every visible window of this process except the spared one; returns how many.
pub fn guard_own_windows() -> Result<usize, String> {
    window::guard_own(SPARED.get().map(String::as_str), None).map_err(|e| e.to_string())
}

/// Guards every visible window of this process and puts the one called `top` above the others.
/// False when `top` is not visible yet: a toolkit may show a window a moment after it was asked to.
pub fn guard_with_top(top: &str) -> Result<bool, String> {
    let found = window::own_titled(top).is_some();
    let spared = SPARED.get().map(String::as_str);
    window::guard_own(spared, Some(top)).map_err(|e| e.to_string())?;
    Ok(found)
}

/// Cuts our visible window `title_is` down to `rects` (left, top, right, bottom in physical pixels), so a click
/// anywhere else reaches the app below; false when it is not visible yet.
pub fn shape(title_is: &str, rects: &[[i32; 4]]) -> Result<bool, String> {
    let rects: Vec<Rect> = rects
        .iter()
        .map(|&e| kx_platform_windows::screen::rect_of(e))
        .collect();
    match window::shape(title_is, &rects) {
        Ok(()) => Ok(true),
        Err(UiError::NoWindow) => Ok(false),
        Err(e) => Err(e.to_string()),
    }
}

/// Brings `target` back to the front if something else took it; true when it is in front after `settle_ms`.
/// Spike only (quick-fill, P9): the product never asks Windows for the focus, so this call stays here.
pub fn bring_back(target: HWND, settle_ms: u64) -> bool {
    if foreground() != target {
        // SAFETY: a plain request Windows may refuse; the check below sees the result.
        let _ = unsafe { SetForegroundWindow(target) };
    }
    std::thread::sleep(std::time::Duration::from_millis(settle_ms));
    foreground() == target
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_window_that_is_not_shown_yet_is_not_shaped() {
        assert_eq!(
            shape("spike test: no such window", &[[0, 0, 10, 10]]),
            Ok(false)
        );
    }

    #[test]
    fn the_spike_program_shows_no_window_to_guard() {
        assert_eq!(guard_own_windows(), Ok(0));
        assert_eq!(guard_with_top("spike test: no such window"), Ok(false));
    }
}
