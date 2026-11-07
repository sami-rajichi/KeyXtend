//! The clipboard panel's sizes: the mock-up's tick box, pin and pills, with big targets for one finger.

use super::pad::refused_in;
use super::*;

#[test]
fn the_clipboard_panel_takes_the_mock_up_sizes() {
    let c = load().expect("shape.toml loads").clip;
    assert_eq!(c.tick_px, [22.0, 6.0, 1.5, 11.0], ".ck");
    assert_eq!((c.pin_px, c.kept_px), (13.0, 11.0), ".pin .i, .sm");
    assert_eq!(
        c.filters_pad_px,
        [6.0, 10.0, 2.0],
        ".filters, trimmed so the panel fits above the keyboard"
    );
    assert_eq!(c.tick_weight, 600, ".ck");
}

#[test]
fn the_clipboard_panels_targets_are_big_for_one_finger() {
    let s = load().expect("shape.toml loads");
    let least = 24.0;
    assert!(s.panel.filter_px[0] >= least, "a chip, WCAG 2.5.8");
    assert!(s.clip.button_px[0] >= least, "a button, WCAG 2.5.8");
    assert!(
        s.clip.expires_px[0] > 10.0,
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
        ("clip", "\nkept_px", "\nhue = 1.0\nkept_px", "hue"),
    ]);
}

#[test]
fn the_clipboard_panel_is_trimmed_to_fit_above_the_keyboard() {
    let s = load().expect("shape.toml loads");
    assert_eq!(s.panel.row_pad_px, 2.0, "rows keep their two title lines");
    assert_eq!(s.clip.room_px[0], 4.0, "the storage line's padding");
}

#[test]
fn a_panel_that_shrinks_stops_at_its_least_share() {
    refused_in(&[
        ("panel", "min_fit = 0.8", "min_fit = 1.5", "panel.min_fit"),
        ("panel", "min_fit = 0.8", "min_fit = 0.0", "panel.min_fit"),
    ]);
}
