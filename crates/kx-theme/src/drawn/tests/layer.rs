//! Named keys and the shortcuts layer as drawn: a label is text and an icon a mark, but the layer's icons, F-key labels
//! and Next chevron read at 4.5:1, the chevron sits as side icons do with its dots under its ink, and both keys' unlit
//! dots are checked.

use super::*;

#[test]
fn a_named_keys_label_is_text_and_its_icon_a_mark() {
    let t = themes();
    for (at, v) in views(&t, &SysLook::default()) {
        let all = drawn(&v, TEST_ROW_PX);
        let min = |n: &str| all.iter().find(|d| d.name == n).map(|d| d.min);
        assert_eq!(min("legend_mod on key_mod, rest"), Some(WCAG_AA), "{at}");
        let icon = min("legend_mod icon on key_mod, rest");
        assert_eq!(icon, Some(WCAG_SHAPES), "{at}");
    }
}

/// The rows named `names` in `all`, which must hold each in every state.
fn rows<'a>(all: &'a [Drawn], names: &[&str], at: &str) -> Vec<&'a Drawn> {
    let found: Vec<&Drawn> = all
        .iter()
        .filter(|d| names.iter().any(|n| d.name.starts_with(n)))
        .collect();
    assert_eq!(found.len(), names.len() * STATES.len(), "{at}: {names:?}");
    found
}

#[test]
fn the_layers_icons_f_key_labels_and_next_chevron_are_rows_held_to_4_5_to_1() {
    let t = themes();
    for (at, v) in views(&t, &SysLook::default()) {
        let all = drawn(&v, TEST_ROW_PX);
        // An F-key's label, a shortcut's icon and the chevron, in legend_act on key_act, read as text does (spec §4).
        let held = ["legend_act on key_act", SHORTCUT_ICON, NEXT_CHEVRON];
        for d in rows(&all, &held, &at) {
            assert!((d.min - WCAG_AA).abs() < f32::EPSILON, "{at}: {}", d.name);
            assert!(d.ratio() >= d.min, "{at}: {} is {:.2}:1", d.name, d.ratio());
        }
    }
}

#[test]
fn the_unlit_dot_is_checked_on_both_keys_that_draw_dots() {
    let t = themes();
    for sys in [SysLook::default(), hc()] {
        for (at, v) in views(&t, &sys) {
            let all = drawn(&v, TEST_ROW_PX);
            let p = &v.palette;
            for (name, ink) in [(CYCLE_DOT, p.legend_mod), (NEXT_DOT, p.legend_act)] {
                let rest = rows(&all, &[name], &at)[0];
                let faded = ink.over(rest.bg, v.shape.dots.off);
                assert_eq!((rest.ink, rest.min), (faded, WCAG_SHAPES), "{at}: {name}");
            }
        }
    }
}

/// Where Lucide's chevron-right ends its ink, round stroke included: 19 of its 24 units.
const CHEVRON_INK_END: f32 = 19.0 / 24.0;

#[test]
fn the_next_chevron_sits_as_the_side_icons_do_and_its_dots_clear_its_ink_inside_the_cap() {
    let t = themes();
    for (at, v) in views(&t, &SysLook::default()) {
        let h = height(&v, TEST_ROW_PX);
        let (d, icon) = (&v.shape.dots, v.shape.legend.icon_px[1]);
        let (chevron, side) = (spans(&v, h, Mark::Next)[0], spans(&v, h, Mark::Icon)[1]);
        let same = chevron
            .iter()
            .zip(side)
            .all(|(a, b)| (a - b).abs() < f32::EPSILON);
        assert!(
            same,
            "{at}: {chevron:?} is not where side icons sit, {side:?}"
        );
        let row = spans(&v, h, Mark::Pages)[0];
        let from = chevron[0] + icon * d.next_at;
        assert!((row[0] - from).abs() < 1e-4, "{at}: from next_at down");
        assert!((row[1] - row[0] - d.px).abs() < 1e-4, "{at}: an unlit dot");
        let grow = (d.on_px - d.px) / 2.0;
        let lit = [row[0] - grow, row[1] + grow];
        let ink = chevron[0] + icon * CHEVRON_INK_END;
        assert!(lit[0] >= ink, "{at}: the lit dot clears the chevron's ink");
        assert!(lit[1] <= h, "{at}: inside the cap");
    }
}

#[test]
fn an_unlit_dot_faded_too_far_is_named_on_both_keys() {
    let t = themes();
    let (_, v) = views(&t, &SysLook::default()).swap_remove(0);
    let mut shape = v.shape.clone();
    shape.dots.off = 0.05;
    let v = LookView { shape: &shape, ..v };
    let low = low(&v, TEST_ROW_PX);
    for name in [CYCLE_DOT, NEXT_DOT] {
        assert!(low.iter().any(|l| l.starts_with(name)), "{name}: {low:?}");
    }
}
