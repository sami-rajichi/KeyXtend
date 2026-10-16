//! Tests of `[gate_input]`: what a check refuses, and that its texts are the ones the product writes.

use std::path::{Path, PathBuf};

use super::*;

// Workspace paths from `tools/kx-gates/src/p4cfg`; a moved file breaks the build here, never silently.
/// The app's command line, which names the gate flag.
const ARGS_RS: &str = include_str!("../../../../apps/keyxtend/src/args.rs");
/// The app's start, which logs the test build's warning.
const RUN_RS: &str = include_str!("../../../../apps/keyxtend/src/run.rs");

fn shipped() -> GateInput {
    crate::config::load()
        .expect("kx-gates.toml loads")
        .gate_input
}

/// `s` in double quotes, as Rust source writes a text.
fn quoted(s: &str) -> String {
    format!("\"{s}\"")
}

#[test]
fn the_shipped_section_checks() {
    assert_eq!(shipped().check(), Ok(()));
}

#[test]
fn an_empty_text_or_a_zero_wait_is_refused() {
    let bad: [fn(&mut GateInput); 5] = [
        |g| g.program.clear(),
        |g| g.flag.clear(),
        |g| g.warn_line.clear(),
        |g| g.start_ms = 0,
        |g| g.lit_ms = 0,
    ];
    for (i, f) in bad.iter().enumerate() {
        let mut g = shipped();
        f(&mut g);
        assert!(g.check().is_err(), "case {i}");
    }
}

#[test]
fn the_flag_and_the_warning_are_the_ones_the_test_build_takes_and_logs() {
    let g = shipped();
    assert!(ARGS_RS.contains(&quoted(&g.flag)), "the app's flag");
    assert!(RUN_RS.contains(&quoted(&g.warn_line)), "the app's warning");
}

/// Every Rust file under `dir`, at any depth.
fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = std::fs::read_dir(dir).expect("the folder reads");
    for path in entries.map(|e| e.expect("an entry").path()) {
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().is_some_and(|x| x == "rs") {
            out.push(path);
        }
    }
}

#[test]
fn only_the_settings_file_holds_the_flag_so_dev_install_still_signs_kx_gates() {
    let flag = shipped().flag;
    assert!(!flag.is_empty(), "kx-gates-p4.toml names it");
    let (krate, src) = (Path::new(env!("CARGO_MANIFEST_DIR")), "src");
    let mut files = vec![krate.join("build.rs"), krate.join("Cargo.toml")];
    rust_files(&krate.join(src), &mut files);
    let this = krate.join(src).join("p4cfg").join("tests.rs");
    assert!(files.contains(&this), "the walk reaches nested modules");
    for file in files {
        let text = std::fs::read_to_string(&file).expect("the source reads");
        assert!(
            !text.contains(&flag),
            "{} holds the gate flag",
            file.display()
        );
    }
}

fn g5() -> G5 {
    crate::config::load().expect("kx-gates.toml loads").g5
}

#[test]
fn the_shipped_g5_checks_and_its_hang_stays_within_target_windows_cap() {
    let g = g5();
    assert_eq!(g.check(), Ok(()));
    let cap = kx_target_window::config::load()
        .expect("kx-target-window.toml loads")
        .hang_max_ms;
    assert!(
        g.hang_ms <= cap,
        "{} ms past the cap of {cap} ms",
        g.hang_ms
    );
}

#[test]
fn g5_refuses_a_lead_past_the_hang_no_round_no_event_and_a_margin_within_the_slack() {
    let bad: [fn(&mut G5); 5] = [
        |g| g.hang_lead_ms = g.hang_ms,
        |g| g.hang_lead_ms = g.hang_ms + 1,
        |g| g.rounds = 0,
        |g| g.min_events_per_case = 0,
        |g| g.still_margin_px = g.point_slack_px,
    ];
    for (i, f) in bad.iter().enumerate() {
        let mut g = g5();
        f(&mut g);
        assert!(g.check().is_err(), "case {i}");
    }
}

#[test]
fn a_g5_table_without_max_lost_echoes_is_refused() {
    let file: toml::Table = include_str!("../../kx-gates-p4.toml")
        .parse()
        .expect("is TOML");
    let mut g5 = file["g5"].as_table().expect("a [g5] table").clone();
    let read = |t: toml::Table| toml::Value::Table(t).try_into::<G5>();
    assert!(read(g5.clone()).is_ok(), "the shipped table reads");
    g5.remove("max_lost_echoes");
    let e = read(g5).expect_err("max_lost_echoes is needed");
    assert!(e.to_string().contains("max_lost_echoes"), "{e}");
}

#[test]
fn the_hand_try_runs_a_while() {
    let hand = crate::config::load().expect("kx-gates.toml loads").hand;
    assert!(hand.try_secs > 0);
}

fn g12() -> G12 {
    crate::config::load().expect("kx-gates.toml loads").g12
}

#[test]
fn the_shipped_g12_grid_holds_the_roadmaps_200_clicks_and_checks() {
    let g = g12();
    assert_eq!(g.grid[0] * g.grid[1], 200);
    assert_eq!(g.check(), Ok(()));
}

#[test]
fn g12_refuses_an_empty_grid_bad_limits_no_ring_no_capture_and_a_csv_off_the_out_folder() {
    let bad: [fn(&mut G12); 12] = [
        |g| g.grid[1] = 0,
        |g| g.inset_px = -1,
        |g| g.slack_px = -1,
        |g| g.min_fps = f64::NAN,
        |g| g.p99_ms = 0.0,
        |g| g.ring_ms = 0,
        |g| g.show_ms = g.ring_ms,
        |g| g.presentmon_secs = 0,
        |g| g.presentmon.clear(),
        |g| g.presentmon = r"..\g12.csv".into(),
        |g| g.presentmon = "out/g12.csv".into(),
        |g| g.presentmon = r"C:\g12.csv".into(),
    ];
    for (i, f) in bad.iter().enumerate() {
        let mut g = g12();
        f(&mut g);
        assert!(g.check().is_err(), "case {i}");
    }
}

fn g8_hook() -> G8Hook {
    crate::config::load().expect("kx-gates.toml loads").g8_hook
}

#[test]
fn g8_hook_refuses_no_round_no_event_and_a_budget_not_above_0() {
    assert_eq!(g8_hook().check(), Ok(()));
    let bad: [fn(&mut G8Hook); 4] = [
        |g| g.rounds = 0,
        |g| g.min_events = 0,
        |g| g.p99_ms = 0.0,
        |g| g.p99_ms = f64::NAN,
    ];
    for (i, f) in bad.iter().enumerate() {
        let mut g = g8_hook();
        f(&mut g);
        assert!(g.check().is_err(), "case {i}");
    }
}
