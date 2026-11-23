//! The dictation bar's own pairs (VoiceBar.qml, VoiceWave.qml, VoiceNote.qml): each takes the colours its QML draws,
//! and each reads in every theme and mode and in high contrast.

use super::*;
use crate::drawn::voice::{CHIP, DOT, METER};

#[test]
fn every_dictation_pair_reads_in_every_theme_and_mode_and_in_high_contrast() {
    let t = themes();
    for sys in [SysLook::default(), hc()] {
        for (at, v) in views(&t, &sys) {
            let all = drawn(&v, TEST_ROW_PX);
            for name in [CHIP, DOT, METER] {
                let d = all.iter().find(|d| d.name == name);
                let d = d.unwrap_or_else(|| panic!("{at}: {name} is checked"));
                assert!(d.ratio() >= d.min, "{at}: {name} is {:.2}:1", d.ratio());
            }
        }
    }
}

#[test]
fn the_amber_dictation_keys_label_and_icon_are_checked_in_every_state() {
    let t = themes();
    for sys in [SysLook::default(), hc()] {
        for (at, v) in views(&t, &sys) {
            let all = drawn(&v, TEST_ROW_PX);
            for state in STATES {
                for name in ["prep_ink on prep", "prep_ink icon on prep"] {
                    let name = format!("{name}, {state}");
                    assert!(all.iter().any(|d| d.name == name), "{at}: {name}");
                }
            }
        }
    }
}

#[test]
fn the_dot_meter_and_chip_take_the_mock_ups_colours_and_system_ones_in_high_contrast() {
    let t = themes();
    for sys in [SysLook::default(), hc()] {
        for (at, v) in views(&t, &sys) {
            let (all, p) = (drawn(&v, TEST_ROW_PX), &v.palette);
            let find = |n: &str| all.iter().find(|d| d.name == n).cloned().expect(n);
            let red = if v.contrast { p.ring } else { v.common.rec_dot };
            let dot = find(DOT);
            assert_eq!(
                (dot.ink, dot.bg, dot.min),
                (red, p.pop_bg, WCAG_SHAPES),
                "{at}"
            );
            let meter = find(METER);
            let track = if v.contrast {
                p.pop_bg
            } else {
                p.pop_hover.on(p.pop_bg)
            };
            assert_eq!((meter.ink, meter.bg), (p.ring, track), "{at}");
            let chip = find(CHIP);
            let (bg, ink) = (
                p.chip_bg.on(p.pop_bg),
                if v.contrast { p.legend } else { p.pop_ink },
            );
            assert_eq!((chip.ink, chip.bg, chip.min), (ink, bg, WCAG_AA), "{at}");
        }
    }
}
