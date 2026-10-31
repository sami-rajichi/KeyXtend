//! A pop-up panel's spot and fit (spec §3.4): above the keyboard, else below, else shrunk to the larger room but never
//! below its least share, and then inside the work area even over the keys.

#![allow(
    clippy::float_cmp,
    reason = "the fit is exactly 1 or the least share where it is clamped"
)]

use super::super::*;

/// The theme's gaps to the keyboard and to the screen edge, and its least fit, at 100 %.
const GAP: [i32; 2] = [10, 8];
const LEAST: f64 = 0.8;

fn rect(x: i32, y: i32, w: i32, h: i32) -> Rect {
    Rect { x, y, w, h }
}

fn size(w: i32, h: i32) -> Size {
    Size { w, h }
}

/// A 1366 by 768 screen's work area above a 48 px taskbar, and a 196 px keyboard docked at its bottom edge.
fn laptop() -> (Rect, Rect) {
    let work = rect(0, 0, 1366, 720);
    (work, rect(348, 720 - 8 - 196, 670, 196))
}

#[test]
fn a_panel_that_fits_above_keeps_its_size_and_sits_there() {
    let (work, kb) = laptop();
    let f = fitted(size(480, 400), None, kb, work, GAP, LEAST);
    assert_eq!(f.fit, 1.0);
    assert_eq!(
        (f.x, f.y),
        (348 + 335 - 240, 516 - 10 - 400),
        "centred on the keyboard"
    );
    let left = fitted(size(480, 400), Some(400), kb, work, GAP, LEAST);
    assert_eq!(left.x, 400, "its left edge where asked");
}

#[test]
fn a_panel_too_tall_above_sits_below_when_it_fits_there() {
    let work = rect(0, 0, 1920, 1040);
    let kb = rect(460, 100, 1000, 300);
    let f = fitted(size(480, 600), None, kb, work, GAP, LEAST);
    assert_eq!((f.fit, f.y), (1.0, 100 + 300 + 10));
}

#[test]
fn a_panel_that_fits_neither_shrinks_to_the_larger_room() {
    let (work, kb) = laptop();
    let room = 516 - 10 - 8;
    let f = fitted(size(480, 600), None, kb, work, GAP, LEAST);
    assert!((f.fit - f64::from(room) / 600.0).abs() < 1e-9, "{}", f.fit);
    assert_eq!(f.y, 8, "its bottom the gap above the keys");
    assert_eq!(f.x, 348 + 335 - 199, "centred at its shrunk width");
}

#[test]
fn a_panel_never_shrinks_below_its_least_share_and_then_stays_inside() {
    let (work, kb) = laptop();
    let f = fitted(size(480, 700), None, kb, work, GAP, LEAST);
    assert_eq!(f.fit, LEAST);
    assert_eq!(f.y, 8, "over the keys, from the top edge of the work area");
    let below = rect(348, 8, 670, 196);
    let low = fitted(size(480, 700), None, below, work, GAP, LEAST);
    assert_eq!(low.fit, LEAST);
    assert_eq!(
        low.y,
        720 - 8 - 560,
        "over the keys, up from the bottom edge, where the room is"
    );
}

#[test]
fn a_work_area_too_small_keeps_the_panel_at_its_top_left_inside() {
    let work = rect(100, 50, 300, 200);
    let kb = rect(100, 150, 300, 100);
    let f = fitted(size(480, 600), None, kb, work, GAP, LEAST);
    assert_eq!(f.fit, LEAST);
    assert_eq!(
        (f.x, f.y),
        (108, 58),
        "the edge gap inside the work area's corner"
    );
}

#[test]
fn odd_sizes_never_panic_and_fit_stays_a_share() {
    let (work, kb) = laptop();
    for s in [size(0, 0), size(-5, -5), size(i32::MAX, i32::MAX)] {
        let f = fitted(s, Some(i32::MIN), kb, work, GAP, LEAST);
        assert!((LEAST..=1.0).contains(&f.fit), "{s:?}: {}", f.fit);
    }
}
