//! The scroll pad as drawn: a pressed arrow on the keys' press shade with their press text, and Stop's speed dots,
//! whose unlit one is faded in high contrast too, as Cycle's.

use super::*;

/// Each look: every theme in both modes, then in high contrast.
fn every_view(t: &Themes) -> Vec<(String, LookView<'_>)> {
    let mut all = views(t, &SysLook::default());
    all.extend(
        views(t, &hc())
            .into_iter()
            .map(|(at, v)| (format!("{at} hc"), v)),
    );
    all
}

#[test]
fn a_pressed_arrow_sits_on_the_keys_press_shade_and_takes_their_press_text() {
    let t = themes();
    for (at, v) in every_view(&t) {
        let (d, p) = (&v.dpad, &v.palette);
        let all = drawn(&v, TEST_ROW_PX);
        for (name, ink, face) in [
            ("D-pad arrow", d.ink, d.face),
            ("D-pad lit arrow", d.on_ink, d.on),
        ] {
            let pressed = format!("{name}, press");
            let got = all.iter().find(|x| x.name == pressed);
            let got = got.unwrap_or_else(|| panic!("{at}: {pressed} is drawn"));
            assert_eq!(got.bg, p.press.on(face), "{at}: {pressed}");
            assert_eq!(got.ink, p.press_ink.unwrap_or(ink), "{at}: {pressed}");
        }
    }
}

#[test]
fn an_unlit_speed_dot_is_drawn_against_a_lit_one_in_every_look() {
    let t = themes();
    // High contrast fades it too, as Cycle's dots, since a hollow ring that small reads as filled.
    for (at, v) in every_view(&t) {
        let (d, share) = (&v.dpad, v.shape.dpad.dot_off);
        let all = drawn(&v, TEST_ROW_PX);
        let dot = all.iter().find(|x| x.name == SPEED_DOT);
        let dot = dot.unwrap_or_else(|| panic!("{at}: {SPEED_DOT} is drawn"));
        let pair = (d.stop_ink.over(d.stop, share), d.stop_ink, WCAG_SHAPES);
        assert_eq!((dot.ink, dot.bg, dot.min), pair, "{at}");
    }
}

#[test]
fn a_speed_dot_faded_too_little_is_named() {
    let t = themes();
    let (_, v) = views(&t, &SysLook::default()).swap_remove(0);
    let mut shape = v.shape.clone();
    shape.dpad.dot_off = 1.0;
    let v = LookView { shape: &shape, ..v };
    let low = low(&v, TEST_ROW_PX);
    assert!(low.iter().any(|l| l.starts_with(SPEED_DOT)), "{low:?}");
}
