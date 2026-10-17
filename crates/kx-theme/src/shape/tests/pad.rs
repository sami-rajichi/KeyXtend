//! The scroll pad's sizes: where its arrows sit, Stop's speed dots, the badge's separator and the target's outline.

use super::*;

#[test]
fn the_arrows_sit_across_the_ring_and_stop_holds_the_speed_dots() {
    let d = load().expect("shape.toml loads").dpad;
    assert_eq!(
        d.reach, 0.5,
        "the middle of the ring, as the iPod click wheel's labels"
    );
    assert_eq!(
        (d.dot_px, d.dot_gap_px),
        (3.0, 2.0),
        "mock-up .lv i and its gap"
    );
    assert!(
        d.dot_off > 0.25 && d.dot_off < 1.0,
        "a little above the mock-up's .25"
    );
}

#[test]
fn the_outline_and_the_badges_separator_take_the_mock_up_sizes() {
    let s = load().expect("shape.toml loads");
    assert_eq!(s.outline.px, 2.0, ".scroll-target outline 2px");
    assert_eq!(s.badge.sep_share, 0.4, "the badge's · at opacity .4");
    assert!(s.badge.sep_px > 0.0 && s.badge.sep_px < s.badge.icon_px);
}

#[test]
fn the_old_icon_padding_is_gone() {
    let old = in_section("dpad", "\nreach", "\nicon_pad = 0.22\nreach");
    let e = parse(&old).expect_err("replaced by reach");
    assert!(e.contains("unknown field `icon_pad`"), "{e}");
}

/// A section, a token line as shipped, a bad value for it, and the name the error must give.
type BadIn = (&'static str, &'static str, &'static str, &'static str);

/// Each `BadIn` token is refused with its name.
fn refused_in(cases: &[BadIn]) {
    for (section, from, to, says) in cases {
        let e = parse(&in_section(section, from, to)).expect_err(to);
        assert!(e.contains(says), "{to}: {e}");
    }
}

#[test]
fn an_odd_pad_badge_or_outline_size_is_refused() {
    refused_in(&[
        ("dpad", "reach = 0.5", "reach = 1.5", "dpad.reach"),
        ("dpad", "dot_off = 0.3", "dot_off = -0.1", "dpad.dot_off"),
        ("dpad", "dot_px = 3.0", "dot_px = 0.0", "dpad.dot_px"),
        (
            "dpad",
            "dot_gap_px = 2.0",
            "dot_gap_px = -1.0",
            "dpad.dot_gap_px",
        ),
        (
            "badge",
            "sep_share = 0.4",
            "sep_share = 1.5",
            "badge.sep_share",
        ),
        ("badge", "sep_px = 2.0", "sep_px = 0.0", "badge.sep_px"),
        ("outline", "px = 2.0", "px = nan", "shape: outline.px"),
    ]);
}

#[test]
fn a_pad_gap_below_0_is_refused() {
    refused_in(&[
        (
            "dpad",
            "gap_px = 2.0",
            "gap_px = -1.0",
            "shape: dpad.gap_px",
        ),
        ("dpad", "gap_px = 2.0", "gap_px = nan", "shape: dpad.gap_px"),
    ]);
    let touching = in_section("dpad", "gap_px = 2.0", "gap_px = 0.0");
    assert!(parse(&touching).is_ok(), "arrows may touch");
}

#[test]
fn a_pad_icon_of_0_or_less_is_refused() {
    refused_in(&[
        (
            "dpad",
            "icon_px = 15.0",
            "icon_px = 0.0",
            "shape: dpad.icon_px",
        ),
        (
            "dpad",
            "stop_icon_px = 10.0",
            "stop_icon_px = -1.0",
            "shape: dpad.stop_icon_px",
        ),
        (
            "dpad",
            "stop_icon_px = 10.0",
            "stop_icon_px = inf",
            "shape: dpad.stop_icon_px",
        ),
    ]);
}

#[test]
fn a_hub_inset_of_half_the_pad_or_more_is_refused() {
    // DPad.qml draws the hub r(1 - 2 hub) wide, so half the pad leaves none.
    for hub in ["0.5", "0.7"] {
        let bad = in_section("dpad", "hub = 0.34", &format!("hub = {hub}"));
        let e = parse(&bad).expect_err(hub);
        assert!(e.contains("shape: dpad.hub"), "{hub}: {e}");
    }
    let wide = in_section("dpad", "hub = 0.34", "hub = 0.49");
    let wide = wide.replacen("stop = 0.36", "stop = 0.49", 1);
    assert!(parse(&wide).is_ok(), "a thin ring still leaves a hub");
}

#[test]
fn a_stop_inset_of_half_the_pad_or_more_is_refused() {
    // Stop.qml insets Stop as DPad.qml insets the hub, so half the pad leaves no Stop.
    for stop in ["0.5", "0.7"] {
        let bad = in_section("dpad", "stop = 0.36", &format!("stop = {stop}"));
        let e = parse(&bad).expect_err(stop);
        assert!(e.contains("shape: dpad.stop "), "{stop}: {e}");
    }
}

#[test]
fn a_stop_wider_than_the_hub_is_refused() {
    let wide = in_section("dpad", "stop = 0.36", "stop = 0.2");
    let e = parse(&wide).expect_err("Stop past the hub");
    assert!(e.contains("shape: dpad.stop 0.2"), "{e}");
    let same = in_section("dpad", "stop = 0.36", "stop = 0.34");
    assert!(parse(&same).is_ok(), "Stop may fill the hub");
    let d = load().expect("shape.toml loads").dpad;
    assert!(d.hub <= d.stop, "the shipped Stop sits inside the hub");
}

#[test]
fn the_turn_is_set_in_code_and_still_reaches_qml_and_the_gates() {
    let s = load().expect("shape.toml loads");
    assert_eq!(s.dpad.turn_deg, 45.0, "the arrows sit on the axes");
    let json = serde_json::to_value(&s.dpad).expect("the shape is JSON");
    assert_eq!(json["turn_deg"], 45.0, "DPad.qml reads it");
    assert!(!text().contains("turn_deg"), "written once, in code");
    let set = in_section("dpad", "\nreach", "\nturn_deg = 30.0\nreach");
    let e = parse(&set).expect_err("not a token");
    assert!(e.contains("unknown field `turn_deg`"), "{e}");
}
