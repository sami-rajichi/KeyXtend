//! The screens: each one's work area without the taskbar, its DPI, and where the pointer is.
//! Box and DPI maths are pure and tested everywhere; the OS calls are in `win`.

use kx_module_api::ports::{self, BASE_DPI, Rect};

#[cfg(windows)]
mod win;

#[cfg(windows)]
pub use win::{WinScreens, cursor, dpi_of, window_dpi, work_area};

/// `USER_DEFAULT_SCREEN_DPI`: the ports' `BASE_DPI` (96 dots per inch) is 100 %.
pub const DEFAULT_DPI: u32 = BASE_DPI;
/// `MONITORINFOF_PRIMARY`: the screen that holds the desktop's origin.
const PRIMARY_FLAG: u32 = 1;

/// A Windows box (left, top, right, bottom) as a rect; a box turned inside out is empty.
#[must_use]
pub fn rect_of([left, top, right, bottom]: [i32; 4]) -> Rect {
    let span = |from: i32, to: i32| {
        let wide = (i64::from(to) - i64::from(from)).max(0);
        i32::try_from(wide).unwrap_or(i32::MAX)
    };
    Rect {
        x: left,
        y: top,
        w: span(left, right),
        h: span(top, bottom),
    }
}

/// The edges of `r` (left, top, right, bottom); a far edge stops at the end of the range.
#[must_use]
pub fn edges(r: Rect) -> [i32; 4] {
    [r.x, r.y, r.x.saturating_add(r.w), r.y.saturating_add(r.h)]
}

/// `dpi`, or the default when Windows gave none.
#[must_use]
pub fn dpi_or_default(dpi: u32) -> u32 {
    if dpi > 0 { dpi } else { DEFAULT_DPI }
}

/// `px` logical pixels at `dpi`, in physical pixels, by the ports' `physical`; NaN gives 0.
#[must_use]
pub fn physical(px: f32, dpi: u32) -> i32 {
    ports::physical(f64::from(px), dpi)
}

/// True when monitor flags `flags` mark the primary screen.
#[must_use]
pub fn is_primary(flags: u32) -> bool {
    flags & PRIMARY_FLAG != 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_windows_box_becomes_a_rect_and_back() {
        let r = rect_of([-1920, 0, 0, 1040]);
        assert_eq!(
            r,
            Rect {
                x: -1920,
                y: 0,
                w: 1920,
                h: 1040
            }
        );
        assert_eq!(edges(r), [-1920, 0, 0, 1040]);
    }

    #[test]
    fn a_box_turned_inside_out_is_empty() {
        let r = rect_of([100, 50, 40, 10]);
        assert_eq!((r.x, r.y, r.w, r.h), (100, 50, 0, 0));
    }

    #[test]
    fn far_edges_stay_in_range() {
        let r = rect_of([i32::MIN, i32::MIN, i32::MAX, i32::MAX]);
        assert_eq!((r.w, r.h), (i32::MAX, i32::MAX), "clamped, never wrapped");
        let far = Rect {
            x: i32::MAX,
            y: i32::MAX,
            w: 10,
            h: 10,
        };
        assert_eq!(edges(far), [i32::MAX; 4], "saturated, never wrapped");
    }

    #[test]
    fn logical_pixels_scale_by_the_ports_rule() {
        assert_eq!(physical(33.0, 144), ports::physical(33.0, 144));
    }

    #[test]
    fn a_missing_dpi_is_the_default() {
        assert_eq!(dpi_or_default(0), DEFAULT_DPI);
        assert_eq!(dpi_or_default(144), 144);
    }

    #[test]
    fn only_the_primary_flag_marks_the_primary_screen() {
        assert!(is_primary(PRIMARY_FLAG));
        assert!(!is_primary(0));
    }

    #[cfg(windows)]
    #[test]
    fn the_constants_match_windows() {
        use windows::Win32::UI::WindowsAndMessaging::{
            MONITORINFOF_PRIMARY, USER_DEFAULT_SCREEN_DPI,
        };
        assert_eq!(DEFAULT_DPI, USER_DEFAULT_SCREEN_DPI);
        assert_eq!(PRIMARY_FLAG, MONITORINFOF_PRIMARY);
    }
}
