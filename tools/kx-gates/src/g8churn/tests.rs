//! Tests of G8's churn: its readings, its verdict, its settings and the keys it hovers.

use std::cell::Cell;

use kx_theme::Themes;
use serde_json::json;

use super::*;
use crate::budget;

fn shipped() -> G8 {
    crate::config::load().expect("kx-gates.toml loads").g8
}

fn mem(working_mb: f64, commit_mb: f64) -> Memory {
    Memory {
        working_mb,
        commit_mb,
    }
}

fn churn(before: Memory, after: Memory) -> Churn {
    whole_over(&shipped(), before, after)
}

fn failed(c: &Churn) -> Vec<&'static str> {
    let parts = parts(&shipped(), c);
    parts.iter().filter(|p| !p.pass).map(|p| p.name).collect()
}

/// `n` readings, each `step` MB of working set above the one before, from `from`.
fn climb(from: Memory, n: usize, step: f64) -> Vec<Memory> {
    let n = u32::try_from(n).expect("few rounds");
    let at = |r| mem(from.working_mb + step * f64::from(r), from.commit_mb);
    (1..=n).map(at).collect()
}

/// A churn of the shipped rounds read as `per_round`, in which every pop-up showed.
fn tracked(before: Memory, per_round: Vec<Memory>) -> Churn {
    whole_of(&shipped(), before, per_round)
}

/// `n` flat readings at `at`, with `high` MB more of both at round `spike`.
fn flat_but(at: Memory, n: usize, spike: usize, high: f64) -> Vec<Memory> {
    let mut each = vec![at; n];
    each[spike] = mem(at.working_mb + high, at.commit_mb + high);
    each
}

/// A round that only counts itself in `done`.
fn counted(done: &Cell<u32>) -> impl FnMut() -> Result<(), String> + '_ {
    || {
        done.set(done.get() + 1);
        Ok(())
    }
}

#[test]
fn a_churn_reads_memory_before_its_rounds_and_after_each_one_and_its_result_lists_them() {
    let n = shipped().churn_rounds;
    let done = Cell::new(0);
    // Each reading is the count of rounds run so far, so the list shows each followed its round.
    let read = || Ok(mem(f64::from(done.get()), 0.0));
    let (before, per_round) = readings(n, counted(&done), read).expect("every reading worked");
    assert_eq!(before, mem(0.0, 0.0));
    assert_eq!(per_round, climb(before, n, 1.0));
    let shown = serde_json::to_value(tracked(before, per_round)).expect("serialises");
    let listed = shown["per_round"].as_array().expect("a list");
    assert_eq!(listed.len(), n);
    assert_eq!(listed[0], json!({ "working_mb": 1.0, "commit_mb": 0.0 }));
    // Five rounds at each end of twenty: rounds 1-5 have the median 3, and rounds 16-20 the median 18.
    assert_eq!(
        shown["first_median"],
        json!({ "working_mb": 3.0, "commit_mb": 0.0 })
    );
    assert_eq!(
        shown["last_median"],
        json!({ "working_mb": 18.0, "commit_mb": 0.0 })
    );
}

#[test]
fn a_churn_records_its_warm_up_rounds_from_the_settings_beside_its_readings() {
    let mut g = shipped();
    g.churn_warm_rounds += 2;
    let before = mem(98.0, 138.0);
    let c = Churn::measured(&g, before, climb(before, g.churn_rounds, 1.0));
    let shown = serde_json::to_value(&c).expect("serialises");
    assert_eq!(shown["warm_rounds"], json!(g.churn_warm_rounds));
    let listed = shown["per_round"].as_array().expect("a list");
    assert_eq!(listed.len(), g.churn_rounds);
    let none = Churn::measured(&g, before, vec![]);
    assert_eq!(
        (none.first_median, none.last_median),
        (before, before),
        "no round"
    );
}

