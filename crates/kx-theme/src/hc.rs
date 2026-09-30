//! High contrast: every theme colour becomes one of the seven system colours, and keys get edges.

use kx_platform::ui::SysColors;

use crate::common::Common;
use crate::palette::{Palette, Shadow};
use crate::rgba::Rgba;

/// An inner outline `px` wide in `ink`, since high contrast needs edges.
fn outline(ink: Rgba, px: f32) -> Shadow {
    Shadow {
        inset: true,
        x: 0.0,
        y: 0.0,
        blur: 0.0,
        spread: px,
        color: ink,
    }
}

/// `base` with only system colours and an edge `edge_px` wide on each key (the `[line]` hairline); a tint is not a
/// system colour, so hover shows `hover_line` instead.
#[must_use]
pub fn contrast_palette(base: &Palette, s: &SysColors, edge_px: f32) -> Palette {
    let mut p = base.clone();
    keys(&mut p, s);
    states(&mut p, s);
    panels(&mut p, s);
    p.key_shadow = vec![outline(s.button_text.into(), edge_px)];
    p.window_shadow = Vec::new();
    p.skirt = None;
    p
}

/// The plate, every key face and its legends, dimmed keys in `GrayText`, the LED at rest, chips and the hover tint.
fn keys(p: &mut Palette, s: &SysColors) {
    let (face, ink): (Rgba, Rgba) = (s.button.into(), s.button_text.into());
    p.plate = s.window.into();
    p.dim_ink = s.gray_text.into();
    for c in [
        &mut p.key,
        &mut p.key_mod,
        &mut p.key_act,
        &mut p.danger,
        &mut p.dim,
    ] {
        *c = face;
    }
    for c in [
        &mut p.legend,
        &mut p.legend_mod,
        &mut p.legend_2,
        &mut p.legend_act,
        &mut p.danger_ink,
    ] {
        *c = ink;
    }
    p.led_off = face;
    p.chip_bg = face;
    p.hover = Rgba { a: 0, ..face };
}

/// Enter and every key that is on, locked or recording, in `Highlight` with `HighlightText`.
fn states(p: &mut Palette, s: &SysColors) {
    let (sel, sel_ink): (Rgba, Rgba) = (s.highlight.into(), s.highlight_text.into());
    for c in [
        &mut p.enter,
        &mut p.on,
        &mut p.lock,
        &mut p.rec,
        &mut p.led_on,
        &mut p.ring,
        &mut p.badge_bg,
    ] {
        *c = sel;
    }
    for c in [
        &mut p.enter_ink,
        &mut p.on_ink,
        &mut p.lock_ink,
        &mut p.rec_ink,
        &mut p.badge_ink,
    ] {
        *c = sel_ink;
    }
}

/// Panels and text on the plate in `Window` and `WindowText`, with `Highlight` on the row under the pointer and the
/// selected one.
fn panels(p: &mut Palette, s: &SysColors) {
    let text: Rgba = s.window_text.into();
    p.pop_bg = s.window.into();
    for c in [
        &mut p.pop_ink,
        &mut p.pop_muted,
        &mut p.pop_line,
        &mut p.legend_2_plate,
    ] {
        *c = text;
    }
    p.pop_hover = s.highlight.into();
    p.pop_sel = s.highlight.into();
}

/// The outline of a key under the pointer in high contrast: `Highlight`, as Windows marks hover.
#[must_use]
pub fn hover_line(s: &SysColors) -> Rgba {
    s.highlight.into()
}

/// The shared colours in system colours, keeping the see-through share of the snip layers from `common`.
#[must_use]
pub fn contrast_common(s: &SysColors, common: &Common) -> Common {
    let (window, highlight): (Rgba, Rgba) = (s.window.into(), s.highlight.into());
    let clear = Rgba {
        a: 0,
        ..s.button.into()
    };
    Common {
        press: clear,
        rec_dot: highlight,
        pill_hover: clear,
        snip_dim: Rgba {
            a: common.snip_dim.a,
            ..window
        },
        snip_edge: highlight,
        snip_tag: Rgba {
            a: common.snip_tag.a,
            ..window
        },
    }
}

#[cfg(test)]
pub(crate) mod sample;
#[cfg(test)]
mod tests;
