//! The scroll pad's sizes: where its arrows sit, Stop's speed dots, the badge's separator and the target's outline.

use super::*;

/// The shipped file with `from` replaced by `to` in section `[name]` only.
fn in_section(name: &str, from: &str, to: &str) -> String {
    let all = text();
    let head = format!("[{name}]");
    let (before, after) = all.split_once(&head).expect("the section");
    format!("{before}{head}{}", after.replacen(from, to, 1))
}

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

#[test]
fn an_odd_pad_badge_or_outline_size_is_refused() {
    for (section, from, to, says) in [
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
        ("outline", "px = 2.0", "px = nan", "outline.px"),
    ] {
        let e = parse(&in_section(section, from, to)).expect_err(to);
        assert!(e.contains(says), "{to}: {e}");
    }
}
