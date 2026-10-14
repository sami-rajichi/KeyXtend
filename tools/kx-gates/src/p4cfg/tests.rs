//! Tests of `[gate_input]`: what a check refuses, and that its texts are the ones the product writes.

use std::path::{Path, PathBuf};

use super::*;
use crate::hooklog::HookLine;

// Workspace paths from `tools/kx-gates/src/p4cfg`; a moved file breaks the build here, never silently.
/// The app's command line, which names the gate flag.
const ARGS_RS: &str = include_str!("../../../../apps/keyxtend/src/args.rs");
/// The app's start, which logs the test build's warning.
const RUN_RS: &str = include_str!("../../../../apps/keyxtend/src/run.rs");
/// The mouse module's actor, which logs the hook's numbers at each hook stop.
const ACTOR_RS: &str = include_str!("../../../../crates/kx-mod-mouse/src/actor.rs");

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
    let bad: [fn(&mut GateInput); 6] = [
        |g| g.program.clear(),
        |g| g.flag.clear(),
        |g| g.warn_line.clear(),
        |g| g.hook_line.clear(),
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

#[test]
fn the_hook_line_and_its_fields_are_the_ones_the_mouse_module_logs() {
    let g = shipped();
    assert!(ACTOR_RS.contains(&quoted(&g.hook_line)), "{}", g.hook_line);
    let line = serde_json::to_value(HookLine::default()).expect("serialises");
    let fields = line.as_object().expect("an object");
    assert!(!fields.is_empty());
    for name in fields.keys() {
        let logged = format!("{name} = s.{name}");
        assert!(ACTOR_RS.contains(&logged), "actor.rs logs no {logged}");
    }
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
fn g5_refuses_a_lead_at_or_past_the_hang_no_round_and_no_event_per_case() {
    let bad: [fn(&mut G5); 4] = [
        |g| g.hang_lead_ms = g.hang_ms,
        |g| g.hang_lead_ms = g.hang_ms + 1,
        |g| g.rounds = 0,
        |g| g.min_events_per_case = 0,
    ];
    for (i, f) in bad.iter().enumerate() {
        let mut g = g5();
        f(&mut g);
        assert!(g.check().is_err(), "case {i}");
    }
}

#[test]
fn the_hand_try_runs_a_while() {
    let hand = crate::config::load().expect("kx-gates.toml loads").hand;
    assert!(hand.try_secs > 0);
}
