//! G12 click points and matching: the grid over the text box, and counting the clicks target-window logged.
#![cfg(windows)]

use kx_target_window::log::Press;
use spike_core::hold::Pt;
use windows::Win32::Foundation::{HWND, POINT, RECT};

use crate::hookio::inside;
use crate::tlog::Mouse;
use crate::win;

/// A small grid count as `i32`.
fn small(n: u32) -> i32 {
    i32::try_from(n).unwrap_or(i32::MAX)
}

/// `cols` by `rows` points spread over `r`, `inset` px inside its edges, row by row.
pub fn grid(r: RECT, [cols, rows]: [u32; 2], inset: i32) -> Vec<POINT> {
    let at = |lo: i32, hi: i32, n: u32, i: u32| {
        let (lo, hi) = (lo + inset, hi - inset);
        match n {
            0 | 1 => i32::midpoint(lo, hi),
            _ => lo + (hi - lo) * small(i) / (small(n) - 1),
        }
    };
    let mut out = Vec::new();
    for j in 0..rows {
        for i in 0..cols {
            let (x, y) = (at(r.left, r.right, cols, i), at(r.top, r.bottom, rows, j));
            out.push(POINT { x, y });
        }
    }
    out
}

/// How many of `clicked`, in order, target-window logged as a left press and release within `slack` px.
pub fn logged(got: &[Mouse], clicked: &[POINT], slack: i32) -> usize {
    let near = |m: &Mouse, p: &POINT| (m.x - p.x).abs() <= slack && (m.y - p.y).abs() <= slack;
    let pair = |w: &[Mouse], p: &POINT| {
        w[0].press == Press::LeftDown
            && w[1].press == Press::LeftUp
            && near(&w[0], p)
            && near(&w[1], p)
    };
    let mut rest = got;
    let mut n = 0;
    for p in clicked {
        if let Some(i) = rest.windows(2).position(|w| pair(w, p)) {
            n += 1;
            rest = &rest[i + 2..];
        }
    }
    n
}

/// True when `ring` shows above `target` in `front_first` and its box holds `p`.
pub fn ring_over(front_first: &[HWND], ring: HWND, target: HWND, p: POINT) -> bool {
    let at = |w: HWND| front_first.iter().position(|&o| o == w);
    let above = matches!((at(ring), at(target)), (Some(r), Some(t)) if r < t);
    above && win::rect(ring).is_ok_and(|r| inside(&r, Pt { x: p.x, y: p.y }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn press(press: Press, x: i32, y: i32) -> Mouse {
        Mouse { press, us: 0, x, y }
    }

    fn click(x: i32, y: i32) -> [Mouse; 2] {
        [press(Press::LeftDown, x, y), press(Press::LeftUp, x, y)]
    }

    fn hwnd(v: isize) -> HWND {
        HWND(v as *mut _)
    }

    #[test]
    fn the_grid_spans_the_box_inside_its_inset_row_by_row() {
        let r = RECT {
            left: 100,
            top: 50,
            right: 500,
            bottom: 350,
        };
        let g = grid(r, [3, 2], 50);
        let xy: Vec<(i32, i32)> = g.iter().map(|p| (p.x, p.y)).collect();
        assert_eq!(
            xy,
            [
                (150, 100),
                (300, 100),
                (450, 100),
                (150, 300),
                (300, 300),
                (450, 300)
            ]
        );
        assert_eq!(
            grid(r, [1, 1], 0)
                .iter()
                .map(|p| (p.x, p.y))
                .collect::<Vec<_>>(),
            [(300, 200)]
        );
    }

    #[test]
    fn a_lost_click_counts_once_and_later_clicks_still_match() {
        let pts = [
            POINT { x: 10, y: 10 },
            POINT { x: 50, y: 10 },
            POINT { x: 90, y: 10 },
        ];
        let mut got: Vec<Mouse> = [click(10, 10), click(91, 10)].concat();
        assert_eq!(logged(&got, &pts, 1), 2, "the middle click is lost");
        got.insert(2, press(Press::RightDown, 50, 10));
        assert_eq!(logged(&got, &pts, 1), 2, "a right press is not a click");
        assert_eq!(
            logged(&click(12, 10), &pts[..1], 1),
            0,
            "too far from the point"
        );
    }

    #[test]
    fn a_press_without_its_release_does_not_count() {
        let got = [
            press(Press::LeftDown, 10, 10),
            press(Press::RightDown, 10, 10),
        ];
        assert_eq!(logged(&got, &[POINT { x: 10, y: 10 }], 1), 0);
    }

    #[test]
    fn a_ring_covers_only_while_it_shows_above_the_target() {
        let (ring, target, other) = (hwnd(1), hwnd(2), hwnd(3));
        let p = POINT { x: 5, y: 5 };
        assert!(
            !ring_over(&[target, ring], ring, target, p),
            "behind the target"
        );
        assert!(!ring_over(&[other, target], ring, target, p), "hidden");
    }
}
