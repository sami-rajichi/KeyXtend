//! Where the Voice panel goes (owner: above the bar): the gap above the bar and centred on it, else below it, never over
//! the bar or the keyboard while it fits; shrunk to the larger room, never below the theme's least fit, and always on
//! the work area; beside the keyboard while no bar shows.

use super::{heard, listening_bar, page};
use crate::{Rig, main_screen};
use kx_module_api::ports::{BarPlace, Rect, physical};
use kx_theme::Themes;
use kx_ui::Fitted;
use proptest::prelude::*;

/// The panel's size in Qt units, at 100 % one pixel each.
const W: f64 = 600.0;
const H: f64 = 400.0;
/// The bar's size in pixels.
const BAR_W: i32 = 400;
const BAR_H: i32 = 56;
/// The sizes S, M and L scale the panel by.
const SIZES: [f64; 3] = [1.0, 1.25, 1.5];
/// Cases the property tries.
const CASES: u32 = 128;

fn rect(x: i32, y: i32, w: i32, h: i32) -> Rect {
    Rect { x, y, w, h }
}

fn fitted(x: i32, y: i32, fit: f64) -> Fitted {
    Fitted { x, y, fit }
}

/// The theme's gaps, to the keyboard and to the screen edge, in pixels at 100 %, and its least fit.
fn theme() -> ([i32; 2], f64) {
    let themes = Themes::shipped().expect("the shipped themes");
    let p = &themes.shape.panel;
    (
        p.gap_px.map(|g| physical(f64::from(g), main_screen().dpi)),
        f64::from(p.min_fit),
    )
}

/// The bottom of the main screen's work area.
fn floor() -> i32 {
    let s = main_screen().rect;
    s.y + s.h
}

/// True when the boxes share a pixel.
fn meets(a: Rect, b: Rect) -> bool {
    a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h
}

/// The box a panel of `w` by `h` takes at `p`.
fn taken(p: Fitted, w: f64, h: f64) -> Rect {
    #[allow(clippy::cast_possible_truncation, reason = "test sizes are small")]
    let size = |v: f64| (v * p.fit).round() as i32;
    rect(p.x, p.y, size(w), size(h))
}

/// The bar's box centred over the keyboard `kb`, the gap above it.
fn bar_over(kb: Rect) -> Rect {
    let gap = theme().0[0];
    rect(kb.x + (kb.w - BAR_W) / 2, kb.y - gap - BAR_H, BAR_W, BAR_H)
}

#[test]
fn the_panel_sits_the_gap_above_the_bar_centred_on_it() {
    let rig = Rig::new();
    let kb = rect(300, 640, 1000, 400);
    let bar = bar_over(kb);
    let gap = theme().0[0];
    let want = fitted(800 - 300, bar.y - gap - 400, 1.0);
    assert_eq!(
        rig.hub.voice_panel_at(Some(kb), Some(bar), W, H),
        Some(want)
    );
}

#[test]
fn with_the_bar_at_the_top_of_the_screen_it_opens_below_the_bar_above_the_keyboard() {
    let rig = Rig::new();
    let ([gap, edge], _) = theme();
    let kb = rect(300, 600, 1000, floor() - 600);
    let bar = rect((1920 - BAR_W) / 2, edge, BAR_W, BAR_H);
    let p = rig.hub.voice_panel_at(Some(kb), Some(bar), W, H);
    assert_eq!(p, Some(fitted(960 - 300, edge + BAR_H + gap, 1.0)));
    let p = taken(p.unwrap(), W, H);
    assert!(!meets(p, kb) && !meets(p, bar), "{p:?}");
}

#[test]
fn with_the_keyboard_at_the_top_and_the_bar_at_the_bottom_it_opens_between_them() {
    let rig = Rig::new();
    let ([gap, edge], _) = theme();
    let kb = rect(300, edge, 1000, 400);
    let bar = rect((1920 - BAR_W) / 2, floor() - edge - BAR_H, BAR_W, BAR_H);
    let p = rig.hub.voice_panel_at(Some(kb), Some(bar), W, H);
    assert_eq!(p, Some(fitted(960 - 300, bar.y - gap - 400, 1.0)));
    let p = taken(p.unwrap(), W, H);
    assert!(!meets(p, kb) && !meets(p, bar), "{p:?}");
}

