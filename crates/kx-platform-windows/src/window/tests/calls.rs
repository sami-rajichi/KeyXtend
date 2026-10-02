//! The guard's own OS calls, one at a time: the kept styles and the click region.

use kx_module_api::ports::Rect;
use kx_platform::ui::UiError;
use windows::Win32::Graphics::Gdi::{DeleteObject, HRGN, PtInRegion};

use super::super::guard::add_kept;
use super::super::win::region;
use super::Hidden;

/// Which of `points` lie in region `all`, which is then deleted.
fn inside(all: HRGN, points: &[(i32, i32)]) -> Vec<bool> {
    // SAFETY: plain queries of the test's own region, then one delete.
    unsafe {
        let hits = points
            .iter()
            .map(|&(x, y)| PtInRegion(all, x, y).as_bool())
            .collect();
        let _ = DeleteObject(all.into());
        hits
    }
}

#[test]
fn kept_styles_that_never_land_are_refused() {
    let window = Hidden::new("KeyXtend guard test: gone");
    let gone = window.0;
    drop(window);
    assert!(matches!(add_kept(gone), Err(UiError::Refused(_))));
}

#[test]
fn a_region_covers_each_rect_and_nothing_between() {
    let rects = [
        Rect {
            x: 0,
            y: 0,
            w: 10,
            h: 10,
        },
        Rect {
            x: 20,
            y: 5,
            w: 5,
            h: 5,
        },
    ];
    let all = region(&rects).unwrap();
    let points = [
        (0, 0),
        (9, 9),
        (20, 5),
        (24, 9),
        (10, 0),
        (15, 5),
        (25, 5),
        (20, 4),
    ];
    let hits = inside(all, &points);
    assert_eq!(hits, [true, true, true, true, false, false, false, false]);
}

#[test]
fn no_rect_covers_nothing_and_a_far_edge_past_i32_is_refused_not_overflowed() {
    let none = region(&[]).unwrap();
    assert_eq!(inside(none, &[(0, 0)]), [false]);
    let wide = region(&[Rect {
        x: 0,
        y: 0,
        w: 1_000_000,
        h: 4,
    }])
    .unwrap();
    assert_eq!(inside(wide, &[(999_999, 3), (1_000_000, 3)]), [true, false]);
    let past = Rect {
        x: i32::MAX - 1,
        y: 0,
        w: 10,
        h: 4,
    };
    assert!(matches!(region(&[past]), Err(UiError::Refused(_))));
}
