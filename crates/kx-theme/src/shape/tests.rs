//! Shared-size tests against the shipped `shape.toml`.

#![allow(
    clippy::float_cmp,
    reason = "token values parse to exactly the numbers the mock-up writes"
)]

use super::*;

fn text() -> String {
    crate::SHAPE.to_string()
}

fn load() -> Result<Shape, String> {
    parse(crate::SHAPE)
}

#[test]
fn the_shared_sizes_load() {
    let s = load().expect("shape.toml loads");
    assert_eq!(s.bar.button_px, [24.0, 22.0], "mock-up .tb");
    assert!(s.tip.font_px > 0.0 && s.tip.gap_px > 0.0, "tooltips");
    assert_eq!(s.line.px, 1.0, "mock-up 1px pop-line borders");
    assert_eq!((s.bar.weight, s.bar.lit_weight), (400, 600), ".tb, .tb.on");
    assert_eq!((s.lang.side_weight, s.lang.weight), (400, 600), ".car-it");
}

#[test]
fn the_extra_windows_take_the_mock_up_sizes() {
    let s = load().expect("shape.toml loads");
    assert_eq!(s.pill.button_side_px, 11.0, ".selpill button");
    assert_eq!(
        (s.pill.icon_px, s.pill.gap_px),
        (13.0, 5.0),
        ".selpill .i and gap"
    );
    assert_eq!(s.caption.icon_px, 15.0, ".capbar .i");
    assert_eq!(s.caption.gap_px, 6.0, ".capbar .st gap");
    assert_eq!(s.caption.bar_px, 38.0, ".capbar with its 26 px wave");
    assert_eq!(s.snip.bar_radius_px, 9.0, ".snip-bar");
    assert_eq!(s.caption.shimmer_px, [10.0, 6.0], ".shimmer");
    assert_eq!(s.caption.shimmer_span, 2.0, ".shimmer background-size 200%");
    assert_eq!(s.panel.row_px[0], 46.0, ".lrow");
    assert_eq!(s.panel.head_gap_px, 8.0, ".pop-h");
    assert_eq!(s.panel.off_share, 0.35, ".pg[disabled]");
    assert_eq!(s.panel.page_px[..2], [38.0, 46.0], ".pg");
    assert_eq!(s.panel.gap_px, [10.0, 8.0], "placePop");
    let heavy = text().replacen("status_weight = 600", "status_weight = 950", 1);
    assert!(
        parse(&heavy)
            .expect_err("weight")
            .contains("caption.status_weight")
    );
}

#[test]
fn a_misspelt_size_is_refused() {
    let e = parse(&text().replacen("hub = 0.34", "hob = 0.34", 1)).expect_err("a typo");
    assert!(e.contains("hob"), "{e}");
}

#[test]
fn an_odd_weight_or_share_is_refused() {
    let heavy = text().replacen("lit_weight = 600", "lit_weight = 950", 1);
    assert!(parse(&heavy).expect_err("weight").contains("950"));
    let scale = text().replacen("scale = 0.97", "scale = 1.5", 1);
    assert!(parse(&scale).expect_err("share").contains("press.scale"));
    let side = text().replacen("side = [9.0, 0.45]", "side = [9.0, -0.1]", 1);
    assert!(parse(&side).expect_err("share").contains("lang.side"));
    let flat = text().replacen("shimmer_span = 2.0", "shimmer_span = 0.0", 1);
    assert!(parse(&flat).expect_err("span").contains("shimmer_span"));
}

#[test]
fn a_panel_gap_that_is_not_a_number_or_below_0_is_refused() {
    for bad in ["nan", "-1.0", "inf"] {
        let t = text().replacen(
            "gap_px = [10.0, 8.0]",
            &format!("gap_px = [10.0, {bad}]"),
            1,
        );
        assert!(parse(&t).expect_err(bad).contains("panel.gap_px"), "{bad}");
    }
}

#[test]
fn the_notice_banner_takes_the_mock_up_sizes() {
    let s = load().expect("shape.toml loads");
    let n = &s.notice;
    assert_eq!(
        (n.pad_px, n.gap_px, n.font_px, n.icon_px),
        ([7.0, 14.0], 7.0, 12.5, 14.0),
        ".toast"
    );
    assert_eq!(n.radius_share, 0.9, ".pop");
    assert_eq!(
        (n.button_pad_px, n.button_radius_px),
        ([6.0, 11.0], 7.0),
        ".btn"
    );
    assert_eq!((n.button_px, n.button_weight), (12.0, 600), ".btn");
}

#[test]
fn the_menu_and_the_language_list_take_the_mock_up_sizes() {
    let s = load().expect("shape.toml loads");
    let m = &s.menu;
    assert_eq!(
        (m.width_px, m.pad_px, m.radius_px, m.font_px),
        (200.0, 4.0, 8.0, 13.0),
        ".ctx"
    );
    assert_eq!(
        (m.item_pad_px, m.item_radius_px, m.gap_px),
        ([7.0, 10.0], 5.0, 10.0),
        ".ctx button"
    );
    assert_eq!(
        (m.icon_px, m.icon_share, m.rule_px),
        (15.0, 0.8, [4.0, 2.0]),
        ".ctx .i, hr"
    );
    let l = &s.list;
    assert_eq!((l.width_px, l.pad_px), (220.0, 5.0), ".langs");
    assert_eq!(
        (l.row_pad_px, l.row_radius_px, l.gap_px),
        ([7.0, 8.0], 7.0, 10.0),
        ".lg-it"
    );
    assert_eq!(
        (l.badge_px, l.badge_text_px, l.badge_weight),
        ([28.0, 22.0, 5.0], [12.0, 14.0], 600),
        ".bdg"
    );
    assert_eq!(
        (l.name_px, l.sub_px, l.icon_px),
        (12.5, 10.5, 14.0),
        ".nm, small, .i"
    );
}

