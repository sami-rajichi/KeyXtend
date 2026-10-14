//! G12 click points and matching: the grid over the text box, each press's length, the overlay window over a point,
//! and counting the clicks target-window logged.
#![cfg(windows)]

use kx_mod_mouse::MouseSettings;
use kx_target_window::log::Press;
use spike_core::hold::Pt;
use windows::Win32::Foundation::{HWND, POINT, RECT};

use crate::p4cfg::G12;
use crate::simin::inside;
use crate::tlog::Mouse;

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

/// True when `ring` shows above `target` in `front_first` and its box `ring_box` holds `p`.
pub fn ring_over(front_first: &[HWND], ring: HWND, ring_box: RECT, target: HWND, p: POINT) -> bool {
    let at = |w: HWND| front_first.iter().position(|&o| o == w);
    let above = matches!((at(ring), at(target)), (Some(r), Some(t)) if r < t);
    above && inside(&ring_box, Pt { x: p.x, y: p.y })
}

/// The pointer overlay over `p`: of the test build's shown windows `ours`, each with its box, the first that is not
/// `keyboard`, shows above `target` in `front_first` and holds `p`. Found by place, as its title is translated.
pub fn overlay_at(
    front_first: &[HWND],
    ours: &[(HWND, RECT)],
    keyboard: HWND,
    target: HWND,
    p: POINT,
) -> Option<HWND> {
    let over = |&&(w, r): &&(HWND, RECT)| w != keyboard && ring_over(front_first, w, r, target, p);
    ours.iter().find(over).map(|&(w, _)| w)
}

/// Each G12 press's timing, in ms.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pace {
    /// When the ring shows after the press.
    pub ring_after_ms: u64,
    /// When the press is let go.
    pub press_ms: u64,
}

/// Each press under the test build's `mouse` settings: `ring_after_ms` plus `[g12] ring_ms`. Refused unless
/// `spare_ms` more stays within `hold_ms`, so it stays a click, and the press stays below the guard's `alive_ms`, as
/// the test build's hook holds the press back until its release.
pub fn pace(mouse: &MouseSettings, g: &G12, alive_ms: u64) -> Result<Pace, String> {
    let ring_after_ms = u64::from(mouse.ring_after_ms);
    let (press_ms, hold_ms) = (
        ring_after_ms.saturating_add(g.ring_ms),
        u64::from(mouse.hold_ms),
    );
    if press_ms.saturating_add(g.spare_ms) > hold_ms {
        return Err(format!(
            "a G12 press, ring_after_ms plus g12.ring_ms ({press_ms} ms), plus g12.spare_ms ({} ms) must stay within \
             the test build's hold_ms ({hold_ms} ms), so it stays a click",
            g.spare_ms
        ));
    }
    if press_ms >= alive_ms {
        return Err(format!(
            "a G12 press ({press_ms} ms) must stay below [mouse_guard] alive_ms ({alive_ms} ms): the test build's \
             hook holds it back, so the guard sees nothing until its release"
        ));
    }
    Ok(Pace {
        ring_after_ms,
        press_ms,
    })
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

    /// A box holding `P`, and one away from it.
    const HOLDS: RECT = RECT {
        left: 0,
        top: 0,
        right: 100,
        bottom: 100,
    };
    const AWAY: RECT = RECT {
        left: 200,
        top: 200,
        right: 300,
        bottom: 300,
    };
    const P: POINT = POINT { x: 50, y: 50 };

    #[test]
    fn a_ring_covers_only_while_it_shows_above_the_target_over_the_point() {
        let (ring, target, other) = (hwnd(1), hwnd(2), hwnd(3));
        assert!(ring_over(&[ring, target], ring, HOLDS, target, P));
        assert!(
            !ring_over(&[target, ring], ring, HOLDS, target, P),
            "behind the target"
        );
        assert!(
            !ring_over(&[other, target], ring, HOLDS, target, P),
            "hidden"
        );
        assert!(
            !ring_over(&[ring, target], ring, AWAY, target, P),
            "elsewhere"
        );
    }

    #[test]
    fn the_overlay_skips_the_keyboard_windows_below_target_window_and_boxes_without_the_point() {
        let (kb, over, low, target, other) = (hwnd(1), hwnd(2), hwnd(3), hwnd(4), hwnd(5));
        let front = [kb, over, other, target, low];
        let at = |ours: &[(HWND, RECT)]| overlay_at(&front, ours, kb, target, P);
        assert_eq!(at(&[(kb, HOLDS), (over, HOLDS)]), Some(over));
        assert_eq!(
            at(&[(kb, HOLDS)]),
            None,
            "the keyboard is never the overlay"
        );
        assert_eq!(at(&[(low, HOLDS)]), None, "below target-window");
        assert_eq!(at(&[(over, AWAY)]), None, "its box lacks the point");
        let hidden = overlay_at(&[kb, target], &[(over, HOLDS)], kb, target, P);
        assert_eq!(hidden, None, "not shown");
    }

    fn cfg() -> crate::config::GatesConfig {
        crate::config::load().expect("kx-gates.toml loads")
    }

    fn defaults() -> MouseSettings {
        MouseSettings::from_defaults().expect("the mouse defaults")
    }

    #[test]
    fn a_press_shows_the_ring_and_one_that_would_reach_the_hold_time_is_refused() {
        let (g, alive) = (cfg().g12, cfg().mouse_guard.alive_ms);
        let m = defaults();
        let p = pace(&m, &g, alive).expect("the shipped press fits the default hold");
        assert_eq!(p.press_ms, u64::from(m.ring_after_ms) + g.ring_ms);
        let hold = u32::try_from(p.press_ms + g.spare_ms).expect("fits");
        let at_hold = MouseSettings {
            hold_ms: hold,
            ..m.clone()
        };
        assert!(
            pace(&at_hold, &g, alive).is_ok(),
            "its spare ends at the hold time"
        );
        let short = MouseSettings {
            hold_ms: hold - 1,
            ..m
        };
        let e = pace(&short, &g, alive).expect_err("the spare would reach the hold time");
        assert!(e.contains("hold_ms"), "{e}");
    }

    #[test]
    fn a_press_the_guard_would_take_for_a_lost_hook_is_refused() {
        let g = cfg().g12;
        let m = defaults();
        let press = pace(&m, &g, u64::MAX).expect("fits").press_ms;
        let e = pace(&m, &g, press).expect_err("as long as the alive watch");
        assert!(e.contains("alive_ms"), "{e}");
        assert!(pace(&m, &g, press + 1).is_ok());
    }
}
