//! Tests of G8's verdict: each spec §10 budget's part, the key light's two, the memory's after use and after a scroll,
//! and when a run passes.

use serde_json::json;

use super::*;

/// Idle CPU time of a QML animation at 60 Hz and 0.5 ms a frame over a 10 s measure, and that measure, in ms.
const ANIMATION_MS: (f64, f64) = (300.0, 10_000.0);

/// A run like the P1 numbers (ADR-0013: G2 p99 13.1 ms, 165 Hz) on the shipped budgets.
fn sample() -> (G8, Measured, Outside) {
    let g = crate::config::load().expect("kx-gates.toml loads").g8;
    // Flat memory under churn, near ADR-0021's expected idle.
    let still = Memory {
        working_mb: 98.0,
        commit_mb: 138.0,
    };
    let churn = g8churn::whole_over(&g, still, still);
    let m = Measured {
        log: Some((180.0, 1074.3)),
        memory: Memory {
            working_mb: 61.0,
            commit_mb: 90.0,
        },
        memory_at_ms: 10_000.0,
        core_pct: Some(0.1),
        presses: Presses {
            lit_ms: vec![Some(9.0), Some(12.5)],
            copy_ms: vec![2.0, 2.5, 3.0],
            ..Presses::default()
        },
        resizes: vec![],
        g2: json!({
            "planned": 100, "clicks": 100, "matched": 100, "focus_ok": true, "caret_kept": 100,
            "latency_ms": { "p50": 4.2, "p99": 13.1 },
        }),
        ran_us: 0,
        churn,
        used: Ok(g8use::read_as(still)),
        scrolled: Ok(g8scroll::read_as(still)),
        scroll_left: Vec::new(),
    };
    let o = Outside {
        hz: Some(165.0),
        bus_p99_us: Some(4.0),
        dropped: Some(0.0),
    };
    (g, m, o)
}

fn failed(parts: &[Part]) -> Vec<&str> {
    parts.iter().filter(|p| !p.pass).map(|p| p.name).collect()
}

#[test]
fn a_run_within_every_budget_passes() {
    let (g, m, o) = sample();
    let parts = parts(&g, Some(420.0), &m, false, &o);
    assert!(budget::all_pass(&parts), "{:?}", failed(&parts));
    assert_eq!(
        parts.len(),
        15,
        "one part per spec §10 budget G8 owns, two for the key light, two for the memory after use and two after a \
         scroll"
    );
}

#[test]
fn a_reading_after_a_scroll_that_was_not_taken_fails_both_its_parts_and_the_run() {
    let (g, mut m, o) = sample();
    m.presses.typed_ok = true;
    m.scrolled = Err("the scroll pad has no box".into());
    let parts = parts(&g, Some(420.0), &m, false, &o);
    assert_eq!(failed(&parts), [g8scroll::MEMORY, g8scroll::COMMIT]);
    assert_eq!(
        verdict(false, &m.churn, met(true, &m.presses, &parts)),
        json!(false)
    );
}

#[test]
fn memory_over_its_budget_after_use_fails_its_own_part_and_the_run() {
    let (g, mut m, o) = sample();
    m.presses.typed_ok = true;
    if let Ok(u) = m.used.as_mut() {
        u.memory.commit_mb = g.commit_mb + 1.0;
    }
    let parts = parts(&g, Some(420.0), &m, false, &o);
    assert_eq!(failed(&parts), [g8use::COMMIT]);
    assert_eq!(
        verdict(false, &m.churn, met(true, &m.presses, &parts)),
        json!(false)
    );
}

#[test]
fn a_reading_after_use_that_was_not_taken_fails_both_its_parts_and_the_run() {
    let (g, mut m, o) = sample();
    m.presses.typed_ok = true;
    let whole = parts(&g, Some(420.0), &m, false, &o);
    assert_eq!(
        verdict(false, &m.churn, met(true, &m.presses, &whole)),
        json!(true)
    );
    m.used = Err("the mode key did not read on within 3000 ms".into());
    let parts = parts(&g, Some(420.0), &m, false, &o);
    assert_eq!(failed(&parts), [g8use::MEMORY, g8use::COMMIT]);
    let missing = parts.iter().filter(|p| !p.pass).all(|p| p.got.is_none());
    assert!(missing, "not measured");
    assert_eq!(
        verdict(false, &m.churn, met(true, &m.presses, &parts)),
        json!(false)
    );
}

#[test]
fn presses_that_typed_wrong_or_a_product_left_running_fail_the_run() {
    let (g, mut m, o) = sample();
    m.presses.typed_ok = true;
    let parts = parts(&g, Some(420.0), &m, false, &o);
    assert!(met(true, &m.presses, &parts));
    assert!(!met(false, &m.presses, &parts), "the product never quit");
    m.presses.typed_ok = false;
    assert!(
        !met(true, &m.presses, &parts),
        "the presses' characters did not all arrive alone"
    );
}

