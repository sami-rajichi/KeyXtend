//! Tests of G8's settings: the shipped budgets against spec §10, the product's log lines, and what a check refuses.

use super::*;

/// The product's code that writes the start line and the guard line G8 waits for.
// Workspace paths from `tools/kx-gates/src/g8cfg`; a moved crate breaks the build here, never silently.
const RUN_RS: &str = include_str!("../../../../apps/keyxtend/src/run.rs");
const FACE_RS: &str = include_str!("../../../../crates/kx-ui/src/hub/face.rs");
/// The spec, whose §10 sets the budgets G8 holds the product to.
const SPEC: &str =
    include_str!("../../../../docs/superpowers/specs/2026-09-26-accessible-keyboard-design.md");

fn shipped() -> G8 {
    crate::config::load().expect("kx-gates.toml loads").g8
}

/// The number after each `≤ ` in spec §10's row for `metric`.
fn spec_limits(metric: &str) -> Vec<f64> {
    let head = format!("| {metric} |");
    let row = SPEC.lines().find(|l| l.starts_with(&head));
    let row = row.unwrap_or_else(|| panic!("spec §10 has no {metric} row"));
    let numbers = row.split("≤ ").skip(1);
    numbers
        .filter_map(|n| n.split_whitespace().next()?.parse().ok())
        .collect()
}

#[test]
fn the_shipped_budgets_are_spec_10s() {
    let g = shipped();
    assert_eq!(g.check(), Ok(()));
    let memory = spec_limits("Main process memory, idle");
    assert_eq!(
        memory,
        [g.memory_mb, g.commit_mb],
        "working set, then commit"
    );
    assert_eq!(spec_limits("Cold start to visible keyboard"), [g.start_ms]);
    assert_eq!(
        spec_limits("Key click to character in target app"),
        [g.char_ms]
    );
    assert_eq!(
        spec_limits("Key click to key highlight"),
        [g.highlight_frames, g.highlight_worst_frames],
        "the median press, then the slowest (ADR-0025)"
    );
    assert!(g.start_ms < g.weak_start_ms, "weak mode may start slower");
    assert!(
        g.presentmon_exe.is_empty(),
        "each PC names its own PresentMon"
    );
}

#[test]
fn the_hook_budget_is_spec_10s() {
    let hook = crate::config::load().expect("kx-gates.toml loads").g8_hook;
    assert_eq!(spec_limits("Mouse hook callback"), [hook.p99_ms]);
}

#[test]
fn the_log_lines_g8_waits_for_are_the_ones_the_product_writes() {
    let g = shipped();
    let quoted = |s: &str| format!("\"{s}\"");
    assert!(RUN_RS.contains(&quoted(&g.log_start)), "{}", g.log_start);
    assert!(
        FACE_RS.contains(&quoted(&g.log_guarded)),
        "{}",
        g.log_guarded
    );
}

#[test]
fn the_bus_budget_is_a_share_of_click_to_character_in_microseconds() {
    let g = G8 {
        char_ms: 30.0,
        bus_share: 0.01,
        ..shipped()
    };
    assert!((g.bus_p99_us() - 300.0).abs() < 1e-9, "{}", g.bus_p99_us());
}

#[test]
fn a_g8_table_without_a_commit_budget_is_refused() {
    let file: toml::Table = include_str!("../../kx-gates-p3.toml")
        .parse()
        .expect("is TOML");
    let mut g8 = file["g8"].as_table().expect("a [g8] table").clone();
    let read = |t: toml::Table| toml::Value::Table(t).try_into::<G8>();
    assert!(read(g8.clone()).is_ok(), "the shipped table reads");
    g8.remove("commit_mb");
    let e = read(g8).expect_err("commit_mb is needed");
    assert!(e.to_string().contains("commit_mb"), "{e}");
}

#[test]
fn a_run_that_could_pass_with_nothing_measured_is_refused() {
    let bad: [fn(&mut G8); 14] = [
        |g| g.highlight_worst_frames = f64::NAN,
        |g| g.rise_ms = 0,
        |g| g.rise_wait_ms = g.rise_ms - 1,
        |g| g.churn_rounds = 0,
        |g| g.memory_mb = 0.0,
        |g| g.commit_mb = f64::NAN,
        |g| g.warm_ms = 0,
        |g| g.bus_share = f64::NAN,
        |g| g.presses = 0,
        |g| g.publishes = 0,
        |g| g.quit_tries = 0,
        |g| g.resize_slack_px = -1,
        |g| g.press_key = "c-0xFFFF".into(),
        |g| g.presentmon_secs = 0,
    ];
    for f in bad {
        let mut g = shipped();
        f(&mut g);
        assert!(g.check().is_err());
    }
}

#[test]
fn the_slowest_light_may_take_no_fewer_frames_than_the_median() {
    let g = shipped();
    let even = G8 {
        highlight_worst_frames: g.highlight_frames,
        ..g.clone()
    };
    assert_eq!(even.check(), Ok(()));
    let short = G8 {
        highlight_worst_frames: g.highlight_frames / 2.0,
        ..g
    };
    let e = short
        .check()
        .expect_err("a slowest budget under the median's");
    assert!(e.contains("highlight_worst_frames"), "{e}");
}
