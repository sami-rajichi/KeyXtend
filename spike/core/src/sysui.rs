//! What Windows says about the look: light or dark apps, the accent colour and high contrast.

mod watch;

use windows::UI::ViewManagement::{UIColorType, UISettings};
use windows::Win32::Graphics::Gdi::{
    COLOR_BTNFACE, COLOR_BTNTEXT, COLOR_GRAYTEXT, COLOR_HIGHLIGHT, COLOR_HIGHLIGHTTEXT,
    COLOR_WINDOW, COLOR_WINDOWTEXT, GetSysColor, SYS_COLOR_INDEX,
};
use windows::Win32::System::Registry::{HKEY_CURRENT_USER, RRF_RT_REG_DWORD, RegGetValueW};
use windows::Win32::UI::Accessibility::{HCF_HIGHCONTRASTON, HIGHCONTRASTW, HIGHCONTRASTW_FLAGS};
use windows::Win32::UI::WindowsAndMessaging::{
    SPI_GETCLIENTAREAANIMATION, SPI_GETHIGHCONTRAST, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS,
    SystemParametersInfoW,
};
use windows::core::{BOOL, PCWSTR, w};

use crate::com::Com;
use crate::lookcfg::ModeChoice;
use crate::theme::{Common, Mode, Palette, Rgba, Shadow};
pub use watch::{Watch, watch};

/// Registry key holding the app mode.
const THEME_KEY: PCWSTR = w!("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize");
/// Registry value: 0 means apps use the dark mode.
const LIGHT_VALUE: PCWSTR = w!("AppsUseLightTheme");
/// Bits of one channel in a COLORREF.
const CHANNEL_BITS: u32 = 8;

/// The high-contrast colours Windows asks every app to use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SysColors {
    /// Window background.
    pub window: Rgba,
    /// Text on the window background.
    pub window_text: Rgba,
    /// Selected item background.
    pub highlight: Rgba,
    /// Text on a selected item.
    pub highlight_text: Rgba,
    /// Button face.
    pub button: Rgba,
    /// Text on a button.
    pub button_text: Rgba,
    /// Disabled text.
    pub gray_text: Rgba,
}

/// The accent fills Fluent uses: the darker shade on light, the lighter shade on dark.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Accent {
    /// `AccentDark1`, for light mode.
    pub on_light: Rgba,
    /// `AccentLight2`, for dark mode.
    pub on_dark: Rgba,
}

impl Accent {
    /// The fill for `mode`.
    pub fn fill(&self, mode: Mode) -> Rgba {
        match mode {
            Mode::Light => self.on_light,
            Mode::Dark => self.on_dark,
        }
    }
}

/// Windows' look right now.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SystemLook {
    /// Apps use the dark mode.
    pub dark: bool,
    /// The accent, when it could be read.
    pub accent: Option<Accent>,
    /// The system colours, while high contrast is on.
    pub contrast: Option<SysColors>,
    /// Windows' animation effects are off.
    pub anim_off: bool,
    /// Why a part could not be read, for the status line.
    pub note: Option<String>,
}

/// True when `AppsUseLightTheme` says dark; a missing value is Windows' default, light.
pub fn dark_from(apps_use_light: Option<u32>) -> bool {
    apps_use_light == Some(0)
}

/// A COLORREF (`0x00BBGGRR`) as an opaque colour.
pub fn from_colorref(c: u32) -> Rgba {
    let channel = |i: u32| (c >> (i * CHANNEL_BITS)) as u8;
    Rgba {
        r: channel(0),
        g: channel(1),
        b: channel(2),
        a: u8::MAX,
    }
}

/// The mode to show for `choice`, given Windows' look.
pub fn mode_for(choice: ModeChoice, look: &SystemLook) -> Mode {
    match choice {
        ModeChoice::Light => Mode::Light,
        ModeChoice::Dark => Mode::Dark,
        ModeChoice::Auto if look.dark => Mode::Dark,
        ModeChoice::Auto => Mode::Light,
    }
}

/// True when the high-contrast flags say it is on.
pub fn contrast_on(flags: HIGHCONTRASTW_FLAGS) -> bool {
    (flags & HCF_HIGHCONTRASTON).0 != 0
}

