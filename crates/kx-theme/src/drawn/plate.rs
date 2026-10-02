//! What sits on the plate and the panels: the top bar, the corner and the chips, the panels' texts, rows and marks.

use super::{Drawn, WINDOWS, worst};
use crate::contrast::{WCAG_AA, WCAG_SHAPES};
use crate::rgba::Rgba;
use crate::view::LookView;

/// Text and icons on the plate, solid and, outside high contrast, see-through over a system backdrop of its mode.
pub(super) fn plate(v: &LookView, out: &mut Vec<Drawn>) {
    let solid = v.palette.plate;
    on_plate(v, solid, "", out);
    if !v.contrast {
        let under = WINDOWS[usize::from(v.dark)];
        on_plate(
            v,
            solid.over(under, v.shape.backdrop.plate),
            ", frosted",
            out,
        );
    }
}

/// The top bar's texts and icons, the corner and the chips on `plate`, named with `tag` (TopBar.qml, Chip.qml).
fn on_plate(v: &LookView, plate: Rgba, tag: &str, out: &mut Vec<Drawn>) {
    let p = &v.palette;
    let hot = p.pop_hover.on(plate);
    let chip = p.chip_bg.on(plate);
    let chips = vec![chip, p.hover.on(chip)];
    // The lit size letter is text in pop_ink and the ring marks it, under the pointer too; in high contrast the hovered
    // button is Highlight, so both take sel_ink there, checked as hovered plate text.
    let lit = if v.contrast {
        vec![plate]
    } else {
        vec![plate, hot]
    };
    // A chip is a button, so high contrast draws its icon in ButtonText, as its word (Chip.qml).
    let icon = if v.contrast {
        p.legend
    } else {
        p.legend_2_plate
    };
    let pairs = [
        (
            "legend_2_plate on plate",
            p.legend_2_plate,
            vec![plate],
            WCAG_AA,
        ),
        ("sel_ink on hovered plate", v.sel_ink, vec![hot], WCAG_AA),
        ("lit size letter on plate", p.pop_ink, lit.clone(), WCAG_AA),
        ("ring on plate", p.ring, lit, WCAG_SHAPES),
        ("legend on chip", p.legend, chips.clone(), WCAG_AA),
        ("chip icon on chip", icon, chips, WCAG_SHAPES),
    ];
    for (name, ink, bgs, min) in pairs {
        out.push(worst(&format!("{name}{tag}"), ink, 1.0, &bgs, min));
    }
}

/// Text on the panels (list, notices, tips, tray menu), their rows and buttons, the list's ring and the badge.
pub(super) fn panels(v: &LookView, out: &mut Vec<Drawn>) {
    let p = &v.palette;
    let bg = p.pop_bg;
    let ghost = p.pop_hover.on(bg);
    let main = p.pop_ink;
    let pairs = [
        ("pop_ink on pop_bg", p.pop_ink, vec![bg], WCAG_AA),
        ("pop_muted on pop_bg", p.pop_muted, vec![bg], WCAG_AA),
        (
            "sel_ink on pop_hover",
            v.sel_ink,
            vec![ghost, p.hover.on(ghost)],
            WCAG_AA,
        ),
        (
            "sel_ink on pop_sel",
            v.sel_ink,
            vec![p.pop_sel.on(bg), p.pop_sel.on(ghost)],
            WCAG_AA,
        ),
        (
            "pop_bg on the main button",
            p.pop_bg,
            vec![main, p.hover.on(main)],
            WCAG_AA,
        ),
        ("ring on pop_bg", p.ring, vec![bg], WCAG_SHAPES),
        (
            "badge_ink on badge_bg",
            p.badge_ink,
            vec![p.badge_bg],
            WCAG_AA,
        ),
    ];
    for (name, ink, bgs, min) in pairs {
        out.push(worst(name, ink, 1.0, &bgs, min));
    }
}
