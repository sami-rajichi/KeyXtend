//! The selection strip's keycaps (mock-up design 4, `.d-keys`): the plate's padding, each cap's padding, word and icon,
//! and the gap to the selected line, with an odd size refused by name.

#![allow(
    clippy::float_cmp,
    reason = "token values parse to exactly the numbers the mock-up writes"
)]

use super::pad::refused_in;
use super::*;

#[test]
fn the_keycaps_take_the_mock_up_sizes() {
    let c = load().expect("shape.toml loads").chips;
    assert_eq!(c.pad_px, 3.0, ".d-keys padding");
    assert_eq!(
        (c.cap_pad_px, c.icon_gap_px, c.font_px),
        (10.0, 5.0, 11.5),
        ".d-keys .c"
    );
    assert_eq!(c.icon_px, 13.0, ".d-keys .i");
    assert_eq!(c.gap_px, 8.0, "place(): 8 px under the last line");
}

#[test]
fn an_odd_keycap_size_is_refused_by_name() {
    refused_in(&[
        ("chips", "font_px = 11.5", "font_px = 0.0", "chips.font_px"),
        ("chips", "icon_px = 13.0", "icon_px = nan", "chips.icon_px"),
        ("chips", "pad_px = 3.0", "pad_px = -1.0", "chips.pad_px"),
        (
            "chips",
            "cap_pad_px = 10.0",
            "cap_pad_px = inf",
            "chips.cap_pad_px",
        ),
        (
            "chips",
            "icon_gap_px = 5.0",
            "icon_gap_px = -0.5",
            "chips.icon_gap_px",
        ),
        ("chips", "gap_px = 8.0", "gap_px = nan", "chips.gap_px"),
    ]);
}

/// The gap between caps and the legend's weight are the theme's own (`.d-keys .key` padding `var(--gp)/2`,
/// `font-weight: var(--fw)`), so a shared token for either is refused.
#[test]
fn the_caps_gap_and_weight_stay_the_themes() {
    for token in ["weight = 600", "key_gap_px = 4.0"] {
        let t = in_section("chips", "\ngap_px", &format!("\n{token}\ngap_px"));
        let e = parse(&t).expect_err(token);
        assert!(e.contains("unknown field"), "{token}: {e}");
    }
}
