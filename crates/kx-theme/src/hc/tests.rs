//! High-contrast tests over the sample set of system colours, and over Windows' own contrast themes.

use super::sample::HC;
use super::*;
use crate::Themes;
use crate::contrast::{WCAG_SHAPES, contrast};

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
        assert!(p.skirt.is_none() && p.window_shadow.is_empty() && p.badge_shadow.is_empty());
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

/// Windows' own contrast themes as Microsoft Learn publishes them ("Contrast themes"), not read from this PC: each
/// set's `RRGGBB` values in `SysColors`' order, the window and its text, Highlight and its text, a button and its
/// text, then disabled text.
const WINDOWS: [(&str, &str); 6] = [
    (
        "Aquatic",
        "202020 FFFFFF 8EE3F0 263B50 202020 FFFFFF A6A6A6",
    ),
    ("Desert", "FFFAEF 3D3D3D 903909 FFF5E3 FFFAEF 202020 676767"),
    ("Dusk", "2D3236 B6F6F0 A1BFDE 212D3B 2D3236 B6F6F0 A6A6A6"),
    (
        "Night sky",
        "000000 FFFFFF D6B4FD 2B2B2B 000000 FFEE32 A6A6A6",
    ),
    (
        "High Contrast White",
        "FFFFFF 000000 37006E FFFFFF FFFFFF 000000 600000",
    ),
    (
        "High Contrast Black",
        "000000 FFFFFF 1AEBFF 000000 000000 FFFFFF 3FF23F",
    ),
];

/// The system colours written as `hex`, in `WINDOWS`' order.
fn set(hex: &str) -> SysColors {
    let rgba = |h: &str| {
        let [_, r, g, b] = u32::from_str_radix(h, 16).expect("hex").to_be_bytes();
        [r, g, b, u8::MAX]
    };
    let c: Vec<[u8; 4]> = hex.split(' ').map(rgba).collect();
    SysColors {
        window: c[0],
        window_text: c[1],
        highlight: c[2],
        highlight_text: c[3],
        button: c[4],
        button_text: c[5],
        gray_text: c[6],
    }
}

/// Microsoft's contrast themes never mark a state by colour alone: Preparing keeps a resting key's pair and the key
/// draws a ring in `ring`, so only Recording fills, and both stand out from the face in each of Windows' themes.
#[test]
fn preparing_keeps_a_resting_keys_pair_so_only_recording_fills() {
    let t = themes();
    for (name, hex) in WINDOWS {
        let s = set(hex);
        let sel = Rgba::from(s.highlight);
        for base in t.list.iter().flat_map(|t| [&t.light, &t.dark]) {
            let p = contrast_palette(base, &s, t.shape.line.px);
            assert_eq!((p.prep, p.prep_ink), (p.key, p.legend), "{name}");
            assert_eq!((p.rec, p.ring), (sel, sel), "{name}: Highlight");
            assert!(contrast(p.rec, p.prep) >= WCAG_SHAPES, "{name}: the fill");
            assert!(contrast(p.ring, p.prep) >= WCAG_SHAPES, "{name}: the ring");
        }
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
    assert_eq!(c.press.a, 0, "no tint");
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
