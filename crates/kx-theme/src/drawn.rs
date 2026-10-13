//! What the keyboard draws: each text, icon and mark on the colour it really sits on, blended as QML blends it, with
//! the least contrast WCAG asks of it.

mod place;
mod plate;

use crate::contrast::{WCAG_AA, WCAG_SHAPES, contrast};
use crate::rgba::Rgba;
use crate::view::LookView;
use place::{Mark, height, spans, under};

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

/// A board row at size 1 for kx-theme's own tests; kx-ui's test runs the check on the keyboard's own row.
#[cfg(test)]
pub(crate) const TEST_ROW_PX: f32 = 31.0;

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

/// Every pair view `v` draws on a keyboard whose rows are `row_px` high at size 1: legends on caps, the language
/// key's marks, the D-pad, the plate and the panels.
#[must_use]
pub fn drawn(v: &LookView, row_px: f32) -> Vec<Drawn> {
    let (mut out, h) = (Vec::new(), height(v, row_px));
    keys(v, h, &mut out);
    marks(v, h, &mut out);
    dpad(v, &mut out);
    plate::plate(v, &mut out);
    plate::panels(v, &mut out);
    out
}

/// Every pair of `v` on rows `row_px` high below its minimum but the named exception, as `<pair> is <n>:1`.
#[must_use]
pub fn low(v: &LookView, row_px: f32) -> Vec<String> {
    drawn(v, row_px)
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

/// What a key draws: its name, ink, opacity, what it is and the least contrast it needs.
struct Legend<'a> {
    name: &'a str,
    ink: Rgba,
    share: f32,
    mark: Mark,
    min: f32,
}

/// A key's colour and, on a sculpted cap, the skirt around its top.
#[derive(Clone, Copy)]
struct Fill {
    top: Rgba,
    skirt: Rgba,
}

/// Each key's ink, colour, skirt and what the ink draws, by token, as kx-mod-keyboard's paint.rs pairs them; a
/// named key's label also has an icon.
const KEYS: [(&str, &str, &str, Mark); 9] = [
    ("legend", "key", "key", Mark::Main),
    ("legend_2", "key", "key", Mark::Second),
    ("legend_mod", "key_mod", "key_mod", Mark::Label),
    ("legend_act", "key_act", "key_mod", Mark::Label),
    ("enter_ink", "enter", "enter", Mark::Label),
    ("danger_ink", "danger", "danger", Mark::Label),
    ("on_ink", "on", "on", Mark::Label),
    ("lock_ink", "lock", "on", Mark::Label),
    ("rec_ink", "rec", "rec", Mark::Label),
];

/// The palette colour `name`; an unknown one is clear, which fails the opacity test.
fn token(v: &LookView, name: &str) -> Rgba {
    v.palette.get(name).unwrap_or(Rgba::CLEAR)
}

/// A key of colour `top` with the skirt called `skirt`, if its cap has skirts.
fn fill(v: &LookView, top: &str, skirt: &str) -> Fill {
    let top = token(v, top);
    let skirt = v.palette.skirt.as_ref().and_then(|s| s.get(skirt));
    Fill {
        top,
        skirt: skirt.unwrap_or(top),
    }
}

/// The colours under `mark` on a key `h` high of `f` in each of `STATES`: where it sits, under the hover tint, and
/// pressed under the press shade.
fn states(v: &LookView, h: f32, mark: Mark, f: Fill) -> [Vec<Rgba>; 3] {
    let (mut rest, mut down) = (Vec::new(), Vec::new());
    for span in spans(v, h, mark) {
        rest.extend(under(v, h, f.top, f.skirt, span, false));
        down.extend(under(v, h, f.top, f.skirt, span, true));
    }
    let hover = rest.iter().map(|&s| v.palette.hover.on(s)).collect();
    let press = down.into_iter().map(|s| v.palette.press.on(s)).collect();
    [rest, hover, press]
}

/// `l` on a key `h` high of `f` in every state; pressed, it takes the press text when the look has one.
fn on_cap(v: &LookView, h: f32, l: &Legend, f: Fill) -> Vec<Drawn> {
    let states = states(v, h, l.mark, f);
    let inks = [l.ink, l.ink, v.palette.press_ink.unwrap_or(l.ink)];
    let named = STATES.iter().zip(states.iter()).zip(inks);
    named
        .map(|((state, bgs), ink)| worst(&format!("{}, {state}", l.name), ink, l.share, bgs, l.min))
        .collect()
}

/// Each legend on its key `h` high, where it sits, with the hover tint or press shade under it (Key.qml); a named
/// key's icon is a mark. Dimmed keys are left out, as WCAG sets no minimum for inactive parts (1.4.3).
fn keys(v: &LookView, h: f32, out: &mut Vec<Drawn>) {
    for (ink, top, skirt, mark) in KEYS {
        let (name, f) = (format!("{ink} on {top}"), fill(v, top, skirt));
        let ink = token(v, ink);
        let text = Legend {
            name: &name,
            ink,
            share: 1.0,
            mark,
            min: WCAG_AA,
        };
        out.extend(on_cap(v, h, &text, f));
        if mark == Mark::Label {
            let name = name.replacen(" on ", " icon on ", 1);
            let icon = Legend {
                name: &name,
                mark: Mark::Icon,
                min: WCAG_SHAPES,
                ..text
            };
            out.extend(on_cap(v, h, &icon, f));
        }
    }
}

/// The language key's faded marks in `legend_mod` on a key `h` high: Cycle's unlit dots and globe, List's chevron,
/// Carousel's side names.
fn marks(v: &LookView, h: f32, out: &mut Vec<Drawn>) {
    let (l, f) = (&v.shape.lang, fill(v, "key_mod", "key_mod"));
    let marks = [
        ("Cycle unlit dot", l.cycle.dot_off, Mark::Dots, WCAG_SHAPES),
        ("Cycle globe", l.cycle.icon_share, Mark::Globe, WCAG_SHAPES),
        (
            "List chevron",
            l.list.icon_share,
            Mark::Chevron,
            WCAG_SHAPES,
        ),
        (EXCEPTION, l.side[1], Mark::Side, WCAG_AA),
    ];
    for (name, share, mark, min) in marks {
        let ink = v.palette.legend_mod;
        let l = Legend {
            name,
            ink,
            share,
            mark,
            min,
        };
        out.extend(on_cap(v, h, &l, f));
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

#[cfg(test)]
mod tests;
