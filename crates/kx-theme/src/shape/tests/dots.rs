//! The one set of dots the language key's Cycle design and the shortcuts layer's Next key draw.

use super::pad::refused_in;
use super::*;

#[test]
fn the_cycle_and_next_keys_share_the_mock_up_dots() {
    let d = load().expect("shape.toml loads").dots;
    assert_eq!((d.px, d.gap_px), (3.0, 2.0), ".dots i and its gap");
    assert!(
        d.off > 0.3,
        "raised from the mock-up's .3 to keep 3:1 (drawn)"
    );
    assert!(d.on_px > d.px, "the lit dot stands out by size");
}

#[test]
fn the_cycle_design_no_longer_sizes_its_own_dots() {
    let old = in_section("lang.cycle", "\nicon_share", "\ndot_px = 3.0\nicon_share");
    let e = parse(&old).expect_err("moved to [dots]");
    assert!(e.contains("unknown field `dot_px`"), "{e}");
}

#[test]
fn an_odd_dot_is_refused_by_name() {
    refused_in(&[
        ("dots", "off = 0.72", "off = nan", "shape: dots.off"),
        ("dots", "off = 0.72", "off = 1.5", "shape: dots.off"),
        ("dots", "on_px = 4.5", "on_px = 3.0", "shape: dots.on_px"),
        ("dots", "\npx = 3.0", "\npx = 0.0", "shape: dots.px"),
        (
            "dots",
            "gap_px = 2.0",
            "gap_px = -1.0",
            "shape: dots.gap_px",
        ),
    ]);
}
