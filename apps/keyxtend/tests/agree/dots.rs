//! The Next key holds the most pages' dots on one line inside its cap, as `PageDots.qml` lays them out under its
//! chevron, in every theme, high contrast's flat caps too, and at every size step.

use crate::pages;
use kx_mod_keyboard::spec::NEXT_ID;
use kx_mod_keyboard::view::{board_view, with_layer};
use kx_mod_keyboard::{BoardSpec, KeyboardSettings, Layer, MAX_PAGES, SPEC};
use kx_theme::{Cap, Shape, Themes};
use std::collections::HashSet;

/// Float rounding the widths may carry.
const SLACK: f64 = 1e-9;

/// The Next key's cell width on the shipped board at `size`.
fn next_w(spec: &BoardSpec, settings: &KeyboardSettings, size: f64) -> f64 {
    let pages = pages();
    let known: HashSet<_> = pages.iter().flatten().cloned().collect();
    let mut layer = Layer::new(spec.layer_grid(), &pages, &known);
    layer.toggle();
    let mut view = board_view(spec, settings, size);
    with_layer(&mut view, spec, &layer, &[]);
    view.keys.iter().find(|k| k.id == NEXT_ID).unwrap().cell.w
}

/// The room a line of dots may take on a `w` wide cell at `size`: the cap, one key gap `gap` narrower, less the label
/// padding, or on a sculpted cap its top plus what a label may spill, as `NamedFace.qml` gives a label.
fn room(w: f64, gap: f32, cap: &Cap, shape: &Shape, size: f64) -> f64 {
    let pad = match cap {
        Cap::Skirt {
            top_inset_px: t,
            label_spill_px: spill,
            ..
        } => t[1] + t[3] - 2.0 * spill,
        _ => 2.0 * shape.legend.label_pad_px,
    };
    w - f64::from(gap + pad) * size
}

/// The length of one line of `n` dots at `size`, with space for the larger lit dot, as `PageDots.qml` lays them out.
fn line(n: usize, shape: &Shape, size: f64) -> f64 {
    let d = &shape.dots;
    let n = f64::from(u32::try_from(n).unwrap());
    let len = n * f64::from(d.px) + (n - 1.0) * f64::from(d.gap_px) + f64::from(d.on_px - d.px);
    len * size
}

/// Every size step the default settings allow, from the smallest to the largest.
fn sizes(s: &KeyboardSettings) -> Vec<f64> {
    let (mut all, half) = (vec![s.size_min], s.size_step / 2.0);
    while let Some(next) = all
        .last()
        .map(|l| l + s.size_step)
        .filter(|n| *n <= s.size_max + half)
    {
        all.push(next);
    }
    all
}

#[test]
fn the_most_pages_dots_fit_one_line_inside_the_next_key_in_every_theme_and_size() {
    let (spec, themes) = (BoardSpec::shipped().unwrap(), Themes::shipped().unwrap());
    let settings: KeyboardSettings = SPEC.defaults_as().unwrap();
    let all = sizes(&settings);
    assert!(all.len() > 1, "{all:?}");
    for size in all {
        let (w, want) = (
            next_w(&spec, &settings, size),
            line(MAX_PAGES, &themes.shape, size),
        );
        for t in &themes.list {
            for cap in [&t.look.cap, &Cap::Flat {}] {
                let room = room(w, t.look.gap_px, cap, &themes.shape, size);
                let at = format!("{} at {size:.2}", t.look.id);
                assert!(
                    want <= room + SLACK,
                    "{at}: {MAX_PAGES} dots take {want:.2} of {room:.2}"
                );
            }
        }
    }
}
