//! The layout of the app in front for the spike faces, read through the product's layout source (ADR-0014).
//! `Follow` moved to kx-mod-layouts (P3 Task 10). The rows, widths and key caps moved to kx-mod-keyboard's board
//! (Task 11); key names come from the translations (Task 15), so the Windows key-name reading ends.

use kx_module_api::ports::{LayoutId, LayoutSource, Scan};
use kx_platform_windows::{WinLayouts, layouts, window};
use windows::Win32::UI::Input::KeyboardAndMouse::HKL;

/// Spike scan code `code` as the product's `Scan`.
pub fn scan(code: u32) -> Scan {
    Scan::new(code as u16)
}

/// The layout of the app in front, else the first installed one, read through `WinLayouts::foreground`.
pub fn front_layout() -> HKL {
    let id = WinLayouts
        .foreground()
        .or_else(|| WinLayouts.installed().first().copied());
    id.and_then(layouts::hkl).unwrap_or_default()
}

/// The layout of the app in front when it is another program's; `None` for our own window or none,
/// so kx-mod-layouts' `Follow` keeps the shown layout.
pub fn other_front() -> Option<LayoutId> {
    let front = window::foreground();
    let other = !front.is_invalid() && !window::is_ours(front);
    WinLayouts.foreground().filter(|_| other)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_spike_code_is_the_products_scan_code() {
        assert_eq!(scan(0xE035), Scan::new(0xE035));
        assert!(scan(0xE035).is_extended());
    }

    #[test]
    fn the_front_layout_is_an_installed_one() {
        let front = layouts::id(front_layout());
        assert!(WinLayouts.installed().contains(&front), "{front}");
    }
}
