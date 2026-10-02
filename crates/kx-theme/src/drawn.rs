//! What the keyboard draws: each text, icon and mark on the colour it really sits on, blended as QML blends it, with
//! the least contrast WCAG asks of it.

use crate::contrast::{WCAG_AA, WCAG_SHAPES, contrast};
use crate::look::Cap;
use crate::rgba::Rgba;
use crate::view::LookView;

/// The one pair allowed below its minimum (owner, 2026-10-02): the Carousel's side names, faded as decoration.
pub const EXCEPTION: &str = "Carousel side names";

/// Windows 11's plain window colours, light then dark (Mica's base): what a system backdrop shows behind the plate.
pub const WINDOWS: [Rgba; 2] = [
    Rgba {
        r: 0xF3,
        g: 0xF3,
        b: 0xF3,
        a: u8::MAX,
    },
    Rgba {
        r: 0x20,
        g: 0x20,
        b: 0x20,
        a: u8::MAX,
    },
];

/// A cap at rest, under the pointer and pressed.
const STATES: [&str; 3] = ["rest", "hover", "press"];

/// One ink as drawn on its background, both opaque, and the least contrast it needs.
#[derive(Debug, Clone, PartialEq)]
pub struct Drawn {
    /// What it is and in which state, such as `legend_2 on key, hover`.
    pub name: String,
    /// The ink, blended at its opacity.
    pub ink: Rgba,
    /// The background where the ink reads worst.
    pub bg: Rgba,
    /// `WCAG_AA` for text, `WCAG_SHAPES` for icons and marks.
    pub min: f32,
}

impl Drawn {
    /// Its contrast ratio.
    #[must_use]
    pub fn ratio(&self) -> f32 {
        contrast(self.ink, self.bg)
    }
}

/// Every pair view `v` draws: legends on caps, the language key's marks, the D-pad, the plate and the panels.
#[must_use]
pub fn drawn(v: &LookView) -> Vec<Drawn> {
    let mut out = Vec::new();
    keys(v, &mut out);
    marks(v, &mut out);
    dpad(v, &mut out);
    plate(v, &mut out);
    panels(v, &mut out);
    out
}

/// Every pair of `v` below its minimum but the named exception, as `<pair> is <n>:1`.
#[must_use]
pub fn low(v: &LookView) -> Vec<String> {
    drawn(v)
        .into_iter()
        .filter(|d| d.ratio() < d.min && !d.name.starts_with(EXCEPTION))
        .map(|d| format!("{} is {:.2}:1", d.name, d.ratio()))
        .collect()
}

/// The pair `name`: `ink` at opacity `share` over each of `bgs`, kept where it reads worst.
fn worst(name: &str, ink: Rgba, share: f32, bgs: &[Rgba], min: f32) -> Drawn {
    let on = |bg: Rgba| Drawn {
        name: name.to_owned(),
        ink: ink.over(bg, share),
        bg,
        min,
    };
    let all = bgs.iter().map(|&bg| on(bg));
    // An empty list draws the ink on itself, which fails any check.
    all.min_by(|a, b| a.ratio().total_cmp(&b.ratio()))
        .unwrap_or_else(|| on(ink))
}

/// The colours a cap of `fill` shows under its legends: its gradient's two ends and its own colour.
fn spots(cap: &Cap, fill: Rgba, pressed: bool) -> Vec<Rgba> {
    let g = match cap {
        Cap::Flat {} => return vec![fill],
        Cap::Convex {
            rest,
            pressed: down,
        } => {
            if pressed {
                down
            } else {
                rest
            }
        }
        Cap::Skirt { top, .. } => top,
    };
    let (light, dark) = (
        Rgba::WHITE.over(fill, g.lighten),
        Rgba::BLACK.over(fill, g.darken),
    );
    vec![light, fill, dark]
}

