//! Quick-fill's texts and icons (`QuickFill*.qml`): each takes the colours its QML draws, and each reads at 4.5:1 in
//! every theme and mode and in high contrast, icons too.

use super::*;
use crate::drawn::fill::{CHIP, DANGER, GHOST, INK, LINE, MUTED, SAVE, SHOWN, TILE_ICON};

/// Every Quick-fill pair, by name.
const PAIRS: [&str; 9] = [
    INK, MUTED, TILE_ICON, GHOST, SHOWN, CHIP, SAVE, DANGER, LINE,
];

#[test]
fn every_quick_fill_text_and_icon_reads_at_4_5_in_every_theme_mode_and_high_contrast() {
    let t = themes();
    for sys in [SysLook::default(), hc()] {
        for (at, v) in views(&t, &sys) {
            let all = drawn(&v, TEST_ROW_PX);
            for name in PAIRS {
                let d = all.iter().find(|d| d.name == name);
                let d = d.unwrap_or_else(|| panic!("{at}: {name} is checked"));
                assert!(
                    (d.min - WCAG_AA).abs() < f32::EPSILON,
                    "{at}: {name} reads as text does"
                );
                assert!(d.ratio() >= d.min, "{at}: {name} is {:.2}:1", d.ratio());
            }
        }
    }
}

#[test]
fn a_kind_icon_sits_on_its_tile_in_the_panels_ink_or_highlight_text() {
    let t = themes();
    for sys in [SysLook::default(), hc()] {
        for (at, v) in views(&t, &sys) {
            let (all, p) = (drawn(&v, TEST_ROW_PX), &v.palette);
            let find = |n: &str| all.iter().find(|d| d.name == n).cloned().expect(n);
            let icon = find(TILE_ICON);
            assert_eq!(
                icon.ink, v.sel_ink,
                "{at}: the panel's ink, or Highlight text"
            );
            assert_eq!(icon.bg, p.pop_hover.on(p.pop_bg), "{at}: on its tile");
            assert_eq!(
                find(SAVE).ink,
                p.enter_ink,
                "{at}: Save wears Enter's colours"
            );
            assert_eq!(
                find(DANGER).ink,
                p.danger_ink,
                "{at}: Delete wears the danger keys'"
            );
        }
    }
}

// A line a click aims the letters at takes pop_hover under the pointer, so its texts and icon turn to sel_ink there.
#[test]
fn a_hovered_line_reads_in_highlight_text_on_its_tint() {
    let t = themes();
    for sys in [SysLook::default(), hc()] {
        for (at, v) in views(&t, &sys) {
            let all = drawn(&v, TEST_ROW_PX);
            let line = all.iter().find(|d| d.name == LINE).expect(LINE);
            let p = &v.palette;
            assert_eq!(line.ink, v.sel_ink, "{at}: as ClipRow's hovered row");
            assert_eq!(line.bg, p.pop_hover.on(p.pop_bg), "{at}: on the hover tint");
        }
    }
}
