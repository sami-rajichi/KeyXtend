//! A sample high-contrast set shared by the unit tests and the fixtures test, so there is one copy.

use kx_platform::ui::SysColors;

/// Yellow on black with a cyan highlight and green disabled text, as `[r, g, b, a]`; each role has its own colour.
pub(crate) const HC: SysColors = SysColors {
    window: [0x00, 0x00, 0x00, 0xFF],
    window_text: [0xFF, 0xFF, 0xFF, 0xFF],
    highlight: [0x1A, 0xEB, 0xFF, 0xFF],
    highlight_text: [0x05, 0x05, 0x05, 0xFF],
    button: [0x10, 0x10, 0x10, 0xFF],
    button_text: [0xFF, 0xFF, 0x00, 0xFF],
    gray_text: [0x3F, 0xF2, 0x3F, 0xFF],
};