#[test]
fn a_reading_that_fails_ends_the_churn_with_its_own_error() {
    for at in [0, 2] {
        let done = Cell::new(0);
        let read = || match done.get() {
            r if r == at => Err("memory: access denied".to_string()),
            r => Ok(mem(f64::from(r), 0.0)),
        };
        let e = readings(5, counted(&done), read).expect_err("a reading failed");
        assert_eq!(e, "memory: access denied");
        assert_eq!(done.get(), at, "no round runs after a failed reading");
    }
}

#[test]
fn a_round_that_fails_mid_churn_keeps_the_readings_so_far_in_its_error_in_mb() {
    let cases = [
        (1, "98.0/138.0"),
        (3, "98.0/138.0, 99.0/139.0, 100.0/140.0"),
    ];
    for (at, so_far) in cases {
        let done = Cell::new(0);
        let round = || {
            done.set(done.get() + 1);
            match done.get() {
                r if r == at => Err("the theme cycle broke".to_string()),
                _ => Ok(()),
            }
        };
        // Each reading grows 1 MB a round from a little over a whole MB, so the error shows it rounded.
        let read = || {
            let r = f64::from(done.get());
            Ok(mem(98.04 + r, 138.04 + r))
        };
        let e = readings(5, round, read).expect_err("a round failed");
        assert!(e.starts_with("the theme cycle broke"), "{e}");
        assert!(e.contains("MB") && e.ends_with(so_far), "{e}");
    }
}

#[test]
fn growth_is_the_median_of_the_last_rounds_less_the_median_of_the_first_so_a_spike_that_falls_back_passes()
 {
    let g = shipped();
    let (before, over) = (mem(98.0, 138.0), g.churn_growth_mb + 1.0);
    let spike = flat_but(before, g.churn_rounds, g.churn_rounds / 2, over);
    assert!(failed(&tracked(before, spike)).is_empty(), "it fell back");
    let jumped = mem(before.working_mb + over, before.commit_mb + over);
    let c = tracked(before, vec![jumped; g.churn_rounds]);
    assert!(
        failed(&c).is_empty(),
        "a jump before the first round is warm-up, not growth"
    );
}

#[test]
fn an_even_number_of_median_rounds_takes_the_mean_of_the_two_middle_readings() {
    let (mut g, before) = (shipped(), mem(98.0, 138.0));
    g.churn_median_rounds = 4;
    let mut each = vec![before; g.churn_rounds];
    let first = [(98.0, 138.0), (99.0, 140.0), (97.0, 139.0), (120.0, 150.0)];
    for (r, &(w, c)) in first.iter().enumerate() {
        each[r] = mem(w, c);
    }
    let c = whole_of(&g, before, each);
    assert_eq!(c.first_median, mem(98.5, 139.5), "not the nearest rank");
    assert_eq!(c.last_median, before);
}

#[test]
fn a_steady_climb_of_three_quarters_of_an_mb_a_round_fails() {
    let before = mem(98.0, 138.0);
    let steady = climb(before, shipped().churn_rounds, 0.75);
    assert_eq!(failed(&tracked(before, steady)), ["churn_working_mb"]);
}

#[test]
fn one_high_reading_at_the_start_or_the_end_does_not_decide_the_growth() {
    let (g, before) = (shipped(), mem(98.0, 138.0));
    let over = g.churn_growth_mb + 1.0;
    for spike in [0, g.churn_rounds - 1] {
        let c = tracked(before, flat_but(before, g.churn_rounds, spike, over));
        assert!(failed(&c).is_empty(), "round {spike}");
        assert_eq!((c.first_median, c.last_median), (before, before));
    }
}

