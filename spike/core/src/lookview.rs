//! The look a face draws now: the chosen theme in the right mode, with the Windows accent or high contrast applied.

use serde::Serialize;

use crate::lookcfg::{LookConfig, ModeChoice};
use crate::sysui::{self, SystemLook};
use crate::theme::{Common, Dpad, Look, Mode, Motion, Palette, Shadow, Shape, Themes, with_accent};

/// Everything a face needs to paint one frame of the look.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LookView<'a> {
    /// Colours, after the accent or high contrast.
    pub palette: Palette,
    /// Colours every theme shares, as system colours in high contrast.
    pub common: Common,
    /// Shape and type of the theme.
    pub look: &'a Look,
    /// Sizes all themes share.
    pub shape: &'a Shape,
    /// Animations; every length is 0 with reduced motion.
    pub motion: &'a Motion,
    /// The D-pad's colours.
    pub dpad: Dpad,
    /// The mode shown.
    pub dark: bool,
    /// High contrast is on, so effects and glows are off.
    pub contrast: bool,
    /// Room the window shadow needs around the plate, at size 1.
    pub margin: f32,
}

/// How far `layers` reach past the shape they sit under.
pub fn reach(layers: &[Shadow]) -> f32 {
    let far = |s: &Shadow| s.x.abs().max(s.y.abs()) + s.blur + s.spread;
    layers
        .iter()
        .filter(|s| !s.inset)
        .map(far)
        .fold(0.0, f32::max)
        .ceil()
}

/// The palette for theme `i` in `mode`: system colours in high contrast, else the theme's, with the accent if it follows it.
fn palette(
    t: &Themes,
    i: usize,
    mode: Mode,
    sys: &SystemLook,
    cfg: &LookConfig,
) -> Option<Palette> {
    let theme = t.list.get(i)?;
    if let Some(c) = &sys.contrast {
        return Some(sysui::contrast_palette(c));
    }
    let base = theme.palette(mode);
    let follows = cfg.accent_themes.contains(&theme.look.id);
    Some(match sys.accent.filter(|_| follows) {
        Some(a) => with_accent(base, a.fill(mode), cfg.min_contrast),
        None => base.clone(),
    })
}

/// Theme `i` as it should look for mode `choice` while Windows looks like `sys`; `None` for a theme that does not exist.
pub fn look_view<'a>(
    t: &'a Themes,
    i: usize,
    choice: ModeChoice,
    sys: &SystemLook,
    cfg: &LookConfig,
) -> Option<LookView<'a>> {
    let mode = sysui::mode_for(choice, sys);
    let palette = palette(t, i, mode, sys, cfg)?;
    let look = &t.list.get(i)?.look;
    let dpad = look.dpad.resolve(&palette).ok()?;
    let common = match &sys.contrast {
        Some(c) => sysui::contrast_common(c, &t.common),
        None => t.common,
    };
    Some(LookView {
        margin: reach(&palette.window_shadow),
        palette,
        common,
        look,
        shape: &t.shape,
        motion: t.motion.pick(cfg.reduced_motion || sys.anim_off),
        dpad,
        dark: mode == Mode::Dark,
        contrast: sys.contrast.is_some(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sysui::{Accent, SysColors};
    use crate::theme::{self, Rgba};

    fn rgb(s: &str) -> Rgba {
        Rgba::parse(s).expect("a colour")
    }

    fn setup() -> (Themes, LookConfig) {
        let t = theme::load().expect("themes.toml loads");
        (t, crate::config::load().expect("spike.toml loads").look)
    }

    fn teal() -> SystemLook {
        let accent = Accent {
            on_light: rgb("#1D6978"),
            on_dark: rgb("#71D4DB"),
        };
        SystemLook {
            accent: Some(accent),
            ..SystemLook::default()
        }
    }

    #[test]
    fn by_default_native_keeps_its_own_blue() {
        let (t, cfg) = setup();
        let light = look_view(&t, 0, ModeChoice::Light, &teal(), &cfg).expect("light");
        assert_eq!(light.palette.enter, t.list[0].light.enter);
        let dark = look_view(&t, 0, ModeChoice::Dark, &teal(), &cfg).expect("dark");
        assert_eq!(dark.palette.on, t.list[0].dark.on);
    }

    #[test]
    fn only_listed_themes_follow_the_windows_accent() {
        let (t, mut cfg) = setup();
        cfg.accent_themes = vec!["native".into()];
        let native = look_view(&t, 0, ModeChoice::Light, &teal(), &cfg).expect("native");
        assert_eq!(native.palette.enter, rgb("#1D6978"));
        let et66 = look_view(&t, 1, ModeChoice::Light, &teal(), &cfg).expect("et66");
        assert_eq!(
            et66.palette.enter, t.list[1].light.enter,
            "ET66 keeps its yellow"
        );
    }

    #[test]
    fn auto_follows_windows_and_dark_uses_the_light_accent_shade() {
        let (t, mut cfg) = setup();
        cfg.accent_themes = vec!["native".into()];
        let sys = SystemLook {
            dark: true,
            ..teal()
        };
        let v = look_view(&t, 0, ModeChoice::Auto, &sys, &cfg).expect("native");
        assert!(v.dark);
        assert_eq!(v.palette.on, rgb("#71D4DB"));
    }

    #[test]
    fn high_contrast_replaces_every_theme_colour() {
        let (t, cfg) = setup();
        let black = rgb("#000000");
        let colours = SysColors {
            window: black,
            window_text: rgb("#FFFFFF"),
            highlight: rgb("#1AEBFF"),
            highlight_text: black,
            button: black,
            button_text: rgb("#FFFF00"),
            gray_text: rgb("#3FF23F"),
        };
        let sys = SystemLook {
            contrast: Some(colours),
            ..teal()
        };
        let v = look_view(&t, 2, ModeChoice::Light, &sys, &cfg).expect("dolch");
        assert!(v.contrast && v.palette.skirt.is_none());
        assert_eq!(
            v.dpad.face, black,
            "Dolch's grey arrows become the button colour"
        );
        assert_eq!(v.common.snip_edge, colours.highlight, "shared colours too");
        let plain = look_view(&t, 2, ModeChoice::Light, &teal(), &cfg).expect("dolch");
        assert_eq!(plain.common, t.common);
    }

    #[test]
    fn reduced_motion_or_windows_animations_off_zero_every_duration() {
        let (t, mut cfg) = setup();
        let off = SystemLook {
            anim_off: true,
            ..SystemLook::default()
        };
        let press = |sys: &SystemLook, cfg: &LookConfig| {
            let v = look_view(&t, 0, ModeChoice::Light, sys, cfg).expect("native");
            v.motion.moves["press"].ms
        };
        let full = t.motion.full.moves["press"].ms;
        assert!(full > 0 && press(&SystemLook::default(), &cfg) == full);
        assert_eq!(press(&off, &cfg), 0, "Windows' animation effects are off");
        cfg.reduced_motion = true;
        assert_eq!(
            press(&SystemLook::default(), &cfg),
            0,
            "the reduced motion setting"
        );
    }

    #[test]
    fn the_margin_fits_the_widest_outer_shadow() {
        let (t, cfg) = setup();
        let v = look_view(&t, 0, ModeChoice::Light, &SystemLook::default(), &cfg).expect("native");
        assert!(
            (v.margin - 44.0).abs() < f32::EPSILON,
            "12 down plus 32 blur: {}",
            v.margin
        );
        assert!(look_view(&t, 9, ModeChoice::Light, &SystemLook::default(), &cfg).is_none());
    }
}
