//! The layout of the app in front for the spike faces, read through the product's layout source (ADR-0014).
//! `Follow` moved to kx-mod-layouts (P3 Task 10). The rows, widths and key caps moved to kx-mod-keyboard's board
//! (Task 11); key names come from the translations (Task 15), so the Windows key-name reading ends.

use kx_module_api::ports::{KeyChars, LangFacts, LayoutId, LayoutSource, Scan, SwitchError};
use kx_platform_windows::{WinLayouts, layouts, window};
use windows::Win32::UI::Input::KeyboardAndMouse::HKL;

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

/// `WinLayouts` seen as `relabel` sees it: our own window in front counts as none, so a typing click
/// checks the shown table against the same app that `Follow` follows.
pub struct OtherFront;

impl LayoutSource for OtherFront {
    fn installed(&self) -> Vec<LayoutId> {
        WinLayouts.installed()
    }

    fn foreground(&self) -> Option<LayoutId> {
        other_front()
    }

    fn keys(&self, layout: LayoutId, scans: &[Scan]) -> Vec<KeyChars> {
        WinLayouts.keys(layout, scans)
    }

    fn facts(&self, layout: LayoutId) -> LangFacts {
        WinLayouts.facts(layout)
    }

    fn caps_on(&self) -> bool {
        WinLayouts.caps_on()
    }

    fn types_any(&self, layout: LayoutId, key: Scan, held: &[Scan]) -> bool {
        WinLayouts.types_any(layout, key, held)
    }

    fn request(&self, layout: LayoutId) -> Result<(), SwitchError> {
        WinLayouts.request(layout)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_front_layout_is_an_installed_one() {
        let front = layouts::id(front_layout());
        assert!(WinLayouts.installed().contains(&front), "{front}");
    }
}
