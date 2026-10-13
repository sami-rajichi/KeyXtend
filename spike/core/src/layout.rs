//! The layout of the app in front, which picks voice's spoken language (P11), read through the product's
//! layout source (ADR-0014). The keyboard's own layout reading moved to the product in P3.

use kx_module_api::ports::LayoutSource;
use kx_platform_windows::{WinLayouts, layouts};
use windows::Win32::UI::Input::KeyboardAndMouse::HKL;

/// The layout of the app in front, else the first installed one, read through `WinLayouts::foreground`.
pub fn front_layout() -> HKL {
    let id = WinLayouts
        .foreground()
        .or_else(|| WinLayouts.installed().first().copied());
    id.and_then(layouts::hkl).unwrap_or_default()
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
