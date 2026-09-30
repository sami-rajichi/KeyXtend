//! Contrast tests against the shipped themes: every legend, the rings, and the WCAG maths.

use super::*;
use crate::Themes;
use crate::rgba::Rgba;

/// Plain Windows 11 app backgrounds in light and dark mode, which the ring is drawn over.
const WINDOW_BG: [&str; 2] = ["#F3F3F3", "#202020"];

fn load() -> Themes {
    Themes::shipped().expect("tokens load")
}

#[test]
fn every_ring_stands_out_on_a_plain_window_of_its_mode() {
    let bg = WINDOW_BG.map(|c| Rgba::parse(c).expect("window colour"));
    for t in load().list {
        let id = &t.look.id;
        assert!(contrast(t.light.ring, bg[0]) >= WCAG_SHAPES, "{id} light");
        assert!(contrast(t.dark.ring, bg[1]) >= WCAG_SHAPES, "{id} dark");
    }
}

#[test]
fn contrast_follows_wcag() {
    let (black, white) = (Rgba::parse("#000000"), Rgba::parse("#FFFFFF"));
    let (black, white) = (black.expect("black"), white.expect("white"));
    assert!((contrast(black, white) - 21.0).abs() < 0.01);
    assert!(
        (contrast(white, black) - 21.0).abs() < 0.01,
        "order does not matter"
    );
    assert!((contrast(white, white) - 1.0).abs() < 0.01);
}

#[test]
fn every_legend_reaches_wcag_aa() {
    let low = low_contrast(&load().list, WCAG_AA);
    assert!(low.is_empty(), "{low:?}");
}

#[test]
fn text_on_the_plate_has_its_own_token_and_reaches_wcag_aa() {
    let mut seen = 0;
    for t in load().list {
        for mode in Mode::ALL {
            let (p, at) = (t.palette(mode), format!("{}.{}", t.look.id, mode.key()));
            let on_plate = p
                .legend_pairs()
                .into_iter()
                .filter(|(n, ..)| n.ends_with(" on plate"));
            for (pair, ink, bg) in on_plate {
                assert!(pair.starts_with("legend_2_plate "), "{at}: {pair}");
                assert!(contrast(ink, bg) >= WCAG_AA, "{at}: {pair}");
                seen += 1;
            }
        }
    }
    assert_eq!(seen, 6, "one plate pair per theme and mode");
}

#[test]
fn chip_words_and_icons_reach_wcag_aa_on_the_chip() {
    for t in load().list {
        for mode in Mode::ALL {
            let p = t.palette(mode);
            let on_chip: Vec<_> = p
                .legend_pairs()
                .into_iter()
                .filter(|(n, ..)| n.ends_with(" on chip"))
                .collect();
            let names: Vec<&str> = on_chip.iter().map(|(n, ..)| *n).collect();
            assert_eq!(names, ["legend on chip", "legend_2_plate on chip"]);
            let chip = p.chip_bg.over(p.plate, p.chip_bg.share());
            for (pair, ink, bg) in on_chip {
                assert_eq!(bg, chip, "{pair}: the chip lies over the plate");
                assert!(contrast(ink, bg) >= WCAG_AA, "{}: {pair}", t.look.id);
            }
        }
    }
}

#[test]
fn a_legend_too_close_to_its_key_is_named() {
    let mut all = load().list;
    all[0].light.legend = all[0].light.key;
    let low = low_contrast(&all, WCAG_AA);
    assert!(
        low.iter()
            .any(|l| l.starts_with("native.light: legend on key")),
        "{low:?}"
    );
    assert!(low_contrast(&all, 1.0).is_empty(), "nothing is below 1:1");
}

#[test]
fn a_user_minimum_starts_at_wcag_aa_and_ends_at_black_on_white() {
    let (black, white) = (Rgba::parse("#000000"), Rgba::parse("#FFFFFF"));
    let (black, white) = (black.expect("black"), white.expect("white"));
    assert!((contrast(black, white) - MAX_CONTRAST).abs() < 0.01);
    assert!((MIN_CONTRAST.start() - WCAG_AA).abs() < f32::EPSILON);
    assert!(MIN_CONTRAST.contains(&MAX_CONTRAST));
    for bad in [4.4, MAX_CONTRAST + 1.0, f32::NAN] {
        assert!(!MIN_CONTRAST.contains(&bad), "{bad}");
    }
}

#[test]
fn the_list_ring_and_tick_stand_out_on_the_panel() {
    let low = low_shapes(&load().list);
    assert!(low.is_empty(), "{low:?}");
    for t in load().list {
        for mode in Mode::ALL {
            let pairs = t.palette(mode).shape_pairs();
            let names: Vec<&str> = pairs.iter().map(|(n, ..)| *n).collect();
            assert_eq!(names, ["ring on pop_bg"], "{}", t.look.id);
        }
    }
}

#[test]
fn a_ring_too_close_to_the_panel_is_named() {
    let mut all = load().list;
    all[0].dark.ring = all[0].dark.pop_bg;
    let low = low_shapes(&all);
    assert!(
        low.iter()
            .any(|l| l.starts_with("native.dark: ring on pop_bg")),
        "{low:?}"
    );
}
