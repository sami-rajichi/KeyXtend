//! Look-view tests against the shipped tokens and sample system looks.

use kx_platform::ui::{Accent, SysColors};

use super::*;

/// A shipped colour as the system gives colours.
fn port(s: &str) -> [u8; 4] {
    let c = Rgba::parse(s).expect("a colour");
    [c.r, c.g, c.b, c.a]
}

fn rgb(s: &str) -> Rgba {
    Rgba::parse(s).expect("a colour")
}

fn themes() -> Themes {
    Themes::shipped().expect("tokens load")
}

/// Theme `id` in `mode`, keeping its own colours, with full motion.
fn choice(id: &str, mode: ModeChoice) -> LookChoice {
    LookChoice {
        theme: id.into(),
        mode,
        follow_accent: false,
        reduced_motion: false,
        min_contrast: WCAG_AA,
    }
}

fn teal() -> SysLook {
    let accent = Accent {
        on_light: port("#1D6978"),
        on_dark: port("#71D4DB"),
    };
    SysLook {
        accent: Some(accent),
        ..SysLook::default()
    }
}

/// A sample high-contrast set: yellow on black, cyan highlight.
fn hc_colours() -> SysColors {
    let black = port("#000000");
    SysColors {
        window: black,
        window_text: port("#FFFFFF"),
        highlight: port("#1AEBFF"),
        highlight_text: black,
        button: black,
        button_text: port("#FFFF00"),
        gray_text: port("#3FF23F"),
    }
}

#[test]
fn by_default_native_keeps_its_own_blue() {
    let t = themes();
    let light = look_view(&t, &choice("native", ModeChoice::Light), &teal()).expect("light");
    assert_eq!(light.palette.enter, t.list[0].light.enter);
    let dark = look_view(&t, &choice("native", ModeChoice::Dark), &teal()).expect("dark");
    assert_eq!(dark.palette.on, t.list[0].dark.on);
}

#[test]
fn only_themes_that_take_the_accent_follow_it() {
    let t = themes();
    let follow = |id: &str| LookChoice {
        follow_accent: true,
        ..choice(id, ModeChoice::Light)
    };
    let native = look_view(&t, &follow("native"), &teal()).expect("native");
    assert_eq!(native.palette.enter, rgb("#1D6978"));
    let et66 = look_view(&t, &follow("et66"), &teal()).expect("et66");
    assert_eq!(
        et66.palette.enter, t.list[1].light.enter,
        "ET66 keeps its yellow"
    );
    let dolch = look_view(&t, &follow("dolch"), &teal()).expect("dolch");
    assert_eq!(
        dolch.palette.on, t.list[2].light.on,
        "skirts keep their colour"
    );
}

#[test]
fn auto_follows_the_system_and_dark_uses_the_light_accent_shade() {
    let t = themes();
    let sys = SysLook {
        dark: true,
        ..teal()
    };
    let native = LookChoice {
        follow_accent: true,
        ..choice("native", ModeChoice::Auto)
    };
    let v = look_view(&t, &native, &sys).expect("native");
    assert!(v.dark);
    assert_eq!(v.palette.on, rgb("#71D4DB"));
}

#[test]
fn a_setting_below_wcag_aa_still_keeps_legends_at_aa() {
    let t = themes();
    let yellow = SysLook {
        accent: Some(Accent {
            on_light: port("#FFD800"),
            on_dark: port("#FFD800"),
        }),
        ..SysLook::default()
    };
    for min in [1.0, f32::NAN] {
        let native = LookChoice {
            follow_accent: true,
            min_contrast: min,
            ..choice("native", ModeChoice::Light)
        };
        let v = look_view(&t, &native, &yellow).expect("native");
        let c = crate::contrast::contrast(v.palette.enter_ink, v.palette.enter);
        assert!(c >= WCAG_AA, "{min}: {c}");
    }
}

#[test]
fn high_contrast_replaces_every_theme_colour() {
    let t = themes();
    let colours = hc_colours();
    let sys = SysLook {
        contrast: Some(colours),
        ..teal()
    };
    let v = look_view(&t, &choice("dolch", ModeChoice::Light), &sys).expect("dolch");
    assert!(v.contrast && v.palette.skirt.is_none());
    assert_eq!(
        v.dpad.face,
        Rgba::from(colours.button),
        "Dolch's grey arrows become the button colour"
    );
    assert_eq!(
        v.common.snip_edge,
        Rgba::from(colours.highlight),
        "shared colours too"
    );
    assert_eq!(
        v.sel_ink,
        Rgba::from(colours.highlight_text),
        "text on a Highlight fill"
    );
    let plain = look_view(&t, &choice("dolch", ModeChoice::Light), &teal()).expect("dolch");
    assert_eq!(plain.common, t.common);
    assert_eq!(plain.sel_ink, plain.palette.pop_ink);
}

#[test]
fn reduced_motion_or_system_animations_off_zero_every_duration() {
    let t = themes();
    let off = SysLook {
        anim_off: true,
        ..SysLook::default()
    };
    let press = |sys: &SysLook, c: &LookChoice| {
        let v = look_view(&t, c, sys).expect("native");
        v.motion.moves["press"].ms
    };
    let mut c = choice("native", ModeChoice::Light);
    let full = t.motion.full.moves["press"].ms;
    assert!(full > 0 && press(&SysLook::default(), &c) == full);
    assert_eq!(press(&off, &c), 0, "the system's animation effects are off");
    c.reduced_motion = true;
    assert_eq!(
        press(&SysLook::default(), &c),
        0,
        "the reduced motion setting"
    );
}

#[test]
fn the_margin_fits_the_widest_outer_shadow() {
    let t = themes();
    let v = look_view(
        &t,
        &choice("native", ModeChoice::Light),
        &SysLook::default(),
    )
    .expect("native");
    assert!(
        (v.margin - 44.0).abs() < f32::EPSILON,
        "12 down plus 32 blur: {}",
        v.margin
    );
    let none = choice("nope", ModeChoice::Light);
    assert!(look_view(&t, &none, &SysLook::default()).is_none());
}

#[test]
fn a_key_under_the_pointer_gets_an_outline_only_in_high_contrast() {
    let t = themes();
    let plain = look_view(
        &t,
        &choice("native", ModeChoice::Light),
        &SysLook::default(),
    )
    .expect("native");
    assert_eq!(plain.hover_line.a, 0, "the theme's tint shows hover");
    let colours = hc_colours();
    let sys = SysLook {
        contrast: Some(colours),
        ..SysLook::default()
    };
    let hc = look_view(&t, &choice("native", ModeChoice::Light), &sys).expect("native");
    assert_eq!(hc.hover_line, Rgba::from(colours.highlight));
    assert!(t.shape.hc_outline.px > 0.0, "the outline has a width");
}

#[test]
fn the_look_carries_the_icons() {
    let t = themes();
    let v = look_view(&t, &choice("et66", ModeChoice::Dark), &SysLook::default()).expect("et66");
    assert_eq!(v.icons.bar.theme, "palette", "the theme button");
    assert_eq!(v.icons, &t.icons);
}
