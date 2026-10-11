//! High-contrast tests over the sample set of system colours.

use super::sample::HC;
use super::*;
use crate::Themes;

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

fn themes() -> Themes {
    Themes::shipped().expect("tokens load")
}

#[test]
fn a_system_colour_keeps_its_channels() {
    let c = Rgba::from([1, 2, 3, 4]);
    assert_eq!((c.r, c.g, c.b, c.a), (1, 2, 3, 4));
}

#[test]
fn high_contrast_uses_only_system_colours_in_every_theme_and_mode() {
    let t = themes();
    let allowed = allowed(&HC);
    let system = |c: Rgba| allowed.iter().any(|&a| rgb(a) == rgb(c));
    for base in t.list.iter().flat_map(|t| [&t.light, &t.dark]) {
        let p = contrast_palette(base, &HC, t.shape.line.px);
        assert!(p.named().iter().all(|&(_, c)| system(c)), "{p:?}");
        assert_eq!(p.hover.a, 0, "no tints in high contrast");
        assert!(p.skirt.is_none() && p.window_shadow.is_empty());
        assert_eq!(p.key_shadow.len(), 1, "one edge per key");
        assert_eq!(p.key_shadow[0].color, Rgba::from(HC.button_text));
    }
}

#[test]
fn a_pressed_key_takes_highlight_with_its_text_never_a_tint() {
    let t = themes();
    for base in t.list.iter().flat_map(|t| [&t.light, &t.dark]) {
        let p = contrast_palette(base, &HC, t.shape.line.px);
        assert_eq!(
            p.press,
            Rgba::from(HC.highlight),
            "as Windows presses a button"
        );
        assert_eq!(
            p.press_ink,
            Some(Rgba::from(HC.highlight_text)),
            "legends follow"
        );
    }
}

#[test]
fn a_key_edge_is_a_hairline_wide() {
    let t = themes();
    for px in [t.shape.line.px, 2.5] {
        let p = contrast_palette(&t.list[0].light, &HC, px);
        let edge = p.key_shadow[0];
        assert!(
            edge.inset && (edge.spread - px).abs() < f32::EPSILON,
            "{px}"
        );
    }
}

#[test]
fn shared_colours_turn_to_system_colours_and_keep_their_see_through_share() {
    let base = themes().common;
    let c = contrast_common(&HC, &base);
    let allowed = [HC.window, HC.highlight, HC.button].map(Rgba::from);
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
    let t = themes();
    let p = contrast_palette(&t.list[0].dark, &HC, t.shape.line.px);
    assert_eq!(p.dim, Rgba::from(HC.button));
    assert_eq!(p.dim_ink, Rgba::from(HC.gray_text), "disabled text");
    assert_eq!(
        p.legend_2_plate,
        Rgba::from(HC.window_text),
        "text on the plate"
    );
}

#[test]
fn the_hover_outline_is_highlight() {
    assert_eq!(hover_line(&HC), Rgba::from(HC.highlight));
}
