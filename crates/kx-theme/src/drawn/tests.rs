//! Drawn-contrast tests over the shipped themes in every mode, over a system backdrop and in high contrast.

use kx_platform::ui::SysLook;

use super::*;
use crate::Themes;
use crate::contrast::luminance;
use crate::hc::sample::HC;
use crate::look::Cap;
use crate::mode::{Mode, ModeChoice};
use crate::view::{LookChoice, look_view};

fn themes() -> Themes {
    Themes::shipped().expect("tokens load")
}

/// Theme `id` in `mode`, keeping its own colours.
fn choice(id: &str, mode: Mode) -> LookChoice {
    let mode = match mode {
        Mode::Light => ModeChoice::Light,
        Mode::Dark => ModeChoice::Dark,
    };
    LookChoice {
        theme: id.into(),
        mode,
        follow_accent: false,
        reduced_motion: false,
        min_contrast: WCAG_AA,
    }
}

/// Every theme's view in both modes on system look `sys`, each named `<id>.<mode>`.
fn views<'a>(t: &'a Themes, sys: &SysLook) -> Vec<(String, LookView<'a>)> {
    let mut all = Vec::new();
    for theme in &t.list {
        for mode in Mode::ALL {
            let id = &theme.look.id;
            let v = look_view(t, &choice(id, mode), sys, None).expect("a shipped theme");
            all.push((format!("{id}.{}", mode.key()), v));
        }
    }
    all
}

/// The pairs below their minimum in every view on `sys`, named by view.
fn all_low(sys: &SysLook) -> Vec<String> {
    let t = themes();
    let views = views(&t, sys);
    let named = views.iter().map(|(at, v)| (at, low(v, TEST_ROW_PX)));
    named
        .flat_map(|(at, low)| low.into_iter().map(move |l| format!("{at}: {l}")))
        .collect()
}

/// High contrast with the sample system colours.
fn hc() -> SysLook {
    SysLook {
        contrast: Some(HC),
        ..SysLook::default()
    }
}

#[test]
fn every_pair_the_keyboard_draws_reaches_its_minimum_in_every_theme_and_mode() {
    let low = all_low(&SysLook::default());
    assert!(low.is_empty(), "{low:#?}");
}

#[test]
fn high_contrast_draws_every_pair_readably() {
    let low = all_low(&hc());
    assert!(low.is_empty(), "{low:#?}");
}

#[test]
fn the_carousel_side_names_are_the_one_pair_allowed_below_its_minimum() {
    let t = themes();
    let mut faded = 0;
    for (at, v) in views(&t, &SysLook::default()) {
        let all = drawn(&v, TEST_ROW_PX);
        let side: Vec<&Drawn> = all
            .iter()
            .filter(|d| d.name.starts_with(EXCEPTION))
            .collect();
        assert_eq!(side.len(), STATES.len(), "{at}: checked in every state");
        assert!(side.iter().all(|d| d.min >= WCAG_AA), "{at}: as text");
        faded += side.iter().filter(|d| d.ratio() < d.min).count();
        let under = all.iter().filter(|d| d.ratio() < d.min);
        let names: Vec<&str> = under.map(|d| d.name.as_str()).collect();
        assert!(
            names.iter().all(|n| n.starts_with(EXCEPTION)),
            "{at}: {names:?}"
        );
    }
    assert!(faded > 0, "the owner's fade still needs its exception");
}

#[test]
fn every_drawn_pair_is_opaque_so_its_ratio_is_what_shows() {
    let t = themes();
    for sys in [SysLook::default(), hc()] {
        for (at, v) in views(&t, &sys) {
            for d in drawn(&v, TEST_ROW_PX) {
                let opaque = d.ink.a == u8::MAX && d.bg.a == u8::MAX;
                assert!(opaque, "{at}: {} {:?} on {:?}", d.name, d.ink, d.bg);
            }
        }
    }
}

#[test]
fn a_legend_too_close_to_its_key_is_named() {
    let t = themes();
    let (_, mut v) = views(&t, &SysLook::default()).swap_remove(0);
    v.palette.legend = v.palette.key;
    let low = low(&v, TEST_ROW_PX);
    assert!(
        low.iter().any(|l| l.starts_with("legend on key, rest")),
        "{low:?}"
    );
}

#[test]
fn legends_are_checked_on_a_sculpted_caps_lit_top_and_under_hover_and_press() {
    let t = themes();
    let v = look_view(&t, &choice("dolch", Mode::Light), &SysLook::default(), None).expect("dolch");
    let all = drawn(&v, TEST_ROW_PX);
    let find = |n: &str| all.iter().find(|d| d.name == n).expect(n).clone();
    let latched = find("lock_ink on lock, rest");
    assert!(
        luminance(latched.bg) > luminance(v.palette.lock),
        "white reads worst on the lit top"
    );
    let (rest, hover) = (
        find("legend_2 on key, rest"),
        find("legend_2 on key, hover"),
    );
    assert_ne!(rest.bg, hover.bg, "the hover tint lies under the legend");
    assert!(all.iter().any(|d| d.name == "legend on key, press"));
}

