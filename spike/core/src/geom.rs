//! Pure box maths for kx-gates: a box's centre and points on a text line. The pill's anchor moved to
//! `kx_module_api::ports::geom` (P10); line boxes from UI Automation are `kx_platform_windows::uia`'s (P7).

use windows::Win32::Foundation::RECT;

use crate::pt::Pt;

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
    fn huge_boxes_from_another_app_never_overflow() {
        let huge = r(i32::MAX - 1, i32::MAX - 1, i32::MAX, i32::MAX);
        assert_eq!(
            centre(&huge),
            Pt {
                x: i32::MAX - 1,
                y: i32::MAX - 1
            }
        );
        let low = r(i32::MIN, i32::MIN, i32::MIN + 1, i32::MIN + 1);
        let _ = (line_points(&huge, i32::MAX), line_points(&low, i32::MIN));
    }

    #[test]
    fn line_points_start_inside_the_line_and_never_pass_its_middle() {
        let (start, mid) = line_points(&r(100, 50, 300, 70), 3);
        assert_eq!((start, mid), (Pt { x: 103, y: 60 }, Pt { x: 200, y: 60 }));
        let (start, mid) = line_points(&r(100, 50, 104, 70), 3);
        assert_eq!(start, mid);
    }
}
