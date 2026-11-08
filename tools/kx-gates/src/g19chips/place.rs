//! Where the strip should sit (P10 design "Where it sits"): the selection's line boxes joined as the platform joins
//! them, then `geom::anchor` on that screen's work area with the theme's gap for the keyboard's size, clear of the
//! plate, landed where Qt puts a window; and its verdict.
#![cfg(windows)]

use kx_module_api::ports::{BASE_DPI, Point, Rect, Strip, anchor, physical};
use kx_platform_windows::screen::{edges, rect_of};
use kx_theme::Shape;
use serde_json::{Value, json};
use windows::Win32::Foundation::RECT;

/// The theme's gap from the text for a keyboard at `size` on a screen of `dpi`, in physical pixels, as the hub sets it.
pub fn gap(shape: &Shape, size: f64, dpi: u32) -> i32 {
    physical(shape.chips.gap(size), dpi)
}

/// The screen's scale at `dpi`.
fn scale(dpi: u32) -> f64 {
    f64::from(dpi) / f64::from(BASE_DPI)
}

/// One axis of `snap`: QML's `Math.round` of the logical place, then Qt's rounding back to physical pixels.
#[allow(
    clippy::cast_possible_truncation,
    reason = "Screen places fit an i32 by far."
)]
fn snapped(p: i32, origin: i32, d: f64) -> i32 {
    let o = f64::from(origin);
    let logical = (o + (f64::from(p) - o) / d + 0.5).floor();
    (o + ((logical - o) * d).round()) as i32
}

/// Where Qt puts a window that kx-ui asks for at physical `p` on a screen whose top-left is `origin`: the strip's
/// `place()` takes the logical place (`Spots.logical`) and rounds it, and Qt scales it back.
pub fn snap(p: Point, origin: (i32, i32), dpi: u32) -> Point {
    let d = scale(dpi);
    Point {
        x: snapped(p.x, origin.0, d),
        y: snapped(p.y, origin.1, d),
    }
}

/// The keyboard's plate: from its window's top-left to its resize corner's middle, which sits on the plate's corner.
pub fn plate(win: Rect, corner: Rect) -> Rect {
    let mid = |at: i32, len: i32| at.saturating_add(len / 2);
    Rect {
        w: mid(corner.x, corner.w).saturating_sub(win.x).max(0),
        h: mid(corner.y, corner.h).saturating_sub(win.y).max(0),
        ..win
    }
}

/// The keyboard's size from its resize corner, `corner_w` physical px wide on a screen of `dpi`, whose width is the
/// theme's `corner_px` at that size.
pub fn size_of(corner_w: i32, corner_px: f32, dpi: u32) -> f64 {
    f64::from(corner_w) / (f64::from(corner_px) * scale(dpi))
}

/// `r` as the ports hold a box; an upside-down box gets no size.
pub fn rect(r: &RECT) -> Rect {
    let side = |from: i32, to: i32| to.saturating_sub(from).max(0);
    Rect {
        x: r.left,
        y: r.top,
        w: side(r.left, r.right),
        h: side(r.top, r.bottom),
    }
}

/// The height at the middle of `r`.
fn middle(r: Rect) -> i32 {
    r.y.saturating_add(r.h / 2)
}

/// The smallest box that holds `a` and `b`.
fn join(a: Rect, b: Rect) -> Rect {
    let ([al, at, ar, ab], [bl, bt, br, bb]) = (edges(a), edges(b));
    rect_of([al.min(bl), at.min(bt), ar.max(br), ab.max(bb)])
}

/// The box of the line that holds `rects[at]`: it and every part whose middle lies within its height, joined.
pub fn line(rects: &[Rect], at: usize) -> Option<Rect> {
    let line = *rects.get(at)?;
    let [_, top, _, bottom] = edges(line);
    let on = rects
        .iter()
        .filter(|r| (top..bottom).contains(&middle(**r)));
    Some(on.fold(line, |joined, part| join(joined, *part)))
}

