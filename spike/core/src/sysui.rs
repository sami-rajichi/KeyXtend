//! The mode and high-contrast maths over Windows' look, which the product's `WinLook` reads
//! (`kx_platform_windows::sysui`, ADR-0014); this part moves to `kx-theme` with the look view.

use kx_platform::ui::{self as port, SystemLook as _};
use kx_platform_windows::WinLook;

pub use kx_platform::ui::WatchGuard;

use crate::lookcfg::ModeChoice;
use crate::theme::{Common, Mode, Palette, Rgba, Shadow};

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
}

/// A port colour as a theme colour.
fn rgba([r, g, b, a]: port::Rgba) -> Rgba {
    Rgba { r, g, b, a }
}

/// The product's look in the spike's theme colours.
pub fn from_port(look: port::SysLook) -> SystemLook {
    let accent = look.accent.map(|a| Accent {
        on_light: rgba(a.on_light),
        on_dark: rgba(a.on_dark),
    });
    let contrast = look.contrast.map(|c| SysColors {
        window: rgba(c.window),
        window_text: rgba(c.window_text),
        highlight: rgba(c.highlight),
        highlight_text: rgba(c.highlight_text),
        button: rgba(c.button),
        button_text: rgba(c.button_text),
        gray_text: rgba(c.gray_text),
    });
    SystemLook {
        dark: look.dark,
        accent,
        contrast,
        anim_off: look.anim_off,
    }
}

/// Reads Windows' look now.
pub fn read() -> SystemLook {
    from_port(WinLook.read())
}

/// Calls `on_change` on the product's watcher thread with each new look, until the guard drops.
pub fn watch(on_change: impl Fn(SystemLook) + Send + 'static) -> Result<WatchGuard, String> {
    let forward = move |look| on_change(from_port(look));
    WinLook.watch(Box::new(forward)).map_err(|e| e.to_string())
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
    fn the_product_look_keeps_every_colour_and_flag() {
        let colour = |n: u8| [n, n + 1, n + 2, u8::MAX];
        let look = port::SysLook {
            dark: true,
            accent: Some(port::Accent {
                on_light: colour(1),
                on_dark: colour(4),
            }),
            contrast: Some(port::SysColors {
                window: colour(10),
                window_text: colour(20),
                highlight: colour(30),
                highlight_text: colour(40),
                button: colour(50),
                button_text: colour(60),
                gray_text: colour(70),
            }),
            anim_off: true,
        };
        let spike = from_port(look);
        assert!(spike.dark && spike.anim_off);
        let accent = spike.accent.expect("an accent");
        assert_eq!(accent.on_dark, rgb("#040506"));
        let c = spike.contrast.expect("contrast colours");
        assert_eq!((c.window, c.gray_text), (rgb("#0A0B0C"), rgb("#464748")));
        assert_eq!(from_port(port::SysLook::default()), SystemLook::default());
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
