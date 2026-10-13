//! Tests of G8's churn memory by step: the slope, each step's change and how the steps add up to the round.

use serde_json::json;

use super::*;
use crate::g8churn::whole_of;

fn mem(working_mb: f64, commit_mb: f64) -> Memory {
    Memory {
        working_mb,
        commit_mb,
    }
}

/// Exact in binary floats, so sums and slopes compare equal.
const QUARTER: f64 = 0.25;
const EIGHTH: f64 = 0.125;

#[test]
fn the_slope_of_a_line_is_its_growth_per_round_and_two_points_are_needed() {
    let line: Vec<f64> = (0..20).map(|r| 100.0 + QUARTER * f64::from(r)).collect();
    assert_eq!(slope(&line), Some(QUARTER));
    assert_eq!(slope(&[98.0; 5]), Some(0.0), "flat");
    assert_eq!(slope(&[98.0]), None);
    assert_eq!(slope(&[]), None);
    // One high reading in the middle of a flat run tilts nothing.
    assert_eq!(slope(&[1.0, 1.0, 6.0, 1.0, 1.0]), Some(0.0));
    // A jump between the halves: the fit spreads it over the rounds.
    assert_eq!(slope(&[0.0, 0.0, 1.0, 1.0]), Some(0.4));
}

#[test]
fn the_round_slope_fits_the_working_set_and_the_commit_apart() {
    let readings: Vec<Memory> = (0..4)
        .map(|r| mem(f64::from(r) * QUARTER, f64::from(r) * EIGHTH))
        .collect();
    assert_eq!(slopes(&readings), Some(mem(QUARTER, EIGHTH)));
    assert_eq!(slopes(&readings[..1]), None);
}

/// `n` rounds from `before` in which tips add `tips`, the bubble nothing and the themes `themes`, with the round's own
/// reading equal to its last step's.
fn rounds(
    before: Memory,
    n: usize,
    tips: Memory,
    themes: Memory,
) -> (Vec<Memory>, Vec<Vec<(Step, Memory)>>) {
    let add = |a: Memory, b: Memory| mem(a.working_mb + b.working_mb, a.commit_mb + b.commit_mb);
    let (mut at, mut per_round, mut after) = (before, Vec::new(), Vec::new());
    for _ in 0..n {
        let t = add(at, tips);
        at = add(t, themes);
        after.push(vec![(Step::Tips, t), (Step::Bubble, t), (Step::Themes, at)]);
        per_round.push(at);
    }
    (per_round, after)
}

#[test]
fn each_step_lists_what_it_changed_each_round_and_its_slope_and_the_steps_add_up_to_the_round() {
    let before = mem(98.0, 138.0);
    let (per_round, after) = rounds(before, 6, mem(QUARTER, 0.0), mem(EIGHTH, QUARTER));
    let steps = growth(before, &per_round, &after);
    let names: Vec<Step> = steps.iter().map(|s| s.step).collect();
    assert_eq!(
        names,
        [Step::Tips, Step::Bubble, Step::Themes],
        "in the order they ran"
    );
    assert_eq!(steps[0].per_round, vec![mem(QUARTER, 0.0); 6]);
    assert_eq!(steps[1].per_round, vec![mem(0.0, 0.0); 6]);
    assert_eq!(steps[2].slope, Some(mem(EIGHTH, QUARTER)));
    let total = slopes(&per_round).expect("six rounds");
    let summed = steps
        .iter()
        .filter_map(|s| s.slope)
        .fold(mem(0.0, 0.0), |a, s| {
            mem(a.working_mb + s.working_mb, a.commit_mb + s.commit_mb)
        });
    assert_eq!(summed, total);
    let shown = serde_json::to_value(&steps[1]).expect("serialises");
    assert_eq!(shown["step"], json!("bubble"));
    assert_eq!(
        shown["slope"],
        json!({ "working_mb": 0.0, "commit_mb": 0.0 })
    );
}

#[test]
fn a_round_starts_from_the_reading_after_the_round_before_so_nothing_between_rounds_is_lost() {
    let before = mem(98.0, 138.0);
    let after = vec![
        vec![
            (Step::Tips, mem(99.0, 138.0)),
            (Step::Themes, mem(99.0, 139.0)),
        ],
        vec![
            (Step::Tips, mem(101.0, 140.0)),
            (Step::Themes, mem(101.0, 140.0)),
        ],
    ];
    // The first round's own reading came a little later than its last step's.
    let per_round = vec![mem(100.0, 139.0), mem(101.0, 140.0)];
    let steps = growth(before, &per_round, &after);
    assert_eq!(steps[0].per_round, [mem(1.0, 0.0), mem(1.0, 1.0)]);
    assert_eq!(steps[1].per_round, [mem(0.0, 1.0), mem(0.0, 0.0)]);
    assert!(growth(before, &[], &[]).is_empty(), "no round");
}

#[test]
fn a_failed_reading_after_a_step_names_the_step() {
    assert_eq!(
        Step::Bubble.to_string(),
        "bubble",
        "as the result writes it"
    );
    let failed = read_after(Step::Themes, Err("access denied".to_string()));
    assert_eq!(
        failed,
        Err("memory after the themes step: access denied".to_string())
    );
    let m = mem(98.0, 138.0);
    assert_eq!(read_after(Step::Tips, Ok(m)), Ok((Step::Tips, m)));
}

#[test]
fn a_churns_result_fits_its_rounds_slope_beside_the_readings_it_already_lists() {
    let g = crate::config::load().expect("kx-gates.toml loads").g8;
    let n = u32::try_from(g.churn_rounds).expect("few rounds");
    let per_round: Vec<Memory> = (1..=n)
        .map(|r| mem(98.0 + QUARTER * f64::from(r), 138.0))
        .collect();
    let c = whole_of(&g, mem(98.0, 138.0), per_round);
    let shown = serde_json::to_value(&c).expect("serialises");
    assert_eq!(
        shown["round_slope"],
        json!({ "working_mb": QUARTER, "commit_mb": 0.0 })
    );
    assert_eq!(shown["step_growth"], json!([]), "a real run fills it");
    assert_eq!(
        shown["per_round"].as_array().map(Vec::len),
        Some(g.churn_rounds)
    );
}
