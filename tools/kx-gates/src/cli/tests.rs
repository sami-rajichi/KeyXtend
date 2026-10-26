//! Tests of `cli`.

use super::*;

fn args(s: &str) -> Vec<String> {
    s.split_whitespace().map(String::from).collect()
}

#[test]
fn parses_each_gate() {
    let a = parse(&args("g1 notepad --count 5 --seed 7")).expect("parses");
    assert_eq!(
        a,
        Args {
            gate: "g1".into(),
            name: "notepad".into(),
            count: Some(5),
            seed: Some(7),
            attach: false,
            pause_ms: None
        }
    );
    let a = parse(&args("g1 word --attach --pause 100")).expect("parses");
    assert!(a.attach);
    assert_eq!(a.pause_ms, Some(100));
    assert_eq!(
        parse(&args("g2 keyxtend --clicks 10"))
            .expect("parses")
            .count,
        Some(10)
    );
    assert_eq!(parse(&args("g3 keyxtend")).expect("parses").count, None);
    assert_eq!(parse(&args("g5 plain")).expect("parses").name, "plain");
    assert_eq!(parse(&args("g5 admin")).expect("parses").name, "admin");
    let a = parse(&args("assist right --secs 30")).expect("parses");
    assert_eq!((a.name.as_str(), a.count), ("right", Some(30)));
    let a = parse(&args("g4 keyxtend --clicks 50 --seed 3")).expect("parses");
    assert_eq!(
        (a.gate.as_str(), a.count, a.seed),
        ("g4", Some(50), Some(3))
    );
}

#[test]
fn a_click_run_counts_its_option_or_setting_and_refuses_one_past_the_guards_limit() {
    let cfg = config::load().expect("loads");
    let count = |s: &str| clicks(&cfg, &parse(&args(s)).expect("parses"));
    assert_eq!(count("g2 keyxtend --clicks 5"), Ok(5));
    assert_eq!(count("g2 keyxtend"), Ok(cfg.g2.clicks));
    assert_eq!(count("g4 keyxtend"), Ok(cfg.g4.clicks));
    let e = count("g4 keyxtend --clicks 100000").expect_err("past the limit");
    assert!(e.contains("g4") && e.contains("limit"), "{e}");
}

/// Command lines of gates that take only a name, each of which `parse` must take.
const NAME_ONLY: &[&str] = &[
    "g6 chrome",
    "g6 hang",
    "g6 unaware",
    "g7 qt",
    "g12 keyxtend",
    "g17 explorer",
    "g18 notepad",
    "g19 core",
    "g20 notepad",
    "g21 win32",
    "g19 qt",
    "g22 qt",
    "g22 bench",
    "g22 bench-cloud",
    "g22 typing",
    "g23 qt",
    "g24 qt",
    "g25 notepad",
    "close qt",
    "close keyxtend",
    "g8 keyxtend",
    "g8 weak",
    "g8 arabic",
    "g8 hook",
    "g8 hook-weak",
    "g9 keyxtend",
    "g10 keyxtend",
    "g11 keyxtend",
    "g13 keyxtend",
    "g26 keyxtend",
    "g21 keyxtend",
    "g27 keyxtend",
    "g8 predict",
];

#[test]
fn parses_gates_that_take_only_a_name() {
    for probe in NAME_ONLY {
        let a = parse(&args(probe)).expect("parses");
        assert_eq!((a.count, a.seed), (None, None));
        assert!(named(&a.gate).is_some(), "{probe} has no run");
    }
}

#[test]
fn a_keyboard_gate_refuses_a_face_that_is_gone() {
    let e = parse(&args("g2 slint")).expect_err("slint is gone");
    assert!(e.contains("unknown keyboard"), "{e}");
}

/// Command lines `parse` must refuse: missing parts, options a gate does not take, and faces that are gone.
const BAD_INPUT: &[&str] = &[
    "",
    "g1",
    "g9 x",
    "g1 target --clicks 3",
    "g3 keyxtend --seed 1",
    "g2 keyxtend --clicks",
    "g1 target --count x",
    "g2 keyxtend --attach",
    "g2 keyxtend --pause 5",
    "g1 word --pause",
    "g4 keyxtend --attach",
    "g4 keyxtend --count 5",
    "g4 keyxtend --pause 5",
    "g5 plain --seed 1",
    "assist grab --seed 1",
    "g17 notepad --seed 1",
    "g18 chrome --clicks 5",
    "g6 word --count 2",
    "g19 core --seed 1",
    "g20 notepad --count 3",
    "g21 chrome --clicks 2",
    "g22",
    "g22 bench --seed 1",
    "g22 typing --pause 10",
    "g25 notepad --attach",
    "g23 qt --seed 2",
    "g24 qt --clicks 3",
    "g7 qt --seed 1",
    "g12 keyxtend --seed 1",
    "g12 keyxtend --clicks 5",
    "g12 qt",
    "g2 qt",
    "g3 qt",
    "g4 qt --clicks 5",
    "g7 keyxtend",
    "g24 keyxtend",
];

#[test]
fn rejects_bad_input() {
    for bad in BAD_INPUT {
        assert!(parse(&args(bad)).is_err(), "{bad} should fail");
    }
}

#[test]
fn named_gates_take_no_option_and_only_their_names() {
    for bad in [
        "g8 qt",
        "g8 keyxtend --clicks 5",
        "g26 keyxtend --clicks 5",
        "g11 keyxtend --seed 1",
        "g27 qt",
        "g27 keyxtend --seed 1",
        "g8 predict --clicks 5",
        "g28 keyxtend",
        "g14 keyxtend",
    ] {
        assert!(parse(&args(bad)).is_err(), "{bad} should fail");
    }
}

#[test]
fn the_help_names_every_gate_that_takes_only_a_name() {
    let help = crate::usage::usage();
    for (gate, ..) in GATES.iter().filter(|g| g.2.is_some()) {
        assert!(help.contains(&format!("kx-gates {gate} <")), "{gate}");
    }
}

#[test]
fn every_gate_holds_the_mouse_but_the_ones_that_need_the_owners_own_mouse() {
    for gate in ["g1", "g2", "g3", "g4", "g8", "g9", "g21", "g25", "g27"] {
        assert_eq!(mouse(gate, "x"), Mouse::Held, "{gate}");
    }
    for gate in ["g5", "g6", "g12", "g17", "g18"] {
        assert_eq!(
            mouse(gate, "x"),
            Mouse::Drives,
            "{gate} drives the test build"
        );
    }
    for gate in ["assist", "g13", "g23", "g26", "close", "g99"] {
        assert_eq!(mouse(gate, "x"), Mouse::Free, "{gate}");
    }
    let names: std::collections::BTreeSet<_> = GATES.iter().map(|g| g.0).collect();
    assert_eq!(names.len(), GATES.len(), "each gate once");
}

#[test]
fn g8s_hook_names_drive_the_test_build_and_its_other_names_stay_held() {
    for name in [g8hook::HOOK, g8hook::HOOK_WEAK] {
        assert_eq!(mouse("g8", name), Mouse::Drives, "g8 {name}");
        assert_eq!(mouse("g9", name), Mouse::Held, "only G8's");
    }
    assert_eq!(mouse("g8", crate::product::NAME), Mouse::Held);
    assert_eq!(mouse("g8", crate::g8::WEAK), Mouse::Held);
    assert_eq!(mouse("g8", crate::g8lang::ARABIC), Mouse::Held);
    assert_eq!(mouse("g8", crate::g8predict::PREDICT), Mouse::Held);
}
