//! Tests of `aim`.

use super::*;

fn bx(left: i32, top: i32, right: i32, bottom: i32) -> RECT {
    RECT {
        left,
        top,
        right,
        bottom,
    }
}

fn pt(x: i32, y: i32) -> POINT {
    POINT { x, y }
}

/// A 1920 x 1040 work area with the keyboard across its lower part, its top at 700 and the pad at x 1500.
const WORK: RECT = RECT {
    left: 0,
    top: 0,
    right: 1920,
    bottom: 1040,
};
const KB: RECT = RECT {
    left: 600,
    top: 700,
    right: 1600,
    bottom: 1000,
};
const PAD_X: i32 = 1500;
const REACH: Reach = Reach {
    under_px: 160,
    lead_px: 120,
};

#[test]
fn a_window_that_reaches_under_the_keyboard_stays_put() {
    let maximised = WORK;
    assert_eq!(laid(maximised, KB, PAD_X, REACH, WORK), None);
    let just = bx(1000, 580, 1600, 860);
    assert_eq!(
        laid(just, KB, PAD_X, REACH, WORK),
        None,
        "exactly far enough"
    );
}

#[test]
fn a_window_short_of_the_keyboard_or_beside_the_pad_is_laid_across_the_top() {
    let want = Some(bx(0, 0, 1920, 860));
    assert_eq!(
        laid(bx(0, 0, 900, 500), KB, PAD_X, REACH, WORK),
        want,
        "too short"
    );
    assert_eq!(
        laid(bx(0, 0, 1400, 1040), KB, PAD_X, REACH, WORK),
        want,
        "beside the pad"
    );
    assert_eq!(
        laid(bx(1000, 600, 1900, 1000), KB, PAD_X, REACH, WORK),
        want,
        "no room above"
    );
    let low = bx(600, 980, 1600, 1040);
    let capped = laid(bx(0, 0, 10, 10), low, PAD_X, REACH, WORK);
    assert_eq!(capped, Some(bx(0, 0, 1920, 1040)), "kept in the work area");
}

#[test]
fn the_glide_starts_above_the_keyboard_at_the_pads_x_only_on_the_window() {
    assert_eq!(start(WORK, KB, PAD_X, 120), Some(pt(1500, 580)));
    assert_eq!(
        start(bx(0, 0, 1400, 900), KB, PAD_X, 120),
        None,
        "beside it"
    );
    assert_eq!(
        start(bx(0, 600, 1920, 900), KB, PAD_X, 120),
        None,
        "above it"
    );
}

#[test]
fn the_glide_goes_from_its_start_to_the_pad_in_even_steps() {
    let points = glide(pt(1500, 580), pt(1500, 880), 3);
    assert_eq!(
        points,
        [pt(1500, 580), pt(1500, 680), pt(1500, 780), pt(1500, 880)]
    );
}

#[test]
fn the_kept_point_is_the_glides_last_one_outside_the_keyboard() {
    let points = glide(pt(1500, 580), pt(1500, 880), 3);
    assert_eq!(last_outside(&points, KB), Some(pt(1500, 680)));
    let back_out = [pt(1500, 580), pt(1500, 800), pt(1700, 800), pt(1500, 800)];
    assert_eq!(
        last_outside(&back_out, KB),
        Some(pt(1700, 800)),
        "the latest"
    );
    assert_eq!(last_outside(&[pt(1500, 800)], KB), None, "never outside");
    assert_eq!(
        last_outside(&[pt(1500, 700)], KB),
        None,
        "the top edge is on the keyboard"
    );
}

#[test]
fn the_picture_band_is_the_window_above_the_keyboard_inset_on_every_side() {
    let laid = bx(0, 0, 1920, 860);
    assert_eq!(band(laid, KB, 16), Some(bx(16, 16, 1904, 684)));
    let short = bx(100, 100, 900, 500);
    assert_eq!(
        band(short, KB, 16),
        Some(bx(116, 116, 884, 484)),
        "ends above it"
    );
    assert_eq!(
        band(bx(0, 690, 1920, 1000), KB, 16),
        None,
        "all under its top"
    );
}
