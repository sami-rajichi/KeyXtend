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
    look_view(t, &choice, &SysLook::default()).expect("a shipped theme")
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
    let main = spans(&v, Mark::Main)[0];
    assert!(near(main, [7.8525, 16.9525]), "{main:?}");
}

#[test]
fn the_corner_legends_ride_near_the_top_and_the_bottom() {
    let t = themes();
    let v = view(&t, "dolch");
    let all = spans(&v, Mark::Second);
    let (top, bottom) = (all[0], all[1]);
    assert!(near(top, [2.394, 8.0361]), "{top:?}");
    assert!(near(bottom, [17.09, 22.55]), "{bottom:?}");
}

#[test]
fn cycles_dots_fill_the_cap_as_langcycle_lays_them_out() {
    let t = themes();
    let v = view(&t, "dolch");
    // Six dots of 3 px with 2 px gaps fill the 28 px cap, centred 1.5 px up.
    assert_eq!(spans(&v, Mark::Dots), vec![[-1.5, 26.5]]);
}

#[test]
fn a_flat_cap_shows_only_its_own_colour() {
    let t = themes();
    let v = view(&t, "native");
    let key = v.palette.key;
    assert_eq!(under(&v, key, key, [0.0, height(&v)], true), vec![key]);
}

#[test]
fn a_span_off_a_sculpted_top_also_sits_on_its_skirt() {
    let t = themes();
    let v = view(&t, "dolch");
    let (key, skirt) = (v.palette.key, Rgba::BLACK);
    assert!(under(&v, key, skirt, [0.0, height(&v)], false).contains(&skirt));
    assert!(!under(&v, key, skirt, [8.0, 16.0], false).contains(&skirt));
}

#[test]
fn a_pressed_convex_cap_is_lit_from_the_bottom() {
    let t = themes();
    let v = view(&t, "et66");
    let key = v.palette.key;
    let rest = under(&v, key, key, [0.0, height(&v)], false);
    let down = under(&v, key, key, [0.0, height(&v)], true);
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
    let all = under(&v, key, key, [0.0, height(&v)], false);
    assert!(all.contains(&key), "{all:?}");
    assert_eq!(all.len(), 5, "both ends and the three stops");
}