/// A 1 px inner outline in `ink`, since high contrast needs edges.
fn outline(ink: Rgba) -> Shadow {
    Shadow {
        inset: true,
        x: 0.0,
        y: 0.0,
        blur: 0.0,
        spread: 1.0,
        color: ink,
    }
}

/// A palette made only of system colours, with one outline per key; a tint is not a system colour, so hover shows nothing yet.
pub fn contrast_palette(s: &SysColors) -> Palette {
    let (face, ink, sel, sel_ink) = (s.button, s.button_text, s.highlight, s.highlight_text);
    Palette {
        plate: s.window,
        key: face,
        key_mod: face,
        key_act: face,
        enter: sel,
        enter_ink: sel_ink,
        danger: face,
        danger_ink: ink,
        legend: ink,
        legend_mod: ink,
        legend_2: ink,
        legend_act: ink,
        on: sel,
        on_ink: sel_ink,
        lock: sel,
        lock_ink: sel_ink,
        rec: sel,
        rec_ink: sel_ink,
        led_off: face,
        led_on: sel,
        ring: sel,
        badge_bg: sel,
        badge_ink: sel_ink,
        pop_bg: s.window,
        pop_ink: s.window_text,
        pop_muted: s.window_text,
        pop_line: s.window_text,
        pop_hover: sel,
        pop_sel: sel,
        chip_bg: face,
        hover: Rgba { a: 0, ..face },
        key_shadow: vec![outline(ink)],
        window_shadow: Vec::new(),
        skirt: None,
    }
}

/// The shared colours in system colours, keeping the see-through share of the snip layers from `common`.
pub fn contrast_common(s: &SysColors, common: &Common) -> Common {
    let clear = Rgba { a: 0, ..s.button };
    Common {
        press: clear,
        rec_dot: s.highlight,
        pill_hover: clear,
        snip_dim: Rgba {
            a: common.snip_dim.a,
            ..s.window
        },
        snip_edge: s.highlight,
        snip_tag: Rgba {
            a: common.snip_tag.a,
            ..s.window
        },
    }
}

/// `AppsUseLightTheme`, if it is set.
fn apps_use_light() -> Option<u32> {
    let (mut v, mut n) = (0u32, size_of::<u32>() as u32);
    let data = Some(std::ptr::from_mut(&mut v).cast());
    // SAFETY: reads one DWORD into `v`, whose size is passed in `n`.
    let r = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            THEME_KEY,
            LIGHT_VALUE,
            RRF_RT_REG_DWORD,
            None,
            data,
            Some(&mut n),
        )
    };
    r.is_ok().then_some(v)
}

/// True while high contrast is on.
fn high_contrast() -> bool {
    let mut hc = HIGHCONTRASTW {
        cbSize: size_of::<HIGHCONTRASTW>() as u32,
        ..Default::default()
    };
    let data = Some(std::ptr::from_mut(&mut hc).cast());
    let none = SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0);
    // SAFETY: fills `hc`, whose size is in `cbSize`.
    let ok = unsafe { SystemParametersInfoW(SPI_GETHIGHCONTRAST, hc.cbSize, data, none) }.is_ok();
    ok && contrast_on(hc.dwFlags)
}

/// True when Windows' animation effects are off; a failed read counts as on, Windows' default.
fn animations_off() -> bool {
    let mut on = BOOL(1);
    let data = Some(std::ptr::from_mut(&mut on).cast());
    let none = SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0);
    // SAFETY: fills one BOOL, which is what this query writes.
    let ok = unsafe { SystemParametersInfoW(SPI_GETCLIENTAREAANIMATION, 0, data, none) }.is_ok();
    ok && !on.as_bool()
}

