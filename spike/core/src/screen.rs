//! Where the spike's later windows go: the caption (P11), the test panel (P13) and a snip's pointer (P12).
//! Built on the product's screens and windows (`kx_platform_windows`, ADR-0014) and kx-mod-keyboard's place maths.

use kx_mod_keyboard::Area;
use kx_mod_keyboard::place;
use kx_module_api::ports::{Point, Rect};
use kx_platform::ui::UiError;
use kx_platform_windows::screen::{DEFAULT_DPI, dpi_of, rect_of, work_area};
use kx_platform_windows::window::own_bounds;
use windows::Win32::Foundation::RECT;

use crate::popspot::pop_spot;
use crate::pt::Pt;

/// Error when the window a panel belongs to is not shown.
const NOT_SHOWN: &str = "the keyboard window is not shown";

/// A product error as the face's text.
fn text(e: UiError) -> String {
    e.to_string()
}

/// A Windows box as the ports' rect.
fn rect(r: &RECT) -> Rect {
    rect_of([r.left, r.top, r.right, r.bottom])
}

/// A ports point as the face's point.
fn pt(p: Point) -> Pt {
    Pt { x: p.x, y: p.y }
}

/// Top-left of a box of `size` centred `gap` above the bottom of `work`, inside it (the product's `bottom_centre`).
pub fn bottom_centre(work: &RECT, size: (i32, i32), gap: i32) -> Pt {
    let r = rect(work);
    let raised = Rect {
        h: r.h.saturating_sub(gap).max(0),
        ..r
    };
    pt(place::bottom_centre(raised, size))
}

/// Where a `size` panel goes by our window `title` (see `pop_in`); the spot is in physical pixels.
pub fn pop_for(
    title: &str,
    plate_w: f64,
    size: [f64; 2],
    gaps: [f64; 2],
    rtl: bool,
) -> Result<Pt, String> {
    let r = own_rect(title)?;
    let work = work_area(&r).map_err(text)?;
    Ok(pop_in(&r, &work, dpi_of(&r), plate_w, size, gaps, rtl))
}

/// Where a `size` panel goes by window `r` on `work` at `dpi` (see `pop_spot`); sizes and `gaps` are logical pixels.
/// It lines up with the plate, `plate_w` wide from the window's corner, and clears the whole window, strip too.
fn pop_in(
    r: &RECT,
    work: &RECT,
    dpi: u32,
    plate_w: f64,
    size: [f64; 2],
    gaps: [f64; 2],
    rtl: bool,
) -> Pt {
    let k = f64::from(dpi) / f64::from(DEFAULT_DPI);
    let kb = Area {
        w: plate_w * k,
        ..area_of(r)
    };
    let (w, h) = (size[0] * k, size[1] * k);
    let (x, y) = pop_spot(kb, w, h, area_of(work), gaps.map(|g| g * k), rtl);
    Pt {
        x: x.round() as i32,
        y: y.round() as i32,
    }
}

/// `r` as an area.
fn area_of(r: &RECT) -> Area {
    Area {
        x: f64::from(r.left),
        y: f64::from(r.top),
        w: f64::from(r.right - r.left),
        h: f64::from(r.bottom - r.top),
    }
}

/// The box of our visible window `title`, in physical pixels.
fn own_rect(title: &str) -> Result<RECT, String> {
    own_bounds(title).map_err(|e| match e {
        UiError::NoWindow => NOT_SHOWN.to_string(),
        other => text(other),
    })
}

/// Where the mouse pointer is, in physical pixels.
pub fn cursor() -> Result<Pt, String> {
    let p = kx_platform_windows::screen::cursor().map_err(text)?;
    Ok(Pt { x: p.x, y: p.y })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(left: i32, top: i32, right: i32, bottom: i32) -> RECT {
        RECT {
            left,
            top,
            right,
            bottom,
        }
    }

    #[test]
    fn a_box_sits_centred_above_the_bottom_of_the_work_area() {
        assert_eq!(
            bottom_centre(&rect(0, 0, 1920, 1040), (900, 70), 30),
            Pt { x: 510, y: 940 }
        );
        let wide = bottom_centre(&rect(100, 0, 500, 800), (600, 50), 10);
        assert_eq!(
            wide.x, 100,
            "a box wider than the screen starts at its left edge"
        );
    }

    #[test]
    fn a_panel_sits_above_the_plate_in_physical_pixels_at_the_screen_scale() {
        let (kb, work) = (rect(300, 900, 1500, 1400), rect(0, 0, 2880, 1560));
        let rtl = pop_in(&kb, &work, 144, 800.0, [480.0, 300.0], [10.0, 8.0], true);
        assert_eq!(
            rtl,
            Pt { x: 780, y: 435 },
            "300 + 1200 - 720, 900 - 450 - 15"
        );
        let ltr = pop_in(&kb, &work, 144, 800.0, [480.0, 300.0], [10.0, 8.0], false);
        assert_eq!(ltr.x, 300);
    }

    #[test]
    fn a_panel_below_clears_the_whole_keyboard_window() {
        let (kb, work) = (rect(300, 100, 1500, 600), rect(0, 0, 2880, 1560));
        let below = pop_in(&kb, &work, 144, 800.0, [480.0, 300.0], [10.0, 8.0], true);
        assert_eq!(below.y, 615, "600 + 15, under the strip too");
    }

    #[test]
    fn a_window_that_is_not_shown_has_no_panel_spot() {
        let title = "spike test: no such window";
        let panel = pop_for(title, 800.0, [480.0, 300.0], [10.0, 8.0], false);
        assert_eq!(panel, Err(NOT_SHOWN.into()));
    }
}