/// Where a `size` strip goes for a selection of `boxes`, `gap` from its line on `area`, clear of `clear`; none with
/// no box.
pub fn want(
    boxes: &[Rect],
    (w, h): (i32, i32),
    gap: i32,
    area: Rect,
    clear: &[Rect],
) -> Option<Point> {
    let first = line(boxes, 0)?;
    let last = line(boxes, boxes.len().checked_sub(1)?)?;
    Some(anchor(first, last, Strip { w, h, gap }, area, clear))
}

/// True when `a` and `b` share a pixel.
fn overlaps(a: Rect, b: Rect) -> bool {
    let ([al, at, ar, ab], [bl, bt, br, bb]) = (edges(a), edges(b));
    al < br && bl < ar && at < bb && bt < ab
}

/// The verdict on the strip at `got` against `want`: within `slack` px each way, clear of every box of `ours`, and
/// whether it sits above the selection's `first` line.
pub fn judge(want: Point, got: Rect, slack: i32, ours: &[Rect], first: Rect) -> Value {
    let off = got.x.abs_diff(want.x).max(got.y.abs_diff(want.y));
    let clear = ours.iter().all(|k| !overlaps(got, *k));
    let above = got.y.saturating_add(got.h) <= first.y;
    let near = i64::from(off) <= i64::from(slack);
    json!({ "ok": near && clear, "off_px": off, "clear": clear, "above": above, "want": [want.x, want.y] })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A box at (`x`, `y`) of `w` by `h`.
    fn r(x: i32, y: i32, w: i32, h: i32) -> Rect {
        Rect { x, y, w, h }
    }

    /// A 1920 by 1032 work area.
    const AREA: Rect = Rect {
        x: 0,
        y: 0,
        w: 1920,
        h: 1032,
    };

    /// The mock-up's `place()` sets the strip `8 * S.size` from the text, at the screen's DPI.
    #[test]
    fn the_gap_grows_with_the_keyboards_size_and_the_screens_dpi() {
        use kx_module_api::ports::BASE_DPI;
        let shape = kx_theme::Themes::shipped().expect("themes").shape;
        let token = f64::from(shape.chips.gap_px);
        for (size, dpi) in [(1.0, BASE_DPI), (1.5, BASE_DPI), (1.5, BASE_DPI * 5 / 4)] {
            assert_eq!(
                gap(&shape, size, dpi),
                physical(token * size, dpi),
                "{size} at {dpi}"
            );
        }
        assert!(gap(&shape, 1.5, BASE_DPI) > gap(&shape, 1.0, BASE_DPI));
    }

    /// The platform's own selection spot, whose line join the gate copies.
    const SPOT: &str = include_str!("../../../../crates/kx-platform-windows/src/selection/spot.rs");
    /// This file.
    const OWN: &str = include_str!("place.rs");

    /// The text of `f` in `src`, from its name to its closing brace.
    fn body<'a>(src: &'a str, f: &str) -> &'a str {
        let from = src.find(f).expect(f);
        let len = src[from..].find("\n}\n").expect("an end");
        &src[from..from + len]
    }

    #[test]
    fn the_line_join_is_the_platforms_own() {
        for f in ["fn line(", "fn middle(", "fn join("] {
            assert_eq!(body(OWN, f), body(SPOT, f), "{f}");
        }
    }

    #[test]
    fn a_place_lands_where_qt_puts_a_window_at_its_rounded_logical_place() {
        use kx_module_api::ports::BASE_DPI;
        let at = |x, y| Point { x, y };
        assert_eq!(snap(at(102, 57), (0, 0), BASE_DPI), at(102, 57), "100 %");
        let five_quarters = BASE_DPI * 5 / 4;
        assert_eq!(snap(at(101, 0), (0, 0), five_quarters), at(101, 0));
        assert_eq!(
            snap(at(102, 0), (0, 0), five_quarters),
            at(103, 0),
            "81.6 rounds to 82"
        );
        assert_eq!(
            snap(at(1922, 2), (1920, 0), five_quarters),
            at(1923, 3),
            "the second screen's own origin"
        );
        assert_eq!(snap(at(-1918, 0), (-1920, 0), five_quarters), at(-1917, 0));
    }

    #[test]
    fn the_plate_ends_at_the_corners_middle_and_the_corner_tells_the_size() {
        use kx_module_api::ports::BASE_DPI;
        let win = r(100, 600, 830, 330);
        let corner = r(895, 895, 30, 30);
        assert_eq!(plate(win, corner), r(100, 600, 810, 310));
        let five_quarters = BASE_DPI * 5 / 4;
        assert!((size_of(25, 20.0, five_quarters) - 1.0).abs() < 1e-9);
        assert!((size_of(30, 20.0, BASE_DPI) - 1.5).abs() < 1e-9);
        assert!(size_of(0, 20.0, BASE_DPI).abs() < 1e-9, "no corner");
    }

    #[test]
    fn a_windows_box_becomes_a_ports_box_and_an_upside_down_one_has_no_size() {
        let up = RECT {
            left: 10,
            top: 20,
            right: 110,
            bottom: 50,
        };
        assert_eq!(rect(&up), r(10, 20, 100, 30));
        let down = RECT {
            left: 110,
            top: 50,
            right: 10,
            bottom: 20,
        };
        assert_eq!(rect(&down), r(110, 50, 0, 0));
    }

    #[test]
    fn parts_of_one_line_join_and_another_line_stays_apart() {
        let parts = [r(10, 100, 50, 20), r(60, 102, 40, 18), r(10, 130, 80, 20)];
        assert_eq!(line(&parts, 0), Some(r(10, 100, 90, 20)));
        assert_eq!(line(&parts, 2), Some(r(10, 130, 80, 20)));
        assert_eq!(line(&parts, 3), None);
    }

    #[test]
    fn the_strip_goes_under_the_last_line_centred_on_it() {
        let boxes = [r(100, 100, 400, 20), r(100, 120, 200, 20)];
        let at = want(&boxes, (160, 40), 8, AREA, &[]).expect("a place");
        assert_eq!(
            at,
            Point { x: 120, y: 148 },
            "centred on the last line, 8 px below"
        );
        assert_eq!(want(&[], (160, 40), 8, AREA, &[]), None);
    }

    #[test]
    fn near_the_bottom_or_over_the_keyboard_it_goes_above() {
        let low = [r(100, 1000, 200, 20)];
        let at = want(&low, (160, 40), 8, AREA, &[]).expect("a place");
        assert_eq!(at.y, 1000 - 8 - 40, "no room below");
        let keyboard = r(0, 700, 1920, 332);
        let mid = [r(100, 640, 200, 20)];
        let at = want(&mid, (160, 40), 8, AREA, &[keyboard]).expect("a place");
        assert_eq!(at.y, 640 - 8 - 40, "clear of the keyboard");
    }

    #[test]
    fn a_strip_within_the_slack_clear_of_the_keyboard_passes() {
        let (want_at, first) = (Point { x: 120, y: 148 }, r(100, 100, 400, 20));
        let good = judge(want_at, r(121, 147, 160, 40), 1, &[], first);
        assert_eq!(
            (good["ok"].as_bool(), good["off_px"].as_i64()),
            (Some(true), Some(1))
        );
        assert_eq!(good["above"], false);
        let far = judge(want_at, r(122, 148, 160, 40), 1, &[], first);
        assert_eq!(far["ok"], false, "2 px off");
        let bubble = r(1800, 900, 60, 60);
        let over = judge(
            want_at,
            r(120, 148, 160, 40),
            1,
            &[bubble, r(0, 170, 1920, 300)],
            first,
        );
        assert_eq!(
            (over["ok"].as_bool(), over["clear"].as_bool()),
            (Some(false), Some(false)),
            "any of our boxes"
        );
        let above = judge(Point { x: 120, y: 52 }, r(120, 52, 160, 40), 1, &[], first);
        assert_eq!(above["above"], true);
    }
}
