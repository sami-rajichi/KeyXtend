//! The Windows side of the look: the app mode in the registry, the accent from `UISettings`, the system
//! colours and parameters, and the UI language.

use kx_platform::ui::{Accent, Rgba, SysColors, SysLook, SystemLook, UiError, WatchGuard};
use windows::UI::ViewManagement::{UIColorType, UISettings};
use windows::Win32::Foundation::ERROR_SUCCESS;
use windows::Win32::Globalization::GetUserDefaultUILanguage;
use windows::Win32::Graphics::Gdi::{
    COLOR_BTNFACE, COLOR_BTNTEXT, COLOR_GRAYTEXT, COLOR_HIGHLIGHT, COLOR_HIGHLIGHTTEXT,
    COLOR_WINDOW, COLOR_WINDOWTEXT, GetSysColor, SYS_COLOR_INDEX,
};
use windows::Win32::System::Registry::{HKEY_CURRENT_USER, RRF_RT_REG_DWORD, RegGetValueW};
use windows::Win32::UI::Accessibility::HIGHCONTRASTW;
use windows::Win32::UI::WindowsAndMessaging::{
    SPI_GETCLIENTAREAANIMATION, SPI_GETHIGHCONTRAST, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS,
    SystemParametersInfoW,
};
use windows::core::{BOOL, PCWSTR, w};

use super::{contrast_on, dark_from, from_colorref};
use crate::com::Com;
use crate::{langinfo, watch};

/// Registry key of the app mode.
const THEME_KEY: PCWSTR = w!(r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize");
/// Registry value: 0 means apps use the dark mode.
const LIGHT_VALUE: PCWSTR = w!("AppsUseLightTheme");

/// The Windows system look and UI language.
#[derive(Clone, Copy, Debug, Default)]
pub struct WinLook;

impl SystemLook for WinLook {
    fn read(&self) -> SysLook {
        read()
    }

    /// Starts a watcher thread; `on_change` runs on it, once per real change, until the guard drops.
    fn watch(&self, on_change: Box<dyn Fn(SysLook) + Send>) -> Result<WatchGuard, UiError> {
        watch::start(on_change)
    }

    fn ui_language(&self) -> String {
        ui_language()
    }
}

/// Reads the look now.
#[must_use]
pub(crate) fn read() -> SysLook {
    SysLook {
        dark: dark_from(apps_use_light()),
        accent: accent(),
        contrast: high_contrast().then(sys_colors),
        anim_off: animations_off(),
    }
}

/// The Windows display language as a BCP-47 tag, such as `fr-FR`; empty when Windows cannot say.
#[must_use]
#[allow(unsafe_code, reason = "A plain language query.")]
pub fn ui_language() -> String {
    // SAFETY: a plain query with no arguments.
    let lang = unsafe { GetUserDefaultUILanguage() };
    langinfo::lang_name(lang).unwrap_or_default()
}

/// `AppsUseLightTheme`, when it is set.
#[allow(unsafe_code, reason = "Reads one registry number into a local.")]
fn apps_use_light() -> Option<u32> {
    let mut value = 0u32;
    let mut size = u32::try_from(size_of::<u32>()).ok()?;
    let data = Some(std::ptr::from_mut(&mut value).cast());
    // SAFETY: Windows writes at most `size` bytes into `value`, and the size back.
    let r = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            THEME_KEY,
            LIGHT_VALUE,
            RRF_RT_REG_DWORD,
            None,
            data,
            Some(&raw mut size),
        )
    };
    (r == ERROR_SUCCESS).then_some(value)
}

/// True while high contrast is on.
#[allow(unsafe_code, reason = "Fills a local struct whose size it passes.")]
fn high_contrast() -> bool {
    let Ok(size) = u32::try_from(size_of::<HIGHCONTRASTW>()) else {
        return false;
    };
    let mut hc = HIGHCONTRASTW {
        cbSize: size,
        ..HIGHCONTRASTW::default()
    };
    let data = Some(std::ptr::from_mut(&mut hc).cast());
    let keep = SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS::default();
    // SAFETY: fills `hc`, whose size is in `cbSize` and in the call.
    let ok = unsafe { SystemParametersInfoW(SPI_GETHIGHCONTRAST, size, data, keep) }.is_ok();
    ok && contrast_on(hc.dwFlags.0)
}

/// True when Windows' animation effects are off; a failed read counts as on, Windows' default.
#[allow(unsafe_code, reason = "Fills one local BOOL.")]
fn animations_off() -> bool {
    let mut on = BOOL::from(true);
    let data = Some(std::ptr::from_mut(&mut on).cast());
    let keep = SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS::default();
    // SAFETY: this query writes one BOOL, which is what `on` is.
    let ok = unsafe { SystemParametersInfoW(SPI_GETCLIENTAREAANIMATION, 0, data, keep) }.is_ok();
    ok && !on.as_bool()
}

/// The seven system colours now.
#[allow(unsafe_code, reason = "Plain system colour queries.")]
fn sys_colors() -> SysColors {
    // SAFETY: a plain query of one system colour.
    let get = |i: SYS_COLOR_INDEX| from_colorref(unsafe { GetSysColor(i) });
    SysColors {
        window: get(COLOR_WINDOW),
        window_text: get(COLOR_WINDOWTEXT),
        highlight: get(COLOR_HIGHLIGHT),
        highlight_text: get(COLOR_HIGHLIGHTTEXT),
        button: get(COLOR_BTNFACE),
        button_text: get(COLOR_BTNTEXT),
        gray_text: get(COLOR_GRAYTEXT),
    }
}

/// The accent fills Fluent uses, from `UISettings`: `AccentDark1` on light and `AccentLight2` on dark.
fn accent() -> Option<Accent> {
    // windows-rs caches WinRT factories for the whole process, so COM stays on for this thread.
    Com::start().ok()?.keep();
    let ui = UISettings::new().ok()?;
    let shade = |t: UIColorType| -> Option<Rgba> {
        let c = ui.GetColorValue(t).ok()?;
        Some([c.R, c.G, c.B, c.A])
    };
    Some(Accent {
        on_light: shade(UIColorType::AccentDark1)?,
        on_dark: shade(UIColorType::AccentLight2)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_look_reads_an_accent_and_opaque_system_colours() {
        let look = WinLook.read();
        assert!(look.accent.is_some(), "Windows 10 and 11 have an accent");
        let colours = sys_colors();
        let all = [colours.window, colours.window_text, colours.button_text];
        assert!(all.iter().all(|c| c[3] == u8::MAX), "{colours:?}");
    }

    #[test]
    fn the_look_reads_the_same_on_another_thread() {
        let here = WinLook.read();
        let there = std::thread::spawn(|| WinLook.read()).join().unwrap();
        assert_eq!(here, there);
    }

    #[test]
    fn the_ui_language_is_a_bcp_47_tag() {
        let tag = WinLook.ui_language();
        assert!(!tag.is_empty(), "Windows names its display language");
        let allowed = |c: char| c.is_ascii_alphanumeric() || c == '-';
        assert!(tag.chars().all(allowed), "{tag}");
    }
}