#[test]
fn the_language_key_designs_dimmed_keys_and_the_outline_take_the_mock_up_sizes() {
    let s = load().expect("shape.toml loads");
    let c = &s.lang.cycle;
    assert_eq!((c.icon_px, c.icon_share), (12.0, 0.75), ".cyc .i");
    assert_eq!(
        (s.lang.list.icon_px, s.lang.list.icon_share),
        (11.0, 0.7),
        ".lst .i"
    );
    assert_eq!(s.lang.icon_gap_px, 3.0, ".lk gap");
    assert_eq!(s.dim.share, 0.4, ".btn[disabled]");
    assert_eq!(s.hc_outline.px, 2.0, ".scroll-target outline");
    assert_eq!(s.backdrop.plate, 0.88, "the P1 test build's plate_alpha");
    let l = &s.legend;
    assert_eq!(
        (l.label_pad_px, l.label_min_px),
        (1.0, 7.0),
        "labels fit their caps"
    );
}

#[test]
fn the_pointer_badge_takes_the_mock_up_sizes() {
    let b = load().expect("shape.toml loads").badge;
    assert_eq!(
        (b.pad_px, b.icon_px, b.gap_px),
        ([3.0, 7.0], 13.0, 4.0),
        ".badge padding, .i, gap"
    );
    assert_eq!((b.font_px, b.weight), (10.5, 600), ".badge font");
    assert_eq!(b.offset_px, [16.0, 18.0], "updateBadge");
    assert_eq!(b.edge_px, 8.0, "placePop's least gap to the edge");
}

/// The shipped file with `from` replaced by `to` in section `[name]` only.
fn in_section(name: &str, from: &str, to: &str) -> String {
    let all = text();
    let head = format!("[{name}]");
    let (before, after) = all.split_once(&head).expect("the section");
    format!("{before}{head}{}", after.replacen(from, to, 1))
}

#[test]
fn an_unknown_or_odd_badge_size_is_refused() {
    let badge = |from, to| in_section("badge", from, to);
    let e = parse(&badge("\nicon_px", "\nhue = 1.0\nicon_px")).expect_err("unknown");
    assert!(e.contains("hue"), "{e}");
    // A pill (mock-up border-radius: 999px): QML rounds it by half its height, so no radius can drift from its size.
    let pill = badge("\nicon_px", "\nradius_px = 9.5\nicon_px");
    let e = parse(&pill).expect_err("no radius token");
    assert!(e.contains("unknown field `radius_px`"), "{e}");
    for (from, to, says) in [
        ("weight = 600", "weight = 950", "badge.weight"),
        ("edge_px = 8.0", "edge_px = -1.0", "badge.edge_px"),
        ("[16.0, 18.0]", "[16.0, nan]", "badge.offset_px"),
    ] {
        let e = parse(&badge(from, to)).expect_err(to);
        assert!(e.contains(says), "{to}: {e}");
    }
}

/// A token line as shipped, a bad value for it, and the name the error must give.
type Bad = (&'static str, &'static str, &'static str);

/// Each `Bad` token is refused with its name.
fn refused(cases: &[Bad]) {
    for (from, to, says) in cases {
        let e = parse(&text().replacen(from, to, 1)).expect_err(to);
        assert!(e.contains(says), "{to}: {e}");
    }
}

#[test]
fn an_odd_weight_or_share_in_a_p3_part_is_refused() {
    refused(&[
        (
            "button_weight = 600",
            "button_weight = 950",
            "notice.button_weight",
        ),
        (
            "badge_weight = 600",
            "badge_weight = 50",
            "list.badge_weight",
        ),
        ("\nshare = 0.4", "\nshare = 1.5", "dim.share"),
        ("icon_share = 0.8", "icon_share = -0.1", "menu.icon_share"),
        ("\npx = 2.0", "\npx = 0.0", "hc_outline.px"),
        ("plate = 0.88", "plate = 1.2", "backdrop.plate"),
        ("weight = 400", "weight = 1000", "bar.weight"),
        ("side_weight = 400", "side_weight = 0", "lang.side_weight"),
    ]);
}

#[test]
fn an_odd_tray_share_or_label_size_is_refused() {
    refused(&[
        (
            "glyph_share = 0.75",
            "glyph_share = 1.5",
            "tray.glyph_share",
        ),
        (
            "radius_share = 0.22",
            "radius_share = -1.0",
            "tray.radius_share",
        ),
        (
            "label_min_px = 7.0",
            "label_min_px = 0.0",
            "legend.label_min_px",
        ),
        (
            "label_pad_px = 1.0",
            "label_pad_px = -1.0",
            "legend.label_pad_px",
        ),
        ("icon_gap_px = 3.0", "icon_gap_px = nan", "lang.icon_gap_px"),
        (
            "mark_px = [10.0, 2.0, 2.0]",
            "mark_px = [10.0, 0.0, 2.0]",
            "bar.mark_px",
        ),
        (
            "mark_px = [10.0, 2.0, 2.0]",
            "mark_px = [10.0, 2.0, -1.0]",
            "bar.mark_px",
        ),
    ]);
}

/// The dots the language key's Cycle design and the Next key share.
mod dots;
/// The scroll pad's arrows, Stop's speed dots and the target's outline.
mod pad;
