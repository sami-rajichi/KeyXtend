//! Motion tests against the shipped `motion.toml`.

#![allow(
    clippy::float_cmp,
    reason = "token values parse to exactly the numbers the mock-up writes"
)]

use super::*;

fn text() -> String {
    crate::MOTION.to_string()
}

fn load() -> Result<Motions, String> {
    parse(crate::MOTION)
}

#[test]
fn the_mock_up_timings_and_curves_load() {
    let m = load().expect("motion.toml loads").full;
    let names: Vec<&str> = m.moves.keys().map(String::as_str).collect();
    let mut all = MOVES.to_vec();
    all.sort_unstable();
    assert_eq!(names, all);
    let ease = [0.25, 0.1, 0.25, 1.0, 1.0, 1.0];
    assert_eq!(
        m.moves["press"],
        Timed { ms: 90, bez: ease },
        ".cap transform .09s ease"
    );
    let spring = [0.3, 1.5, 0.5, 1.0, 1.0, 1.0];
    assert_eq!(
        m.moves["stop"],
        Timed {
            ms: 220,
            bez: spring
        },
        ".dp-stop"
    );
    assert_eq!(m.amount.pulse_low, 0.35, "@keyframes pulse");
}

#[test]
fn reduced_motion_keeps_the_curves_but_takes_no_time() {
    let m = load().expect("motion.toml loads");
    assert!(std::ptr::eq(m.pick(true), &raw const m.reduced));
    for (name, t) in &m.reduced.moves {
        assert_eq!(t.ms, 0, "{name}");
        assert_eq!(t.bez, m.full.moves[name].bez, "{name}");
    }
    assert_eq!(m.reduced.amount, m.full.amount);
}

#[test]
fn a_move_with_an_unknown_curve_is_refused_by_name() {
    let typo = text().replacen("curve = \"spring\"", "curve = \"sprung\"", 1);
    let e = parse(&typo).expect_err("no such curve");
    assert!(e.contains("stop") && e.contains("sprung"), "{e}");
}

#[test]
fn a_missing_or_unknown_move_is_refused() {
    let all = text();
    let gone = all.replacen("shimmer = {", "shine = {", 1);
    let e = parse(&gone).expect_err("shimmer is missing");
    assert!(e.contains("shimmer"), "{e}");
    let extra = all.replacen(
        "[amount]",
        "wobble = { ms = 5, curve = \"ease\" }\n\n[amount]",
        1,
    );
    let e = parse(&extra).expect_err("wobble is unknown");
    assert!(e.contains("wobble"), "{e}");
}

#[test]
fn a_curve_that_runs_back_in_time_or_is_not_a_number_is_refused() {
    let all = text();
    for (bad, says) in [
        ("linear = [1.2, 0.0, 1.0, 1.0]", "x1 and x2"),
        ("linear = [0.0, 0.0, -0.1, 1.0]", "x1 and x2"),
        ("linear = [0.0, inf, 1.0, 1.0]", "y1 and y2"),
    ] {
        let e = parse(&all.replacen("linear = [0.0, 0.0, 1.0, 1.0]", bad, 1)).expect_err(bad);
        assert!(e.contains("linear") && e.contains(says), "{e}");
    }
}

#[test]
fn an_amount_outside_its_range_is_refused() {
    let all = text();
    for (from, to) in [
        ("pulse_low = 0.35", "pulse_low = 1.5"),
        ("pop_scale = 0.98", "pop_scale = -1.0"),
        ("glow_lift = 1.12", "glow_lift = 0.9"),
        ("pop_rise_px = 6.0", "pop_rise_px = -1.0"),
    ] {
        let bad = all.replacen(from, to, 1);
        let e = parse(&bad).expect_err(to);
        assert!(e.contains(from.split(' ').next().unwrap_or(from)), "{e}");
    }
}

#[test]
fn the_pointer_badge_fades_briefly_and_must_be_in_the_file() {
    assert!(MOVES.contains(&"badge"), "a move the faces play");
    let m = load().expect("motion.toml loads").full;
    let ease_out = [0.0, 0.0, 0.58, 1.0, 1.0, 1.0];
    assert_eq!(
        m.moves["badge"],
        Timed {
            ms: 167,
            bez: ease_out
        },
        "WinUI's ControlFastAnimationDuration"
    );
    let gone = text().replacen("badge = {", "badges = {", 1);
    let e = parse(&gone).expect_err("badge is missing");
    assert!(e.contains("moves.badge"), "{e}");
}

#[test]
fn the_notice_and_the_menu_pop_in_as_the_mock_up_does() {
    let m = load().expect("motion.toml loads").full;
    let ease_out = [0.0, 0.0, 0.58, 1.0, 1.0, 1.0];
    assert_eq!(
        m.moves["notice"],
        Timed {
            ms: 200,
            bez: ease_out
        },
        ".toast toastIn"
    );
    assert_eq!(
        m.moves["menu"],
        Timed {
            ms: 140,
            bez: ease_out
        },
        ".ctx ctxIn"
    );
    assert_eq!(m.amount.menu_scale, 0.96, "@keyframes ctxIn");
    let bad = text().replacen("menu_scale = 0.96", "menu_scale = 1.5", 1);
    assert!(parse(&bad).expect_err("above 1").contains("menu_scale"));
}