/// The system colours now.
fn sys_colors() -> SysColors {
    // SAFETY: plain query of a system colour.
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

/// The accent shades from WinRT's `UISettings`.
fn accent() -> Result<Accent, String> {
    let _com = Com::start()?;
    let ui = UISettings::new().map_err(|e| e.to_string())?;
    let shade = |t: UIColorType| {
        let c = ui.GetColorValue(t).map_err(|e| e.to_string())?;
        Ok::<_, String>(Rgba {
            r: c.R,
            g: c.G,
            b: c.B,
            a: c.A,
        })
    };
    Ok(Accent {
        on_light: shade(UIColorType::AccentDark1)?,
        on_dark: shade(UIColorType::AccentLight2)?,
    })
}

/// Reads Windows' look now.
pub fn read() -> SystemLook {
    let accent = accent();
    SystemLook {
        dark: dark_from(apps_use_light()),
        note: accent.as_ref().err().map(|e| format!("accent: {e}")),
        accent: accent.ok(),
        contrast: high_contrast().then(sys_colors),
        anim_off: animations_off(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{self, Rgba};

    fn rgb(s: &str) -> Rgba {
        Rgba::parse(s).expect("a colour")
    }

    fn sys() -> SysColors {
        SysColors {
            window: rgb("#000000"),
            window_text: rgb("#FFFFFF"),
            highlight: rgb("#1AEBFF"),
            highlight_text: rgb("#000000"),
            button: rgb("#101010"),
            button_text: rgb("#FFFF00"),
            gray_text: rgb("#3FF23F"),
        }
    }

    #[test]
    fn apps_use_light_zero_means_dark_and_a_missing_value_means_light() {
        assert!(dark_from(Some(0)));
        assert!(!dark_from(Some(1)));
        assert!(!dark_from(None), "Windows' default is light");
    }

    #[test]
    fn the_high_contrast_flag_alone_turns_it_on() {
        assert!(contrast_on(HCF_HIGHCONTRASTON));
        assert!(!contrast_on(HIGHCONTRASTW_FLAGS(0)));
    }

    #[test]
    fn a_colorref_is_read_as_blue_green_red() {
        assert_eq!(from_colorref(0x0033_2211), rgb("#112233"));
    }

    #[test]
    fn auto_follows_windows_and_a_fixed_mode_does_not() {
        let dark = SystemLook {
            dark: true,
            ..SystemLook::default()
        };
        assert_eq!(mode_for(ModeChoice::Auto, &dark), theme::Mode::Dark);
        assert_eq!(
            mode_for(ModeChoice::Auto, &SystemLook::default()),
            theme::Mode::Light
        );
        assert_eq!(mode_for(ModeChoice::Light, &dark), theme::Mode::Light);
    }

    #[test]
    fn high_contrast_uses_only_system_colours() {
        let s = sys();
        let p = contrast_palette(&s);
        let allowed = [
            s.window,
            s.window_text,
            s.highlight,
            s.highlight_text,
            s.button,
            s.button_text,
            s.gray_text,
        ];
        // Only the see-through hover changes alpha, so compare the colour alone.
        let rgb = |c: &Rgba| (c.r, c.g, c.b);
        let system = |c: &Rgba| allowed.iter().any(|a| rgb(a) == rgb(c));
        assert!(p.named().iter().all(|(_, c)| system(c)), "{p:?}");
        assert_eq!(p.hover.a, 0, "no tints in high contrast yet");
        assert!(p.skirt.is_none() && p.window_shadow.is_empty());
        assert_eq!(
            p.key_shadow.len(),
            1,
            "one outline, as high contrast needs edges"
        );
        assert_eq!(p.key_shadow[0].color, s.button_text);
    }

    #[test]
    fn shared_colours_turn_to_system_colours_and_keep_their_see_through_share() {
        let s = sys();
        let base = crate::theme::load().expect("themes.toml loads").common;
        let c = contrast_common(&s, &base);
        let allowed = [s.window, s.highlight, s.button];
        let rgb = |c: &Rgba| (c.r, c.g, c.b);
        for (n, v) in c.named() {
            assert!(allowed.iter().any(|a| rgb(a) == rgb(&v)), "{n}");
        }
        assert_eq!((c.press.a, c.pill_hover.a), (0, 0), "no tints");
        assert_eq!(
            (c.snip_dim.a, c.snip_tag.a),
            (base.snip_dim.a, base.snip_tag.a)
        );
    }
}
