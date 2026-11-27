//! Snip's sizes (`tokens/snip.toml`, design/snip-parts.css): the capture bar B "Keycaps", the hint, the strip, the
//! countdown, the red edge, the record bar and the editor's steps, and the rules they keep.

use super::*;

/// The shipped file with `from` changed to `to` once, which must be there.
fn with(from: &str, to: &str) -> String {
    let t = text();
    assert!(t.contains(from), "{from}");
    t.replacen(from, to, 1)
}

#[test]
fn the_capture_bar_hint_and_strip_take_the_mock_up_sizes() {
    let s = load().expect("shape.toml loads").snip;
    assert_eq!(s.bar_top_px, 12.0, ".cbar top");
    assert_eq!(
        (s.plate_pad_px, s.key_px),
        (3.0, 34.0),
        ".cb-keys, .cb-keys .cw"
    );
    assert_eq!(s.key_pad_px, [11.0, 9.0], ".cb-keys .cw, .q");
    assert_eq!(
        (s.word_px, s.icon_px, s.icon_gap_px),
        (12.0, 16.0, 6.0),
        ".cw, .i"
    );
    assert_eq!(
        (s.sep_px, s.ring_px),
        (6.0, 2.0),
        ".cb-keys .sep, [aria-pressed]"
    );
    assert_eq!(s.hint_pad_px, [6.0, 12.0], ".hint");
    assert_eq!(s.hint_px, 12.5, ".hint");
    assert_eq!(
        s.bar_top_px + s.key_px + 2.0 * s.plate_pad_px + s.hint_gap_px,
        72.0,
        "hint top 72"
    );
    assert_eq!(s.gap_px, 10.0, "strip r.y + r.h + 10");
    assert_eq!((s.tag_pad_px, s.tag_gap_px), ([4.0, 7.0], 24.0), ".tag");
}

#[test]
fn the_countdown_edge_and_record_bar_take_the_mock_up_sizes() {
    let s = load().expect("shape.toml loads").snip;
    assert_eq!(
        (s.count_px, s.count_num_px, s.count_weight),
        (150.0, 64.0, 600),
        ".count"
    );
    assert_eq!(
        (s.count_word_px, s.count_word_w_px),
        (11.5, 120.0),
        ".count .w"
    );
    assert_eq!((s.edge_px, s.soon_px), (2.0, 2.0), ".edge, .soon");
    assert_eq!((s.rec_px, s.rec_pad_px), (40.0, [4.0, 12.0]), ".recbar");
    assert_eq!((s.rec_key_px, s.rec_key_pad_px), (32.0, 8.0), ".rb");
    assert_eq!(s.rec_sep_px, 6.0, ".recbar .sep margin");
    assert_eq!(
        (s.rec_dot_px, s.rec_time_px, s.rec_time_weight),
        (10.0, 13.0, 600),
        ".dot, .time"
    );
}

#[test]
fn the_editor_steps_hold_one_size_each_and_reach_back_in_order() {
    let s = load().expect("shape.toml loads").snip;
    let steps = s.pen_px.len();
    assert!(steps > 1, "a thickness to pick");
    assert_eq!(
        [s.marker_px.len(), s.disc_px.len(), s.head_px.len()],
        [steps; 3]
    );
    assert!(s.text_px.len() > 1 && s.slop_px > 0);
    for list in [&s.pen_px, &s.marker_px, &s.disc_px, &s.head_px, &s.text_px] {
        assert!(list.windows(2).all(|w| w[0] < w[1]), "{list:?} grows");
    }
    assert!(
        s.marker_share > 0.0 && s.marker_share < 1.0,
        "the highlighter lets the picture through"
    );
}

#[test]
fn uneven_or_empty_steps_and_an_edge_wider_than_its_gap_are_refused() {
    let pen = "pen_px = [2, 4, 6, 10]";
    let cases = [
        (with(pen, "pen_px = [2, 4, 6]"), "snip.pen_px"),
        (with(pen, "pen_px = []"), "snip.pen_px"),
        (
            with("text_px = [16, 20, 28, 36]", "text_px = []"),
            "snip.text_px",
        ),
        (with("glow_px = 6.0", "glow_px = 9.0"), "snip.glow_px"),
        (
            with("marker_share = 0.4", "marker_share = 1.4"),
            "snip.marker_share",
        ),
        (
            with("rec_time_weight = 600", "rec_time_weight = 950"),
            "snip.rec_time_weight",
        ),
        (with("key_px = 34.0", "key_px = 0.0"), "snip.key_px"),
    ];
    for (bad, name) in cases {
        let e = parse(&bad).expect_err(name);
        assert!(e.contains(name), "{name}: {e}");
    }
}

