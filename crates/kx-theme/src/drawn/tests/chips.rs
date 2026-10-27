//! A chip in forget mode on the plate: its word and its forget mark take `danger_ink`, so the word's AA check covers
//! the mark too (Chip.qml).

use super::*;

#[test]
fn a_forget_chip_takes_the_danger_keys_colours_and_reads_in_every_theme_and_mode() {
    let t = themes();
    for sys in [SysLook::default(), hc()] {
        for (at, v) in views(&t, &sys) {
            let (all, p) = (drawn(&v, TEST_ROW_PX), &v.palette);
            let word = all.iter().find(|d| d.name == "danger_ink on forget chip");
            let word = word.cloned().expect("the forget chip's word");
            assert_eq!((word.ink, word.min), (p.danger_ink, WCAG_AA), "{at}");
            assert!(word.ratio() >= word.min, "{at}: {:.2}:1", word.ratio());
        }
    }
}