#[test]
fn a_legend_is_measured_where_it_sits_on_the_lit_top_not_at_its_brightest_end() {
    let t = themes();
    let v = look_view(&t, &choice("dolch", Mode::Dark), &SysLook::default(), None).expect("dolch");
    let all = drawn(&v, TEST_ROW_PX);
    let bg = |n: &str| luminance(all.iter().find(|d| d.name == n).expect(n).bg);
    let Cap::Skirt { top, .. } = &v.look.cap else {
        unreachable!("dolch has sculpted caps")
    };
    let end = luminance(Rgba::WHITE.over(v.palette.key, top.lighten));
    let (corner, centre) = (bg("legend_2 on key, rest"), bg("legend on key, rest"));
    assert!(
        corner < end,
        "the corner legend sits below the brightest end"
    );
    assert!(
        centre < end,
        "so does the main legend, even an Arabic one risen over an AltGr legend"
    );
    assert!(
        centre > luminance(v.palette.key),
        "and still on the lit part"
    );
}

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

#[test]
fn the_plate_is_also_checked_see_through_over_a_backdrop_of_its_mode() {
    let t = themes();
    for (at, v) in views(&t, &SysLook::default()) {
        let all = drawn(&v, TEST_ROW_PX);
        let frosted = all
            .iter()
            .find(|d| d.name == "legend_2_plate on plate, frosted");
        let under = WINDOWS[usize::from(v.dark)];
        let plate = v.palette.plate.over(under, v.shape.backdrop.plate);
        assert_eq!(frosted.map(|d| d.bg), Some(plate), "{at}");
    }
    for (at, v) in views(&t, &hc()) {
        assert!(
            drawn(&v, TEST_ROW_PX)
                .iter()
                .all(|d| !d.name.ends_with("frosted")),
            "{at}: no materials"
        );
    }
}

#[test]
fn the_lit_size_letter_is_text_and_the_ring_that_marks_it_a_shape() {
    let t = themes();
    for (at, v) in views(&t, &SysLook::default()) {
        let all = drawn(&v, TEST_ROW_PX);
        let find = |n: &str| all.iter().find(|d| d.name == n).cloned().expect(n);
        let (letter, ring) = (find("lit size letter on plate"), find("ring on plate"));
        assert_eq!(
            (letter.ink, letter.min),
            (v.palette.pop_ink, WCAG_AA),
            "{at}"
        );
        assert_eq!((ring.ink, ring.min), (v.palette.ring, WCAG_SHAPES), "{at}");
    }
}

#[test]
fn the_bubble_the_tray_menu_icons_and_a_hovered_rows_badge_are_checked() {
    let t = themes();
    for sys in [SysLook::default(), hc()] {
        for (at, v) in views(&t, &sys) {
            let (all, p) = (drawn(&v, TEST_ROW_PX), &v.palette);
            let find = |n: &str| all.iter().find(|d| d.name == n).cloned().expect(n);
            // The bubble's disc is the plate, so high contrast draws its icon in WindowText.
            let ink = if v.contrast {
                p.legend_2_plate
            } else {
                p.legend
            };
            let bubble = find("bubble icon on plate");
            assert_eq!((bubble.ink, bubble.min), (ink, WCAG_SHAPES), "{at}");
            let tray = find("tray menu icon on pop_bg");
            let faded = p.pop_ink.over(p.pop_bg, v.shape.menu.icon_share);
            assert_eq!((tray.ink, tray.min), (faded, WCAG_SHAPES), "{at}");
            let hot = find("tray menu icon on pop_hover");
            assert_eq!((hot.bg, hot.min), (p.pop_hover.on(p.pop_bg), WCAG_SHAPES));
            let badge = find("list badge on a hovered row");
            let twice = p.pop_hover.on(p.pop_hover.on(p.pop_bg));
            assert_eq!((badge.ink, badge.bg), (v.sel_ink, twice), "{at}");
        }
    }
}

/// The pairs Windows' contrast themes are made to read: text on its own background, and Highlight marks on a window.
fn system_pairs() -> Vec<(Rgba, Rgba)> {
    let c = |x: [u8; 4]| Rgba::from(x);
    vec![
        (c(HC.window_text), c(HC.window)),
        (c(HC.highlight), c(HC.window)),
        (c(HC.button_text), c(HC.button)),
        (c(HC.highlight_text), c(HC.highlight)),
        (c(HC.window), c(HC.window_text)),
    ]
}

#[test]
fn in_high_contrast_each_opaque_pair_is_one_windows_pairs() {
    let t = themes();
    let pairs = system_pairs();
    let system = [
        HC.window,
        HC.window_text,
        HC.highlight,
        HC.highlight_text,
        HC.button,
        HC.button_text,
    ]
    .map(Rgba::from);
    for (at, v) in views(&t, &hc()) {
        // Faded marks blend their ink; only full-strength inks are system colours.
        let opaque = drawn(&v, TEST_ROW_PX)
            .into_iter()
            .filter(|d| system.contains(&d.ink));
        for d in opaque {
            assert!(
                pairs.contains(&(d.ink, d.bg)),
                "{at}: {} mixes {:?} on {:?}",
                d.name,
                d.ink,
                d.bg
            );
        }
    }
}
