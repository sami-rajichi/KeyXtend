//! The clipboard panel's own pairs (ClipRow.qml, ClipFilters.qml, ClipButton.qml): each takes the colours its QML
//! draws, and each reads in every theme and mode and in high contrast.

use super::*;

/// Every clipboard pair, by name.
const PAIRS: [&str; 9] = [
    "clip text on a ticked row",
    "clip mark on a ticked row",
    "clip tick number",
    "clip kind icon",
    "clip kind icon on a lit row",
    "clip expires chip",
    "clip chip under the pointer",
    "clip delete button",
    "clip insert button",
];

#[test]
fn every_clipboard_pair_reads_in_every_theme_and_mode_and_in_high_contrast() {
    let t = themes();
    for sys in [SysLook::default(), hc()] {
        for (at, v) in views(&t, &sys) {
            let all = drawn(&v, TEST_ROW_PX);
            for name in PAIRS {
                let d = all.iter().find(|d| d.name == name);
                let d = d.unwrap_or_else(|| panic!("{at}: {name} is checked"));
                assert!(d.ratio() >= d.min, "{at}: {name} is {:.2}:1", d.ratio());
            }
        }
    }
}

#[test]
fn a_ticked_rows_number_wears_the_badge_and_its_marks_the_ring_or_highlight_text() {
    let t = themes();
    for sys in [SysLook::default(), hc()] {
        for (at, v) in views(&t, &sys) {
            let (all, p) = (drawn(&v, TEST_ROW_PX), &v.palette);
            let find = |n: &str| all.iter().find(|d| d.name == n).cloned().expect(n);
            let tick = find("clip tick number");
            assert_eq!(
                (tick.ink, tick.bg, tick.min),
                (p.badge_ink, p.badge_bg, WCAG_AA),
                "{at}"
            );
            let mark = find("clip mark on a ticked row");
            let ink = if v.contrast { v.sel_ink } else { p.ring };
            assert_eq!((mark.ink, mark.min), (ink, WCAG_SHAPES), "{at}");
            assert_eq!(mark.bg, p.pop_sel.on(p.pop_bg), "{at}: on the ticked fill");
            let insert = find("clip insert button");
            assert_eq!(
                insert.ink, p.enter_ink,
                "{at}: Insert wears Enter's colours"
            );
            let delete = find("clip delete button");
            assert_eq!(
                delete.ink, p.danger_ink,
                "{at}: Delete wears the danger keys'"
            );
        }
    }
}