#[test]
fn memory_that_grew_under_churn_fails_g8() {
    let (g, mut m, o) = sample();
    m.churn.last_median.commit_mb += g.churn_growth_mb + 1.0;
    assert_eq!(
        failed(&parts(&g, Some(420.0), &m, false, &o)),
        ["churn_commit_mb"]
    );
}

#[test]
fn a_working_set_or_a_commit_over_its_budget_fails_its_part() {
    let (g, mut m, o) = sample();
    let over = |working_mb, commit_mb| Memory {
        working_mb,
        commit_mb,
    };
    m.memory = over(g.memory_mb + 1.0, g.commit_mb);
    assert_eq!(
        failed(&parts(&g, Some(420.0), &m, false, &o)),
        ["memory_mb"]
    );
    m.memory = over(g.memory_mb, g.commit_mb + 1.0);
    assert_eq!(
        failed(&parts(&g, Some(420.0), &m, false, &o)),
        ["commit_mb"],
        "a trim lowers the working set only, so commit is judged too"
    );
}

#[test]
fn weak_mode_allows_a_slower_cold_start_only() {
    let (g, m, o) = sample();
    assert_eq!(failed(&parts(&g, Some(700.0), &m, false, &o)), ["start_ms"]);
    assert!(budget::all_pass(&parts(&g, Some(700.0), &m, true, &o)));
    assert_eq!(
        failed(&parts(&g, None, &m, true, &o)),
        ["start_ms"],
        "never shown"
    );
}

#[test]
fn the_slowest_light_may_come_one_refresh_after_the_median() {
    let (g, mut m, o) = sample();
    // At 165 Hz with a 2.5 ms median copy: 14.6 ms for the median and 20.7 ms for the slowest.
    m.presses.lit_ms = vec![Some(9.0), Some(12.5), Some(20.0)];
    assert!(budget::all_pass(&parts(&g, Some(420.0), &m, false, &o)));
    m.presses.lit_ms.push(Some(21.0));
    assert_eq!(
        failed(&parts(&g, Some(420.0), &m, false, &o)),
        [LIT_WORST],
        "three frames at 165 Hz plus a copy"
    );
    m.presses.lit_ms = vec![Some(15.0), Some(15.0), Some(16.0)];
    assert_eq!(
        failed(&parts(&g, Some(420.0), &m, false, &o)),
        [LIT_P50],
        "two frames at 165 Hz plus a copy"
    );
}

#[test]
fn a_lost_press_no_refresh_rate_or_no_presentmon_csv_fails_its_parts() {
    let (g, mut m, mut o) = sample();
    m.presses.lit_ms = vec![Some(5.0), None];
    o.dropped = None;
    assert_eq!(
        failed(&parts(&g, Some(420.0), &m, false, &o)),
        [LIT_P50, LIT_WORST, "dropped_frames"]
    );
    let (_, m, _) = sample();
    let no_hz = highlight(&g, &m.presses, None);
    assert!(
        no_hz.iter().all(|p| !p.pass && p.got.is_none()),
        "{no_hz:?}"
    );
}

#[test]
fn an_idle_animation_or_a_lost_character_fails_its_part() {
    let (g, mut m, o) = sample();
    m.core_pct = budget::core_pct(ANIMATION_MS.0, ANIMATION_MS.1);
    m.g2["matched"] = json!(99);
    assert_eq!(
        failed(&parts(&g, Some(420.0), &m, false, &o)),
        ["idle_core_pct", "char_p99_ms"]
    );
}

#[test]
fn weak_mode_a_control_churn_or_missed_pop_ups_are_partial_and_a_miss_fails() {
    let (_, m, _) = sample();
    let whole = &m.churn;
    let no_list = Churn {
        lists: None,
        ..whole.clone()
    };
    let no_bubble = Churn {
        bubbles: whole.rounds - 1,
        ..whole.clone()
    };
    assert_eq!(verdict(false, whole, true), json!(true));
    assert_eq!(
        verdict(false, &no_list, true),
        json!(true),
        "the default design"
    );
    assert_eq!(verdict(true, whole, true), json!(out::PARTIAL));
    assert_eq!(verdict(false, &no_bubble, true), json!(out::PARTIAL));
    let control = Churn {
        scrolling: false,
        ..whole.clone()
    };
    assert_eq!(
        verdict(false, &control, true),
        json!(out::PARTIAL),
        "a control churn left the scroll out"
    );
    assert_eq!(verdict(true, whole, false), json!(false));
    assert_eq!(verdict(false, whole, false), json!(false));
}

#[test]
fn a_g2_error_or_a_slow_bus_fails_its_part() {
    let (g, mut m, mut o) = sample();
    m.g2["error"] = json!("the target could not be brought back");
    o.bus_p99_us = Some(g.bus_p99_us() + 1.0);
    assert_eq!(
        failed(&parts(&g, Some(420.0), &m, false, &o)),
        ["char_p99_ms", "bus_p99_us"]
    );
}
