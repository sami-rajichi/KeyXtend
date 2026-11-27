//! Snip's capture windows' own pairs (SnipKey.qml, SnipHint.qml, SnipCount.qml, SnipRecord.qml): each takes the colours
//! its QML draws, and each reads in every theme and mode and in high contrast.

use super::*;
use crate::drawn::snip::{DOT, HOVER, LIGHT, NUMBER, RING, STOP, TAG, tags};

#[test]
fn every_snip_pair_reads_in_every_theme_and_mode_and_in_high_contrast() {
    let t = themes();
    for sys in [SysLook::default(), hc()] {
        for (at, v) in views(&t, &sys) {
            let all = drawn(&v, TEST_ROW_PX);
            for name in [TAG, DOT, STOP, RING, LIGHT, NUMBER, HOVER] {
                let pairs: Vec<_> = all.iter().filter(|d| d.name.starts_with(name)).collect();
                assert!(!pairs.is_empty(), "{at}: {name} is checked");
                for d in pairs {
                    assert!(d.ratio() >= d.min, "{at}: {} is {:.2}:1", d.name, d.ratio());
                }
            }
        }
    }
}

#[test]
fn the_snip_pairs_take_the_colours_their_windows_draw() {
    let t = themes();
    for sys in [SysLook::default(), hc()] {
        for (at, v) in views(&t, &sys) {
            let (all, p, c) = (drawn(&v, TEST_ROW_PX), &v.palette, &v.common);
            let find = |n: &str| all.iter().find(|d| d.name == n).cloned().expect(n);
            let tag = find(TAG);
            assert_eq!((tag.ink, tag.min), (c.snip_tag_ink, WCAG_AA), "{at}");
            assert!(
                tags(&v).contains(&tag.bg),
                "{at}: on a tag over any picture"
            );
            let opaque = tags(&v).iter().all(|b| b.a == 0xFF);
            assert!(
                opaque && (!v.contrast || tags(&v).len() == 1),
                "{at}: one opaque tag in high contrast"
            );
            let dot = find(DOT);
            let red = if v.contrast { p.ring } else { c.rec_dot };
            assert_eq!(
                (dot.ink, dot.bg, dot.min),
                (red, p.pop_bg, WCAG_SHAPES),
                "{at}: as the dictation bar's"
            );
            let stop = find(STOP);
            assert_eq!(
                (stop.ink, stop.bg, stop.min),
                (p.rec_ink, p.rec, WCAG_AA),
                "{at}"
            );
        }
    }
}

#[test]
fn a_record_key_under_the_pointer_takes_sel_ink_on_its_hovered_pill() {
    let t = themes();
    for sys in [SysLook::default(), hc()] {
        for (at, v) in views(&t, &sys) {
            let (all, p) = (drawn(&v, TEST_ROW_PX), &v.palette);
            let hot = all.iter().find(|d| d.name == HOVER).expect(HOVER);
            let want = (v.sel_ink, p.pop_hover.on(p.pop_bg), WCAG_SHAPES);
            assert_eq!(
                (hot.ink, hot.bg, hot.min),
                want,
                "{at}: as SnipPill.qml draws it"
            );
        }
    }
}

#[test]
fn the_ring_and_red_light_sit_on_a_side_keycap_and_take_the_press_text_pressed() {
    let t = themes();
    for sys in [SysLook::default(), hc()] {
        for (at, v) in views(&t, &sys) {
            let (all, p) = (drawn(&v, TEST_ROW_PX), &v.palette);
            let find = |n: &str| all.iter().find(|d| d.name == n).cloned().expect(n);
            for (name, ink) in [(RING, p.ring_act), (LIGHT, p.rec_light)] {
                let (up, down) = (find(name), find(&format!("{name}, press")));
                let cap = [p.key_act, p.hover.on(p.key_act)];
                assert_eq!((up.ink, up.min), (ink, WCAG_SHAPES), "{at}: {name}");
                assert!(
                    cap.contains(&up.bg),
                    "{at}: {name} on a way's cap, at rest or hovered"
                );
                let pressed = p.press_ink.unwrap_or(ink);
                let under = p.press.on(p.key_act);
                assert_eq!(
                    (down.ink, down.bg),
                    (pressed, under),
                    "{at}: {name} pressed, as a legend"
                );
            }
        }
    }
}

#[test]
fn a_step_number_takes_its_inks_number_colour_at_text_contrast() {
    let t = themes();
    for sys in [SysLook::default(), hc()] {
        for (at, v) in views(&t, &sys) {
            let all = drawn(&v, TEST_ROW_PX);
            for (id, ink, number) in v.common.inks() {
                let name = format!("{NUMBER}, {id}");
                let d = all.iter().find(|d| d.name == name).expect(&name);
                assert_eq!((d.ink, d.bg, d.min), (number, ink, WCAG_AA), "{at}: {id}");
            }
        }
    }
}
