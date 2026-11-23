//! The Voice panel's own pairs (VoiceMark.qml, VoiceSwitch.qml, VoiceProgress.qml, VoiceBand.qml): each takes the
//! colours its QML draws, on every row a mark sits on, and each reads in every theme and mode and in high contrast.

use super::*;
use crate::drawn::voicepanel::{
    BAND, BAND_ICON, LIT_PROGRESS, MARK, MARK_ICON, MARK_LIT, MARK_OUT, PROGRESS, SWITCH_OFF,
    SWITCH_ON, WARN_BAND, WARN_WORDS,
};

/// Every pair the panel adds.
const PAIRS: [&str; 12] = [
    BAND,
    BAND_ICON,
    LIT_PROGRESS,
    MARK,
    MARK_ICON,
    MARK_LIT,
    MARK_OUT,
    PROGRESS,
    SWITCH_OFF,
    SWITCH_ON,
    WARN_BAND,
    WARN_WORDS,
];

#[test]
fn every_voice_panel_pair_reads_in_every_theme_and_mode_and_in_high_contrast() {
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
fn a_mark_takes_the_chip_colours_and_high_contrast_draws_it_in_button_text() {
    let t = themes();
    for sys in [SysLook::default(), hc()] {
        for (at, v) in views(&t, &sys) {
            let (all, p) = (drawn(&v, TEST_ROW_PX), &v.palette);
            let find = |n: &str| all.iter().find(|d| d.name == n).cloned().expect(n);
            let (word, icon) = if v.contrast {
                (p.legend, p.legend)
            } else {
                (p.pop_ink, p.ring)
            };
            assert_eq!((find(MARK).ink, find(MARK).min), (word, WCAG_AA), "{at}");
            assert_eq!(find(MARK_ICON).ink, icon, "{at}");
            let lit = find(MARK_LIT);
            assert_eq!((lit.ink, lit.bg), (p.badge_ink, p.badge_bg), "{at}");
            if v.contrast {
                assert_eq!(find(PROGRESS).bg, p.pop_bg, "{at}: a window track");
            }
            assert_eq!(find(PROGRESS).ink, p.ring, "{at}");
        }
    }
}

#[test]
fn a_warning_band_reads_in_the_panels_ink_and_its_words_in_warn_ink() {
    let t = themes();
    for sys in [SysLook::default(), hc()] {
        for (at, v) in views(&t, &sys) {
            let (all, p) = (drawn(&v, TEST_ROW_PX), &v.palette);
            let find = |n: &str| all.iter().find(|d| d.name == n).cloned().expect(n);
            let ink = if v.contrast { v.sel_ink } else { p.pop_ink };
            assert_eq!(
                (find(WARN_BAND).ink, find(WARN_BAND).bg),
                (ink, p.warn),
                "{at}"
            );
            assert_eq!(
                (find(WARN_WORDS).ink, find(WARN_WORDS).bg),
                (p.warn_ink, p.pop_bg),
                "{at}"
            );
        }
    }
}
