//! Quick-fill's sizes: a width that holds a login's three buttons, its kind chips, and big targets; the list sizes it
//! shares with the clipboard live once in `[panel]`.

use super::pad::refused_in;
use super::*;

#[test]
fn the_clipboard_and_quick_fill_share_their_list_sizes_in_the_panel() {
    let p = load().expect("shape.toml loads").panel;
    assert_eq!(p.tile_px, [46.0, 34.0, 6.0, 17.0], ".th2");
    assert_eq!(
        (p.row_gap_px, p.row_pad_px, p.hint_px),
        (9.0, 2.0, [10.5, 1.0]),
        ".lrow, .tt .m"
    );
    assert_eq!(p.empty_px, 30.0, ".locked-box .i");
    assert_eq!(
        p.line_px,
        [15.0, 2.0, 2.0, 2.0],
        "the line that takes the letters"
    );
    assert_eq!((p.filter_gap_px, p.question_gap_px), (4.0, 12.0));
    assert_eq!(p.question_lines, 2, "a question on one row");
    assert_eq!(
        p.wheel_notch, 120.0,
        "one notch, in Qt's eighths of a degree"
    );
    assert_eq!(p.wheel_rest_s, 1.0, "a slow wheel's steps still add up");
}

#[test]
fn quick_fill_takes_its_two_lines_and_its_kind_chips() {
    let f = load().expect("shape.toml loads").fill;
    assert_eq!(
        f.top_px,
        [6.0, 6.0, 2.0],
        "the page's two lines over its rows"
    );
    assert_eq!(f.kind_rows, 2, "New's six kinds in two rows of three");
}

#[test]
fn quick_fill_is_wider_than_the_mock_up_and_its_targets_are_big() {
    let s = load().expect("shape.toml loads");
    let f = &s.fill;
    assert!(
        f.width_px > 420.0,
        "wider than .fill, for three buttons a row"
    );
    let least = 24.0;
    assert!(s.panel.filter_px[0] >= least, "a chip, WCAG 2.5.8");
    assert!(
        f.kind_px[2] >= least,
        "a kind chip's icon reads at a glance"
    );
    assert!(
        s.clip.button_px[0] >= s.panel.filter_px[0],
        "the page's buttons sit in the filters' band"
    );
}

#[test]
fn an_odd_quick_fill_or_list_size_is_refused_by_name() {
    refused_in(&[
        (
            "fill",
            "width_px = 600.0",
            "width_px = 0.0",
            "fill.width_px",
        ),
        ("fill", "kind_rows = 2", "kind_rows = 0", "fill.kind_rows"),
        (
            "panel",
            "question_lines = 2",
            "question_lines = 0",
            "panel.question_lines",
        ),
        (
            "panel",
            "line_px = [15.0, 2.0",
            "line_px = [15.0, 0.0",
            "panel.line_px",
        ),
        (
            "panel",
            "line_px = [15.0, 2.0, 2.0, 2.0]",
            "line_px = [15.0, 2.0, 2.0, nan]",
            "panel.line_px",
        ),
        (
            "fill",
            "own_share = 0.4",
            "own_share = 1.5",
            "fill.own_share",
        ),
        ("fill", "\nwidth_px", "\nhue = 1.0\nwidth_px", "hue"),
    ]);
}

#[test]
fn an_odd_wheel_size_is_refused_by_name() {
    refused_in(&[
        (
            "panel",
            "wheel_notch = 120.0",
            "wheel_notch = 0.0",
            "panel.wheel_notch",
        ),
        (
            "panel",
            "wheel_rest_s = 1.0",
            "wheel_rest_s = 0.0",
            "panel.wheel_rest_s",
        ),
    ]);
}
