//! Where a panel goes (mock-up `placePop`): above the keyboard, else below it, always inside the screen.

use crate::place::Place;

/// The top-left corner for a `w` by `h` panel next to keyboard `kb` on `screen`, all in one unit.
/// It sits `gap` above the keyboard, else below it, else at the top; it lines up with the keyboard's start edge
/// (the right edge when `rtl`) and stays at least `edge` inside the screen.
pub fn pop_spot(kb: Place, w: f32, h: f32, screen: Place, gaps: [f32; 2], rtl: bool) -> (f32, f32) {
    let [gap, edge] = gaps;
    let (top, left) = (screen.y + edge, screen.x + edge);
    let (bottom, right) = (screen.y + screen.h - edge, screen.x + screen.w - edge);
    let (above, below) = (kb.y - h - gap, kb.y + kb.h + gap);
    let y = if above >= top {
        above
    } else if below + h <= bottom {
        below
    } else {
        top
    };
    let x = if rtl { kb.x + kb.w - w } else { kb.x };
    (inside(x, left, right - w), inside(y, top, bottom - h))
}

/// `v` moved into `lo..=hi`, or `lo` when the range is empty; never panics, unlike `clamp`.
fn inside(v: f32, lo: f32, hi: f32) -> f32 {
    v.min(hi).max(lo)
}

#[cfg(test)]
mod tests {
    use super::*;

    const GAPS: [f32; 2] = [10.0, 8.0];
    const SCREEN: Place = Place {
        x: 0.0,
        y: 0.0,
        w: 1920.0,
        h: 1040.0,
    };

    fn kb(x: f32, y: f32) -> Place {
        Place {
            x,
            y,
            w: 800.0,
            h: 300.0,
        }
    }

    #[test]
    fn it_sits_above_the_keyboard_at_its_start_edge() {
        assert_eq!(
            pop_spot(kb(500.0, 700.0), 480.0, 300.0, SCREEN, GAPS, false),
            (500.0, 390.0)
        );
        let rtl = pop_spot(kb(500.0, 700.0), 480.0, 300.0, SCREEN, GAPS, true);
        assert_eq!(rtl, (820.0, 390.0), "right edges line up");
    }

    #[test]
    fn with_no_room_above_it_goes_below_else_to_the_top() {
        assert_eq!(
            pop_spot(kb(500.0, 100.0), 480.0, 300.0, SCREEN, GAPS, false).1,
            410.0
        );
        let tall = pop_spot(kb(500.0, 300.0), 480.0, 700.0, SCREEN, GAPS, false);
        assert_eq!(tall.1, 8.0, "neither fits");
    }

    #[test]
    fn it_stays_inside_the_screen_up_and_down() {
        let low = pop_spot(kb(500.0, 1100.0), 480.0, 300.0, SCREEN, GAPS, false);
        assert_eq!(low.1, 732.0, "1040 - 8 - 300");
    }

    #[test]
    fn a_panel_wider_than_the_screen_starts_at_its_left_edge() {
        let wide = pop_spot(kb(0.0, 700.0), 2000.0, 300.0, SCREEN, GAPS, true);
        assert_eq!(wide.0, 8.0);
    }

    #[test]
    fn a_second_screen_or_a_top_taskbar_moves_the_edges() {
        let second = Place {
            x: 1920.0,
            y: 40.0,
            w: 1920.0,
            h: 1000.0,
        };
        let rtl = pop_spot(kb(2000.0, 500.0), 480.0, 300.0, second, GAPS, true);
        assert_eq!(rtl, (2320.0, 190.0));
        let high = pop_spot(kb(2000.0, 200.0), 480.0, 300.0, second, GAPS, true);
        assert_eq!(high.1, 510.0, "below, as 200 - 310 is above the taskbar");
    }

    #[test]
    fn it_stays_inside_the_screen_sideways() {
        assert_eq!(
            pop_spot(kb(-100.0, 700.0), 480.0, 300.0, SCREEN, GAPS, false).0,
            8.0
        );
        let rtl = pop_spot(kb(1500.0, 700.0), 480.0, 300.0, SCREEN, GAPS, true);
        assert_eq!(rtl.0, 1432.0, "1920 - 480 - 8");
    }
}
