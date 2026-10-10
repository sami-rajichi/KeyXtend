//! Where legends sit on each kind of cap, and the colours a span of a cap shows.

use kx_platform::ui::SysLook;

use super::*;
use crate::Themes;
use crate::contrast::{WCAG_AA, luminance};
use crate::mode::ModeChoice;
use crate::view::{LookChoice, look_view};

/// How close two positions must be, in px.
const NEAR: f32 = 0.01;

/// The shipped themes.
fn themes() -> Themes {
    Themes::shipped().expect("tokens load")
}

/// Theme `id` in light mode, with its own colours.
fn view<'a>(t: &'a Themes, id: &str) -> LookView<'a> {
    let choice = LookChoice {
        theme: id.into(),
        mode: ModeChoice::Light,
        follow_accent: false,
        reduced_motion: false,
        min_contrast: WCAG_AA,
    };
    look_view(t, &choice, &SysLook::default(), None).expect("a shipped theme")
}

/// The cap's height on the test board's rows.
fn cap(v: &LookView) -> f32 {
    height(v, crate::drawn::TEST_ROW_PX)
}

/// True when spans `a` and `b` match within `NEAR`.
fn near(a: Span, b: Span) -> bool {
    a.iter().zip(b).all(|(x, y)| (x - y).abs() < NEAR)
}

#[test]
fn a_main_legend_sits_where_its_fonts_metrics_put_it() {
    let t = themes();
    let v = view(&t, "dolch");
    // Dolch: a 28 px cap, lifted 1.5 px; Rubik at 13 px has its baseline 4.45 px below the middle.
    let main = spans(&v, cap(&v), Mark::Main)[0];
    assert!(near(main, [7.8525, 16.9525]), "{main:?}");
}

#[test]
fn the_corner_legends_sit_in_from_the_face_the_theme_draws() {
    let t = themes();
    // Dolch: 2.5 px in from its lit top, which runs from 1 px below the 28 px cap's top to 4 px above its bottom;
    // the corners are 6.448 px and 6.24 px.
    let v = view(&t, "dolch");
    let all = spans(&v, cap(&v), Mark::Second);
    let (top, bottom) = (all[0], all[1]);
    assert!(near(top, [5.01528, 9.52888]), "{top:?}");
    assert!(near(bottom, [15.572, 19.94]), "{bottom:?}");
    // Native: 2.5 px in from the flat cap itself; Segoe UI's ascent 1.079 and cap height 0.7 at 6.448 px.
    let v = view(&t, "native");
    let top = spans(&v, cap(&v), Mark::Second)[0];
    assert!(near(top, [4.943_792, 9.457_392]), "{top:?}");
}

#[test]
fn cycles_dots_fill_the_cap_as_langcycle_lays_them_out() {
    let t = themes();
    let v = view(&t, "dolch");
    // Five dots of 3 px with 2 px gaps fit the 28 px cap with room for the lit dot's 4.5 px, centred 1.5 px up.
    assert_eq!(spans(&v, cap(&v), Mark::Dots), vec![[1.0, 24.0]]);
}

#[test]
fn a_flat_cap_shows_only_its_own_colour() {
    let t = themes();
    let v = view(&t, "native");
    let key = v.palette.key;
    assert_eq!(
        under(&v, cap(&v), key, key, [0.0, cap(&v)], true),
        vec![key]
    );
}

#[test]
fn a_span_off_a_sculpted_top_also_sits_on_its_skirt() {
    let t = themes();
    let v = view(&t, "dolch");
    let (key, skirt) = (v.palette.key, Rgba::BLACK);
    assert!(under(&v, cap(&v), key, skirt, [0.0, cap(&v)], false).contains(&skirt));
    assert!(!under(&v, cap(&v), key, skirt, [8.0, 16.0], false).contains(&skirt));
}

#[test]
fn a_pressed_convex_cap_is_lit_from_the_bottom() {
    let t = themes();
    let v = view(&t, "et66");
    let key = v.palette.key;
    let rest = under(&v, cap(&v), key, key, [0.0, cap(&v)], false);
    let down = under(&v, cap(&v), key, key, [0.0, cap(&v)], true);
    assert!(
        luminance(rest[0]) > luminance(rest[1]),
        "lit at the top at rest"
    );
    assert!(
        luminance(down[0]) < luminance(down[1]),
        "lit at the bottom pressed"
    );
}

#[test]
fn a_span_between_stops_also_shows_the_stop_inside_it() {
    let t = themes();
    let v = view(&t, "et66");
    let key = v.palette.key;
    // The plain key colour sits at the rest gradient's middle stop, inside a span across the whole cap.
    let all = under(&v, cap(&v), key, key, [0.0, cap(&v)], false);
    assert!(all.contains(&key), "{all:?}");
    assert_eq!(all.len(), 5, "both ends and the three stops");
}
