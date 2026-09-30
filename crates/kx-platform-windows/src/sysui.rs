//! What Windows says about the look: dark apps, the accent pair, the high-contrast colours, animations and the UI language.
//! Reading Windows' answers is pure and tested everywhere; the queries are in `win`.

use kx_platform::ui::Rgba;

#[cfg(windows)]
mod win;

#[cfg(windows)]
pub(crate) use win::read;
#[cfg(windows)]
pub use win::{WinLook, ui_language};

/// `HCF_HIGHCONTRASTON`: the high-contrast flag that says it is on.
const HCF_HIGHCONTRASTON: u32 = 0x1;

/// True when `AppsUseLightTheme` says dark; a missing value is Windows' default, light.
#[must_use]
pub fn dark_from(apps_use_light: Option<u32>) -> bool {
    apps_use_light == Some(0)
}

/// A COLORREF (`0x00BBGGRR`) as an opaque colour.
#[must_use]
pub fn from_colorref(c: u32) -> Rgba {
    let [red, green, blue, _] = c.to_le_bytes();
    [red, green, blue, u8::MAX]
}

/// True when high-contrast flags `flags` say it is on.
#[must_use]
pub fn contrast_on(flags: u32) -> bool {
    flags & HCF_HIGHCONTRASTON != 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apps_use_light_zero_means_dark_and_a_missing_value_means_light() {
        assert!(dark_from(Some(0)));
        assert!(!dark_from(Some(1)));
        assert!(!dark_from(None), "Windows' default is light");
    }

    #[test]
    fn a_colorref_is_read_as_blue_green_red_and_opaque() {
        assert_eq!(from_colorref(0x0033_2211), [0x11, 0x22, 0x33, u8::MAX]);
        assert_eq!(from_colorref(0), [0, 0, 0, u8::MAX]);
        assert_eq!(
            from_colorref(0xFF00_0000),
            [0, 0, 0, u8::MAX],
            "the top byte is not a colour"
        );
    }

    #[test]
    fn the_high_contrast_flag_alone_turns_it_on() {
        assert!(contrast_on(HCF_HIGHCONTRASTON));
        assert!(contrast_on(HCF_HIGHCONTRASTON | 0x2));
        assert!(!contrast_on(0));
        assert!(!contrast_on(!HCF_HIGHCONTRASTON), "other flags alone");
    }

    #[cfg(windows)]
    #[test]
    fn the_flag_matches_windows() {
        use windows::Win32::UI::Accessibility::HCF_HIGHCONTRASTON as ON;
        assert_eq!(HCF_HIGHCONTRASTON, ON.0);
    }
}