#[test]
fn every_size_is_checked_and_the_text_heights_keep_their_own_count() {
    let cases = [
        (with("glow_px = 6.0", "glow_px = -1.0"), "snip.glow_px"),
        (
            with("dash_px = [4.0, 3.0]", "dash_px = [4.0, 0.0]"),
            "snip.dash_px",
        ),
        (
            with("tag_pad_px = [4.0, 7.0]", "tag_pad_px = [-1.0, 7.0]"),
            "snip.tag_pad_px",
        ),
        (
            with("count_word_w_px = 120.0", "count_word_w_px = 0.0"),
            "snip.count_word_w_px",
        ),
        (
            with("rec_key_pad_px = 8.0", "rec_key_pad_px = -2.0"),
            "snip.rec_key_pad_px",
        ),
        (
            with("rec_sep_px = 6.0", "rec_sep_px = -1.0"),
            "snip.rec_sep_px",
        ),
        (
            with("rec_sep_px = 6.0", "rec_sep_px = 20.0"),
            "snip.rec_sep_px",
        ),
    ];
    for (bad, name) in cases {
        let e = parse(&bad).expect_err(name);
        assert!(e.contains(name), "{name}: {e}");
    }
    let fewer = with("text_px = [16, 20, 28, 36]", "text_px = [16, 20, 28]");
    assert!(
        parse(&fewer).is_ok(),
        "text heights need not match the pen's steps"
    );
}

#[test]
fn the_editor_question_and_panel_take_the_mock_up_sizes() {
    let s = load().expect("shape.toml loads").snip;
    assert_eq!(
        s.editor_rows_px,
        [36.0, 44.0, 40.0],
        ".ed-title, .ed-row, .opts"
    );
    assert_eq!(
        s.editor_pad_px,
        [8.0, 14.0],
        ".ed-row padding, .canvas padding"
    );
    assert_eq!(s.ink_px, [24.0, 2.0, 4.0], ".ink-b, its border, .inks gap");
    assert_eq!(
        s.typing_gap_px, 6.0,
        "the typing box at 380, its text at 386"
    );
    assert_eq!(
        (s.ask_px, s.panel_px),
        ([440.0, 18.0], 500.0),
        ".ask, .panel"
    );
    assert_eq!(
        (s.head_deg, s.mark_weight),
        (25.0, 600),
        "the arrow and text drawn"
    );
    assert!(
        (s.step_share - 13.0 / 22.0).abs() < 0.01,
        "13 px on a 22 px disc"
    );
}

#[test]
fn an_arrow_head_that_does_not_open_and_odd_editor_sizes_are_refused() {
    let cases = [
        (with("head_deg = 25.0", "head_deg = 0.0"), "snip.head_deg"),
        (with("head_deg = 25.0", "head_deg = 90.0"), "snip.head_deg"),
        (
            with("step_share = 0.6", "step_share = 1.5"),
            "snip.step_share",
        ),
        (
            with("mark_weight = 600", "mark_weight = 950"),
            "snip.mark_weight",
        ),
        (
            with("editor_rows_px = [36.0", "editor_rows_px = [0.0"),
            "snip.editor_rows_px",
        ),
        (
            with("ink_px = [24.0, 2.0, 4.0]", "ink_px = [24.0, 2.0, -1.0]"),
            "snip.ink_px",
        ),
        (with("panel_px = 500.0", "panel_px = 0.0"), "snip.panel_px"),
        (
            with("typing_gap_px = 6.0", "typing_gap_px = -1.0"),
            "snip.typing_gap_px",
        ),
    ];
    for (bad, name) in cases {
        let e = parse(&bad).expect_err(name);
        assert!(e.contains(name), "{name}: {e}");
    }
}

#[test]
fn a_frames_handles_show_and_their_click_area_holds_them_at_wcags_least_target() {
    let s = load().expect("shape.toml loads").snip;
    assert!(s.handle_hit_px >= 24.0, "WCAG 2.2 2.5.8: 24 by 24");
    assert!(s.handle_px > 0.0 && s.handle_px <= s.handle_hit_px);
    let cases = [
        (
            with("handle_px = 10.0", "handle_px = 0.0"),
            "snip.handle_px",
        ),
        (
            with("handle_hit_px = 32.0", "handle_hit_px = 8.0"),
            "snip.handle_hit_px",
        ),
    ];
    for (bad, name) in cases {
        let e = parse(&bad).expect_err(name);
        assert!(e.contains(name), "{name}: {e}");
    }
}

#[test]
fn every_snip_size_falls_under_a_rule() {
    let s = load().expect("shape.toml loads").snip;
    let sizes = s.drawn().into_iter().chain(s.spaces()).map(|(n, _)| n);
    let named: Vec<String> = sizes.map(|n| n.replacen("snip.", "", 1)).collect();
    let others = [
        "tag_weight",
        "count_weight",
        "rec_time_weight",
        "mark_weight",
        "glow_share",
        "marker_share",
        "step_share",
        "head_deg",
        "editor_px",
        "editor_min_px",
        "pen_px",
        "marker_px",
        "disc_px",
        "head_px",
        "text_px",
        "slop_px",
    ];
    let fields = toml::Table::try_from(&s).expect("a table");
    let loose: Vec<&String> = fields
        .keys()
        .filter(|k| !named.contains(k) && !others.contains(&k.as_str()))
        .collect();
    assert!(loose.is_empty(), "no rule for {loose:?}");
}
