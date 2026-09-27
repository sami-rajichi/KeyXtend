//! Pure box maths for UI Automation results: line boxes, points on a line, and the pill anchor.

use windows::Win32::Foundation::RECT;

use crate::hold::Pt;

/// Numbers per box from `GetBoundingRectangles`: left, top, width, height.
const BOX: usize = 4;

/// The size of the selection pill and its gap from the text, in physical pixels.
#[derive(Clone, Copy, Debug)]
pub struct Pill {
    /// Width.
    pub w: i32,
    /// Height.
    pub h: i32,
    /// Gap between the text and the pill.
    pub gap: i32,
}

/// The pill's top-left: under the last line box, else above the first, centred on that box; always on `screen`.
pub fn anchor(boxes: &[RECT], screen: &RECT, pill: Pill) -> Option<Pt> {
    let (first, last) = (boxes.first()?, boxes.last()?);
    let below = last.bottom.saturating_add(pill.gap);
    let (next_to, y) = if below.saturating_add(pill.h) <= screen.bottom {
        (last, below)
    } else {
        (
            first,
            first.top.saturating_sub(pill.gap).saturating_sub(pill.h),
        )
    };
    let x = centre(next_to).x.saturating_sub(pill.w / 2);
    // Saturating maths: boxes come from other apps. min then max, not clamp, so a tiny screen cannot panic.
    Some(Pt {
        x: x.min(screen.right.saturating_sub(pill.w)).max(screen.left),
        y: y.min(screen.bottom.saturating_sub(pill.h)).max(screen.top),
    })
}

/// Boxes from `left, top, width, height` groups; empty, non-finite and partial groups are dropped.
pub fn rects_of(v: &[f64]) -> Vec<RECT> {
    v.as_chunks::<BOX>()
        .0
        .iter()
        .filter(|b| b.iter().all(|n| n.is_finite()) && b[2] > 0.0 && b[3] > 0.0)
        .map(|b| RECT {
            left: b[0].round() as i32,
            top: b[1].round() as i32,
            right: (b[0] + b[2]).round() as i32,
            bottom: (b[1] + b[3]).round() as i32,
        })
        .collect()
}

/// The middle of `a` and `b`, which always fits.
fn mid(a: i32, b: i32) -> i32 {
    ((i64::from(a) + i64::from(b)) / 2) as i32
}

/// The centre of `r`.
pub fn centre(r: &RECT) -> Pt {
    Pt {
        x: mid(r.left, r.right),
        y: mid(r.top, r.bottom),
    }
}

/// Two points on text line `line`: `inset` px in from its left edge (never past its middle), and its middle.
pub fn line_points(line: &RECT, inset: i32) -> (Pt, Pt) {
    let mid = centre(line);
    let start = Pt {
        x: line.left.saturating_add(inset).min(mid.x),
        y: mid.y,
    };
    (start, mid)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r(left: i32, top: i32, right: i32, bottom: i32) -> RECT {
        RECT {
            left,
            top,
            right,
            bottom,
        }
    }

    #[test]
    fn line_boxes_come_from_groups_of_four_and_skip_empty_ones() {
        let v = [10.0, 20.0, 100.4, 18.6, 5.0, 5.0, 0.0, 10.0, 1.0, 2.0];
        assert_eq!(rects_of(&v), [r(10, 20, 110, 39)]);
        assert!(rects_of(&[]).is_empty());
    }

    const SCREEN: RECT = RECT {
        left: 0,
        top: 0,
        right: 1920,
        bottom: 1080,
    };
    const PILL: Pill = Pill {
        w: 80,
        h: 30,
        gap: 8,
    };

    #[test]
    fn pill_sits_under_the_last_line_centred_on_it() {
        let lines = [r(100, 100, 500, 120), r(100, 120, 300, 140)];
        assert_eq!(anchor(&lines, &SCREEN, PILL), Some(Pt { x: 160, y: 148 }));
    }

    #[test]
    fn pill_is_clamped_onto_the_screen_when_the_box_leaves_it() {
        let right = anchor(&[r(1900, 100, 2000, 120)], &SCREEN, PILL);
        assert_eq!(right, Some(Pt { x: 1840, y: 128 }));
        let left = anchor(&[r(-50, -20, 10, 0)], &SCREEN, PILL);
        assert_eq!(left, Some(Pt { x: 0, y: 8 }));
    }

    #[test]
    fn pill_goes_above_the_first_line_when_below_has_no_room() {
        let lines = [r(100, 1030, 500, 1050), r(100, 1050, 200, 1070)];
        assert_eq!(anchor(&lines, &SCREEN, PILL), Some(Pt { x: 260, y: 992 }));
    }

    #[test]
    fn boxes_that_are_not_finite_are_dropped() {
        assert!(rects_of(&[f64::INFINITY, 0.0, 10.0, 10.0]).is_empty());
        assert!(rects_of(&[0.0, f64::NAN, 10.0, 10.0]).is_empty());
    }

    #[test]
    fn huge_boxes_from_another_app_never_overflow() {
        let huge = r(i32::MAX - 1, i32::MAX - 1, i32::MAX, i32::MAX);
        assert_eq!(
            centre(&huge),
            Pt {
                x: i32::MAX - 1,
                y: i32::MAX - 1
            }
        );
        assert_eq!(
            anchor(&[huge], &SCREEN, PILL),
            Some(Pt { x: 1840, y: 1050 })
        );
        let low = r(i32::MIN, i32::MIN, i32::MIN + 1, i32::MIN + 1);
        assert_eq!(anchor(&[low], &SCREEN, PILL), Some(Pt { x: 0, y: 0 }));
        let _ = line_points(&huge, i32::MAX);
        assert_eq!(rects_of(&[1e12, 0.0, 1e12, 10.0])[0].right, i32::MAX);
    }

    #[test]
    fn no_boxes_means_no_pill() {
        assert_eq!(anchor(&[], &SCREEN, PILL), None);
    }

    #[test]
    fn line_points_start_inside_the_line_and_never_pass_its_middle() {
        let (start, mid) = line_points(&r(100, 50, 300, 70), 3);
        assert_eq!((start, mid), (Pt { x: 103, y: 60 }, Pt { x: 200, y: 60 }));
        let (start, mid) = line_points(&r(100, 50, 104, 70), 3);
        assert_eq!(start, mid);
    }
}
