//! Named keys and the shortcuts layer as drawn: a label is text and an icon a mark, the layer's icons and F-key labels
//! read at 4.5:1 on their keys, the Next key's dots sit under its chevron, and the unlit dot is checked on both keys.

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
fn the_layers_cells_read_at_4_5_to_1_and_the_next_keys_chevron_at_3_to_1() {
    let t = themes();
    for (at, v) in views(&t, &SysLook::default()) {
        let all = drawn(&v, TEST_ROW_PX);
        // A shortcut's icon and an F-key's label, in legend_act on key_act, read as text does.
        for d in rows(
            &all,
            &["legend_act on key_act", "legend_act icon on key_act"],
            &at,
        ) {
            assert!(
                d.ratio() >= WCAG_AA,
                "{at}: {} is {:.2}:1",
                d.name,
                d.ratio()
            );
        }
        // The chevron rises to make room for its dots, so it is checked where it sits, as a shape (WCAG 1.4.11).
        for d in rows(&all, &[NEXT_CHEVRON], &at) {
            assert!((d.min - WCAG_SHAPES).abs() < f32::EPSILON, "{at}");
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

#[test]
fn the_next_keys_dots_sit_under_its_chevron_inside_the_cap() {
    let t = themes();
    for (at, v) in views(&t, &SysLook::default()) {
        let h = height(&v, TEST_ROW_PX);
        let (chevron, dots) = (spans(&v, h, Mark::Next), spans(&v, h, Mark::Pages));
        let ([up, down], [top, bottom]) = (chevron[0], dots[0]);
        let gap = v.shape.dots.gap_px;
        assert!(
            (top - down - gap).abs() < f32::EPSILON,
            "{at}: one gap under it"
        );
        assert!(
            (bottom - top - v.shape.dots.px).abs() < f32::EPSILON,
            "{at}"
        );
        assert!(up >= 0.0 && bottom <= h, "{at}: inside the cap");
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
