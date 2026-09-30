//! Where the face's windows go on the usable screen, built on the product's screens and windows
//! (`kx_platform_windows::{screen, window}`, ADR-0014); the placement moves with the keyboard's place maths.

use kx_platform::ui::UiError;
use kx_platform_windows::screen::{DEFAULT_DPI, dpi_of, physical, work_area};
use kx_platform_windows::window::own_bounds;
use windows::Win32::Foundation::RECT;

use crate::hold::Pt;
use crate::place::Place;
use crate::popspot::pop_spot;

/// Error when the window a bubble or panel belongs to is not shown.
const NOT_SHOWN: &str = "the keyboard window is not shown";

/// A product error as the face's text.
fn text(e: UiError) -> String {
    e.to_string()
}

/// Top-left of a box of `size` centred `gap` above the bottom of `work`; a box wider than `work` starts at its left edge.
pub fn bottom_centre(work: &RECT, size: (i32, i32), gap: i32) -> Pt {
    Pt {
        x: work.left + ((work.right - work.left - size.0) / 2).max(0),
        y: work.bottom - size.1 - gap,
    }
}

/// Where a `d`-wide bubble goes: `inset` inside the bottom-left or bottom-right corner of `work`, never off it.
pub fn corner_spot(work: &RECT, left: bool, d: i32, inset: i32) -> Pt {
    let x = if left {
        work.left + inset
    } else {
        work.right - d - inset
    };
    Pt {
        x: x.min(work.right - d).max(work.left),
        y: (work.bottom - d - inset).max(work.top),
    }
}

/// Where a window `w` by `h` logical pixels starts, in physical pixels: the bottom centre of the primary work area.
pub fn start_spot(w: f32, h: f32) -> Result<Pt, String> {
    // The primary monitor is the one holding (0, 0).
    let origin = RECT::default();
    let (work, dpi) = (work_area(&origin).map_err(text)?, dpi_of(&origin));
    Ok(bottom_centre(
        &work,
        (physical(w, dpi), physical(h, dpi)),
        0,
    ))
}

/// Where the minimise bubble goes on the screen of our window `title`: a bottom corner, `left` or right.
/// `[d, inset]` are its diameter and inset in logical pixels.
pub fn bubble_for(title: &str, left: bool, [d, inset]: [f32; 2]) -> Result<Pt, String> {
    let r = own_rect(title)?;
    let (work, dpi) = (work_area(&r).map_err(text)?, dpi_of(&r));
    Ok(corner_spot(
        &work,
        left,
        physical(d, dpi),
        physical(inset, dpi),
    ))
}

/// Where a `size` panel goes by our window `title` (see `pop_in`); the spot is in physical pixels.
pub fn pop_for(
    title: &str,
    plate_w: f32,
    size: [f32; 2],
    gaps: [f32; 2],
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
    plate_w: f32,
    size: [f32; 2],
    gaps: [f32; 2],
    rtl: bool,
) -> Pt {
    let k = dpi as f32 / DEFAULT_DPI as f32;
    let kb = Place {
        w: plate_w * k,
        ..place_of(r)
    };
    let (w, h) = (size[0] * k, size[1] * k);
    let (x, y) = pop_spot(kb, w, h, place_of(work), gaps.map(|g| g * k), rtl);
    Pt {
        x: x.round() as i32,
        y: y.round() as i32,
    }
}

/// `r` as a place.
fn place_of(r: &RECT) -> Place {
    Place {
        x: r.left as f32,
        y: r.top as f32,
        w: (r.right - r.left) as f32,
        h: (r.bottom - r.top) as f32,
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
        let work = |left, right, bottom| RECT {
            left,
            top: 0,
            right,
            bottom,
        };
        assert_eq!(
            bottom_centre(&work(0, 1920, 1040), (900, 70), 30),
            Pt { x: 510, y: 940 }
        );
        let wide = bottom_centre(&work(100, 500, 800), (600, 50), 10);
        assert_eq!(
            wide.x, 100,
            "a box wider than the screen starts at its left edge"
        );
    }

    #[test]
    fn the_bubble_sits_in_a_bottom_corner_of_the_work_area() {
        let work = rect(0, 0, 1920, 1020);
        assert_eq!(corner_spot(&work, false, 46, 12), Pt { x: 1862, y: 962 });
        assert_eq!(corner_spot(&work, true, 46, 12), Pt { x: 12, y: 962 });
        let second = rect(1920, 0, 3840, 1080);
        assert_eq!(corner_spot(&second, false, 46, 12), Pt { x: 3782, y: 1022 });
        let tiny = rect(0, 0, 20, 20);
        assert_eq!(
            corner_spot(&tiny, false, 46, 12),
            Pt { x: 0, y: 0 },
            "a screen smaller than the bubble"
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
    fn a_window_that_is_not_shown_has_no_bubble_or_panel_spot() {
        let title = "spike test: no such window";
        assert_eq!(bubble_for(title, true, [46.0, 12.0]), Err(NOT_SHOWN.into()));
        let panel = pop_for(title, 800.0, [480.0, 300.0], [10.0, 8.0], false);
        assert_eq!(panel, Err(NOT_SHOWN.into()));
    }
}