#[test]
fn flat_or_shrinking_memory_passes_and_growth_past_the_budget_fails_the_measure_that_grew() {
    let flat = churn(mem(98.0, 138.0), mem(98.4, 138.2));
    assert!(budget::all_pass(&parts(&shipped(), &flat)));
    assert!(failed(&churn(mem(98.0, 138.0), mem(90.0, 130.0))).is_empty());
    let g = shipped();
    let grew = g.churn_growth_mb + 0.5;
    let commit = churn(mem(98.0, 138.0), mem(98.0, 138.0 + grew));
    assert_eq!(failed(&commit), ["churn_commit_mb"]);
    let working = churn(mem(98.0, 138.0), mem(98.0 + grew, 138.0));
    assert_eq!(failed(&working), ["churn_working_mb"]);
    let at = churn(mem(98.0, 138.0), mem(98.0 + g.churn_growth_mb, 138.0));
    assert!(failed(&at).is_empty(), "the budget itself is allowed");
}

#[test]
fn a_churn_is_whole_when_every_pop_up_showed_and_a_design_without_a_list_still_counts() {
    let full = churn(mem(98.0, 138.0), mem(98.0, 138.0));
    assert!(full.whole());
    let no_list = Churn {
        lists: None,
        ..full.clone()
    };
    assert!(no_list.whole(), "the default Carousel design opens no list");
    let short = full.rounds - 1;
    let missed: [fn(&mut Churn, usize); 6] = [
        |c, n| c.tips = n,
        |c, n| c.lists = Some(n),
        |c, n| c.bubbles = n,
        |c, n| c.returns = n,
        |c, n| c.scrolls = n,
        |c, n| c.outlines = n,
    ];
    for (i, miss) in missed.iter().enumerate() {
        let mut c = full.clone();
        miss(&mut c, short);
        assert!(
            !c.whole(),
            "case {i}: a pop-up or the keyboard's return was missed"
        );
    }
}

#[test]
fn a_churn_needs_every_scroll_and_a_control_that_left_it_out_is_never_whole() {
    let full = churn(mem(98.0, 138.0), mem(98.0, 138.0));
    let unscrolled = Churn {
        scrolls: 0,
        outlines: 0,
        ..full.clone()
    };
    assert!(!unscrolled.whole(), "the scroll step was meant to run");
    let control = Churn {
        scrolling: false,
        ..unscrolled
    };
    assert!(!control.whole(), "[g8_scroll] churn left the scroll out");
}

#[test]
fn churn_settings_that_churn_nothing_are_refused() {
    assert_eq!(check(&shipped()), Ok(()));
    let bad: [fn(&mut G8); 8] = [
        |g| g.churn_rounds = 0,
        |g| g.churn_growth_mb = 0.0,
        |g| g.churn_growth_mb = f64::INFINITY,
        |g| g.churn_tips.clear(),
        |g| g.churn_tips.push("c-0xFFFF".into()),
        |g| g.churn_theme.clear(),
        |g| g.churn_median_rounds = 0,
        |g| g.churn_median_rounds = g.churn_rounds / 2 + 1,
    ];
    for (i, f) in bad.iter().enumerate() {
        let mut g = shipped();
        f(&mut g);
        assert!(check(&g).is_err(), "case {i}");
    }
}

#[test]
fn the_medians_may_take_half_the_rounds_each_but_no_more() {
    let mut g = shipped();
    g.churn_median_rounds = g.churn_rounds / 2;
    assert_eq!(check(&g), Ok(()));
    g.churn_median_rounds += 1;
    let e = check(&g).expect_err("the two ends would share rounds");
    assert!(e.contains("churn_median_rounds"), "{e}");
}

#[test]
fn the_shipped_churn_warms_up_for_five_rounds_as_memory_settles_over_them() {
    assert_eq!(shipped().churn_warm_rounds, 5, "the owner's choice, P3.md");
}

#[test]
fn the_shipped_churn_compares_the_medians_of_five_rounds_at_each_end() {
    assert_eq!(
        shipped().churn_median_rounds,
        5,
        "the owner's choice, P3.md"
    );
}

#[test]
fn each_hovered_key_has_an_icon_so_its_tip_shows() {
    // A key's tip shows its name when it has an icon (Key.qml); every theme shares icons.toml.
    let icons = Themes::shipped().expect("the tokens read").icons.keys;
    for id in &shipped().churn_tips {
        assert!(icons.contains_key(id), "{id}");
    }
}
