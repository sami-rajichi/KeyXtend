//! Tests of `g24prod`.

use serde_json::json;

use super::*;

#[test]
fn the_save_dialogs_leftover_and_memory_checks_find_the_worker_beside_a_config_made_program() {
    let cfg = crate::config::GatesConfig {
        dir: PathBuf::from(r"D:\p\tools\kx-gates"),
        ..crate::config::load().expect("loads")
    };
    let running = [(
        4,
        r"D:\p\target\gate-input\release\keyxtend-worker.exe".to_string(),
    )];
    let program = cfg.program("../../target/gate-input/release/keyxtend.exe");
    assert_eq!(workertok::beside(&running, &workers_dir(&program)), [4]);
    let raw = Path::new(r"D:\p\tools\kx-gates\../../target/gate-input/release/keyxtend.exe");
    let found = workertok::beside(&running, &workers_dir(raw));
    assert_eq!(found, [4], "a program given as it is, as G22's copy is");
}

#[test]
fn the_gate_passes_only_when_both_copies_did_and_nothing_stayed() {
    let (yes, no) = (json!({ "ok": true }), json!({ "ok": false }));
    let two = [yes.clone(), yes.clone()];
    assert!(passed(&two, Run::All, true, &[]));
    assert!(!passed(&[yes.clone(), no], Run::All, true, &[]));
    assert!(!passed(&two, Run::All, false, &[]), "the folder stayed");
    let one = std::slice::from_ref(&yes);
    assert!(!passed(one, Run::All, true, &[]), "a copy missing");
    assert!(passed(one, Run::Trap, true, &[]), "the trap's one copy");
    let left = ["target-window still runs".to_string()];
    assert!(!passed(&two, Run::All, true, &left), "a leftover");
    assert!(!passed(&two, Run::Trap, true, &[]), "a copy too many");
}

#[test]
fn the_trap_runs_only_where_the_keyboard_or_its_bubble_shows_over_a_capture() {
    assert_eq!(Run::All.ways(), MODES);
    assert_eq!(Run::Trap.ways(), [SnipKeyboard::Bubble]);
    assert_eq!(Run::Trap.names(SnipKeyboard::Bubble), trap::NAMES);
    assert_eq!(Run::All.names(SnipKeyboard::Hidden), copy::HIDDEN_PARTS);
    assert_eq!(
        (Run::All.name(), Run::Trap.name()),
        (NAME, crate::g24::TRAP)
    );
}

#[test]
fn a_worker_still_running_once_the_copies_ended_is_a_leftover_named_by_count_only() {
    assert_eq!(workers_left(&[]), None);
    let line = workers_left(&[7, 9]).expect("a leftover");
    assert!(line.contains("2 ") && !line.contains('7'), "{line}");
}

#[test]
fn a_copy_passes_when_each_judged_part_did_and_a_skipped_one_counts_for_neither() {
    let names = ["region", "window"];
    let skipped = json!({ "skipped": true, "why": "another scale" });
    let v = json!({ "region": skipped, "window": { "ok": true } });
    assert!(parts_ok(&v, &names));
    let v = json!({ "region": skipped, "window": { "ok": false } });
    assert!(!parts_ok(&v, &names));
    let v = json!({ "window": { "ok": true } });
    assert!(!parts_ok(&v, &names), "a missing part fails");
    assert_eq!(MODES.len(), 2, "a bubble copy, then a hidden one");
}
