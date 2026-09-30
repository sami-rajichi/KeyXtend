//! High-contrast tests over a sample set of system colours.

use super::*;
use crate::Themes;

/// A sample high-contrast set, as `[r, g, b, a]`.
fn sys() -> SysColors {
    SysColors {
        window: [0x00, 0x00, 0x00, 0xFF],
        window_text: [0xFF, 0xFF, 0xFF, 0xFF],
        highlight: [0x1A, 0xEB, 0xFF, 0xFF],
        highlight_text: [0x00, 0x00, 0x00, 0xFF],
        button: [0x10, 0x10, 0x10, 0xFF],
        button_text: [0xFF, 0xFF, 0x00, 0xFF],
        gray_text: [0x3F, 0xF2, 0x3F, 0xFF],
    }
}

/// The seven system colours of `s`, in theme colours.
fn allowed(s: &SysColors) -> [Rgba; 7] {
    [
        s.window,
        s.window_text,
        s.highlight,
        s.highlight_text,
        s.button,
        s.button_text,
        s.gray_text,
    ]
    .map(Rgba::from)
}

/// The colour without its alpha, since see-through tints keep a system colour's channels.
fn rgb(c: Rgba) -> (u8, u8, u8) {
    (c.r, c.g, c.b)
}

#[test]
fn a_system_colour_keeps_its_channels() {
    let c = Rgba::from([1, 2, 3, 4]);
    assert_eq!((c.r, c.g, c.b, c.a), (1, 2, 3, 4));
}

#[test]
fn high_contrast_uses_only_system_colours() {
    let s = sys();
    let base = &Themes::shipped().expect("tokens load").list[2].light;
    let p = contrast_palette(base, &s);
    let allowed = allowed(&s);
    let system = |c: Rgba| allowed.iter().any(|&a| rgb(a) == rgb(c));
    assert!(p.named().iter().all(|&(_, c)| system(c)), "{p:?}");
    assert_eq!(p.hover.a, 0, "no tints in high contrast");
    assert!(p.skirt.is_none() && p.window_shadow.is_empty());
    assert_eq!(
        p.key_shadow.len(),
        1,
        "one outline, as high contrast needs edges"
    );
    assert_eq!(p.key_shadow[0].color, Rgba::from(s.button_text));
}

#[test]
fn shared_colours_turn_to_system_colours_and_keep_their_see_through_share() {
    let s = sys();
    let base = Themes::shipped().expect("tokens load").common;
    let c = contrast_common(&s, &base);
    let allowed = [s.window, s.highlight, s.button].map(Rgba::from);
    for (n, v) in c.named() {
        assert!(allowed.iter().any(|&a| rgb(a) == rgb(v)), "{n}");
    }
    assert_eq!((c.press.a, c.pill_hover.a), (0, 0), "no tints");
    assert_eq!(
        (c.snip_dim.a, c.snip_tag.a),
        (base.snip_dim.a, base.snip_tag.a)
    );
}

#[test]
fn dimmed_keys_take_gray_text_in_high_contrast() {
    let s = sys();
    let base = &Themes::shipped().expect("tokens load").list[0].dark;
    let p = contrast_palette(base, &s);
    assert_eq!(p.dim, Rgba::from(s.button));
    assert_eq!(p.dim_ink, Rgba::from(s.gray_text), "disabled text");
}

#[test]
fn the_hover_outline_is_highlight() {
    assert_eq!(hover_line(&sys()), Rgba::from(sys().highlight));
}