/// A cap of `fill` in each of `STATES`: its spots, under the hover tint, and pressed under the press shade.
fn states(v: &LookView, fill: Rgba) -> [Vec<Rgba>; 3] {
    let cap = &v.look.cap;
    let rest = spots(cap, fill, false);
    let hover = rest.iter().map(|&s| v.palette.hover.on(s)).collect();
    let down = spots(cap, fill, true);
    let press = down.into_iter().map(|s| v.common.press.on(s)).collect();
    [rest, hover, press]
}

/// `ink` at `share` on a cap of `fill` in every state, as text or a mark needing `min`.
fn on_cap(v: &LookView, name: &str, ink: Rgba, share: f32, fill: Rgba, min: f32) -> Vec<Drawn> {
    let states = states(v, fill);
    STATES
        .iter()
        .zip(states.iter())
        .map(|(state, bgs)| worst(&format!("{name}, {state}"), ink, share, bgs, min))
        .collect()
}

/// Each legend on its key, with the hover tint or press shade under it (Key.qml); dimmed keys are left out, as WCAG
/// sets no minimum for inactive parts (1.4.3).
fn keys(v: &LookView, out: &mut Vec<Drawn>) {
    let p = &v.palette;
    let pairs = [
        ("legend on key", p.legend, p.key),
        ("legend_2 on key", p.legend_2, p.key),
        ("legend_mod on key_mod", p.legend_mod, p.key_mod),
        ("legend_act on key_act", p.legend_act, p.key_act),
        ("enter_ink on enter", p.enter_ink, p.enter),
        ("danger_ink on danger", p.danger_ink, p.danger),
        ("on_ink on on", p.on_ink, p.on),
        ("lock_ink on lock", p.lock_ink, p.lock),
        ("rec_ink on rec", p.rec_ink, p.rec),
    ];
    for (name, ink, fill) in pairs {
        out.extend(on_cap(v, name, ink, 1.0, fill, WCAG_AA));
    }
}

/// The language key's faded marks in `legend_mod`: Cycle's unlit dots and globe, List's chevron, Carousel's side names.
fn marks(v: &LookView, out: &mut Vec<Drawn>) {
    let (l, p) = (&v.shape.lang, &v.palette);
    let marks = [
        ("Cycle unlit dot", l.cycle.dot_off, WCAG_SHAPES),
        ("Cycle globe", l.cycle.icon_share, WCAG_SHAPES),
        ("List chevron", l.list.icon_share, WCAG_SHAPES),
        (EXCEPTION, l.side[1], WCAG_AA),
    ];
    for (name, share, min) in marks {
        out.extend(on_cap(v, name, p.legend_mod, share, p.key_mod, min));
    }
}

/// The D-pad's icons on their flat arrows, tinted as DPad.qml tints them, and Stop's.
fn dpad(v: &LookView, out: &mut Vec<Drawn>) {
    let (d, p) = (&v.dpad, &v.palette);
    for (name, ink, face) in [
        ("D-pad arrow", d.ink, d.face),
        ("D-pad lit arrow", d.on_ink, d.on),
    ] {
        let bgs = [face, p.hover.on(face), v.common.press.on(face)];
        out.push(worst(name, ink, 1.0, &bgs, WCAG_SHAPES));
    }
    let stop = worst("D-pad Stop", d.stop_ink, 1.0, &[d.stop], WCAG_SHAPES);
    out.push(stop);
}

/// Text and icons on the plate, solid and, outside high contrast, see-through over a system backdrop of its mode.
fn plate(v: &LookView, out: &mut Vec<Drawn>) {
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
    // The lit size letter keeps the ring under the pointer, except in high contrast, where it takes sel_ink.
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
        ("ring on plate", p.ring, lit, WCAG_AA),
        ("legend on chip", p.legend, chips.clone(), WCAG_AA),
        ("chip icon on chip", icon, chips, WCAG_SHAPES),
    ];
    for (name, ink, bgs, min) in pairs {
        out.push(worst(&format!("{name}{tag}"), ink, 1.0, &bgs, min));
    }
}

/// Text on the panels (list, notices, tips, tray menu), their rows and buttons, the list's ring and the badge.
fn panels(v: &LookView, out: &mut Vec<Drawn>) {
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

#[cfg(test)]
mod tests;
