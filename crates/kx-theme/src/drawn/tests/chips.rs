//! A chip in forget mode on the plate: its word and its forget mark in the danger keys' colours (Chip.qml).

use super::*;

#[test]
fn a_forget_chip_takes_the_danger_keys_colours_and_reads_in_every_theme_and_mode() {
    let t = themes();
    for sys in [SysLook::default(), hc()] {
        for (at, v) in views(&t, &sys) {
            let (all, p) = (drawn(&v, TEST_ROW_PX), &v.palette);
            let find = |n: &str| all.iter().find(|d| d.name == n).cloned().expect(n);
            let word = find("danger_ink on forget chip");
            assert_eq!((word.ink, word.min), (p.danger_ink, WCAG_AA), "{at}");
            let mark = find("forget mark on forget chip");
            assert_eq!((mark.ink, mark.min), (p.danger_ink, WCAG_SHAPES), "{at}");
            for d in [word, mark] {
                assert!(d.ratio() >= d.min, "{at}: {} is {:.2}:1", d.name, d.ratio());
            }
        }
    }
}