#[test]
fn with_no_bar_it_opens_beside_the_keyboard_as_every_panel_does() {
    let rig = Rig::new();
    let kb = rect(300, 640, 1000, 400);
    let want = rig.hub.pop_fit(Some(kb), None, W, H);
    assert_eq!(rig.hub.voice_panel_at(Some(kb), None, W, H), want);
    let home = rig.hub.pop_fit(None, None, W, H);
    assert_eq!(rig.hub.voice_panel_at(None, None, W, H), home);
}

#[test]
fn with_too_little_room_it_shrinks_never_below_the_least_fit_and_stays_on_the_screen() {
    let rig = Rig::new();
    let ([gap, edge], least) = theme();
    let kb = rect(300, 440, 1000, floor() - 440);
    let bar = bar_over(kb);
    let room = bar.y - gap - edge;
    let p = rig.hub.voice_panel_at(Some(kb), Some(bar), W, H).unwrap();
    assert!((p.fit - f64::from(room) / H).abs() < 1e-9, "{p:?}");
    assert_eq!(p.y, edge, "shrunk into the room above the bar");
    assert!(!meets(taken(p, W, H), bar));
    let low = rect(300, 300, 1000, floor() - 300);
    let p = rig
        .hub
        .voice_panel_at(Some(low), Some(bar_over(low)), W, H)
        .unwrap();
    assert!((p.fit - least).abs() < 1e-9, "{p:?}");
    let t = taken(p, W, H);
    assert!(t.y >= edge && t.y + t.h <= floor() - edge, "{t:?}");
}

#[test]
fn with_no_screen_there_is_no_spot() {
    let rig = Rig::new();
    rig.screens.set_areas(Vec::new());
    let kb = rect(0, 500, 10, 10);
    assert_eq!(
        rig.hub.voice_panel_at(Some(kb), Some(bar_over(kb)), W, H),
        None
    );
}

fn place() -> impl Strategy<Value = BarPlace> {
    prop_oneof![
        Just(BarPlace::Above),
        Just(BarPlace::Bottom),
        Just(BarPlace::Top)
    ]
}

/// A keyboard box wholly on the main screen.
fn keyboard() -> impl Strategy<Value = Rect> {
    let s = main_screen().rect;
    (0..s.w, 0..s.h)
        .prop_flat_map(move |(x, y)| (Just(x), Just(y), 1..=s.w - x, 1..=s.h - y))
        .prop_map(|(x, y, w, h)| rect(x, y, w, h))
}

/// A hub showing a listening bar at `place`, and the bar's box by the keyboard `kb`.
fn bar_by(place: BarPlace, kb: Rect) -> (Rig, Rect) {
    let rig = Rig::new();
    let heard = heard(&rig);
    let mut shown = page(1, listening_bar(), None);
    shown.place = place;
    heard.view(shown);
    let at = rig
        .hub
        .voice_at(Some(kb), f64::from(BAR_W), f64::from(BAR_H));
    let at = at.expect("a spot");
    (rig, rect(at.x, at.y, BAR_W, BAR_H))
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(CASES))]

    /// For any place, keyboard and size, the panel lands inside the work area's edge gap, and whenever it fits whole it
    /// covers neither the bar nor the keyboard.
    #[test]
    fn the_panel_stays_on_the_screen_and_off_the_bar_and_the_keyboard(
        place in place(),
        kb in keyboard(),
        size in 0..SIZES.len(),
    ) {
        let (rig, bar) = bar_by(place, kb);
        let (w, h) = (W * SIZES[size], H * SIZES[size]);
        let p = rig.hub.voice_panel_at(Some(kb), Some(bar), w, h).expect("a spot");
        let t = taken(p, w, h);
        let edge = theme().0[1];
        prop_assert!(t.x >= edge && t.x + t.w <= 1920 - edge, "{:?}", t);
        prop_assert!(t.y >= edge && t.y + t.h <= floor() - edge, "{:?}", t);
        prop_assert!(p.fit >= theme().1 && p.fit <= 1.0, "{:?}", p);
        let clear = !meets(bar, kb);
        if clear && (p.fit - 1.0).abs() < f64::EPSILON {
            prop_assert!(!meets(t, bar) && !meets(t, kb), "{:?} by {:?} and {:?}", t, bar, kb);
        }
    }
}
