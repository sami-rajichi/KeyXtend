//! Spike glue over the product's windows (`kx_platform_windows::window`, ADR-0014) for G7: the focus request, the
//! one Win32 call here.

use kx_platform_windows::window::foreground;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::SetForegroundWindow;

/// Brings `target` back to the front if something else took it; true when it is in front after `settle_ms`.
/// Spike only: G7 brings the test panel forward with it, as the product never asks Windows for the focus.
pub fn bring_back(target: HWND, settle_ms: u64) -> bool {
    if foreground() != target {
        // SAFETY: a plain request Windows may refuse; the check below sees the result.
        let _ = unsafe { SetForegroundWindow(target) };
    }
    std::thread::sleep(std::time::Duration::from_millis(settle_ms));
    foreground() == target
}
