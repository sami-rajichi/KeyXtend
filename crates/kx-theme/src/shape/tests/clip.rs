//! The clipboard panel's sizes: the mock-up's rows, tick box, thumbnail and pills, with big targets for one finger.

use super::pad::refused_in;
use super::*;

#[test]
fn the_clipboard_panel_takes_the_mock_up_sizes() {
    let c = load().expect("shape.toml loads").clip;
    assert_eq!(c.tick_px, [22.0, 6.0, 1.5, 11.0], ".ck");
    assert_eq!(c.thumb_px, [46.0, 34.0, 6.0, 17.0], ".th2");
    assert_eq!(
        (c.row_gap_px, c.meta_px),
        (9.0, [10.5, 1.0]),
        ".lrow, .tt .m"
    );
    assert_eq!((c.pin_px, c.kept_px), (13.0, 11.0), ".pin .i, .sm");
    assert_eq!(c.filters_pad_px, [8.0, 10.0, 4.0], ".filters");
    assert_eq!(c.empty_px, 30.0, ".locked-box .i");
    assert_eq!(c.tick_weight, 600, ".ck");
}

#[test]
fn the_clipboard_panels_targets_are_big_for_one_finger() {
    let c = load().expect("shape.toml loads").clip;
    let least = 24.0;
    assert!(c.filter_px[0] >= least, "a chip, WCAG 2.5.8");
    assert!(c.button_px[0] >= least, "a button, WCAG 2.5.8");
    assert!(
        c.expires_px[0] > 10.0,
        "above the mock-up's .exp, so it reads"
    );
}

#[test]
fn an_odd_clipboard_size_is_refused_by_name() {
    refused_in(&[
        (
            "clip",
            "tick_weight = 600",
            "tick_weight = 950",
            "clip.tick_weight",
        ),
        (
            "clip",
            "button_weight = 600",
            "button_weight = 50",
            "clip.button_weight",
        ),
        ("clip", "caret_px = 2.0", "caret_px = 0.0", "clip.caret_px"),
        (
            "clip",
            "underline_px = 2.0",
            "underline_px = nan",
            "clip.underline_px",
        ),
        ("clip", "\nkept_px", "\nhue = 1.0\nkept_px", "hue"),
    ]);
}
