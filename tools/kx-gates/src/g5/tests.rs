//! Tests of `g5`: the hook's verdict, the rules a screen may break, and the result line.

use super::*;

/// Cases one run plays.
const PLAYED: usize = g5play::CASES;

fn g5() -> G5 {
    crate::config::load().expect("kx-gates.toml loads").g5
}

fn line(events: u64, reinstalls: u32, send_errors: u32) -> HookLine {
    HookLine {
        events,
        reinstalls,
        send_errors,
        ..HookLine::default()
    }
}

#[test]
fn the_hook_passes_with_a_line_no_reinstall_no_send_error_and_enough_events() {
    let per = g5().min_events_per_case;
    let enough = per * PLAYED as u64;
    assert!(hook_ok(&[line(enough, 0, 0)], PLAYED, per));
    let halves = [line(enough / 2, 0, 0), line(enough - enough / 2, 0, 0)];
    assert!(hook_ok(&halves, PLAYED, per), "the lines add up");
    assert!(!hook_ok(&[], PLAYED, per), "no line");
    assert!(!hook_ok(&[line(enough, 1, 0)], PLAYED, per), "a reinstall");
    assert!(!hook_ok(&[line(enough, 0, 1)], PLAYED, per), "a send error");
    assert!(!hook_ok(&[line(enough - 1, 0, 0)], PLAYED, per), "too few");
}

/// A hold of `ms` with the default still radius at 100 %.
fn held(ms: u64) -> Hold {
    Hold { ms, radius_px: 7 }
}

#[test]
fn a_hold_the_guard_would_take_for_a_lost_hook_is_refused() {
    let g = g5();
    let (alive, cap) = (2_000, g.hang_ms);
    let e = refuse(held(alive), alive, &g, cap).expect_err("as long as the alive watch");
    assert!(e.contains("alive_ms"), "{e}");
    assert!(refuse(held(alive + 1), alive, &g, cap).is_err());
    assert_eq!(refuse(held(alive - 1), alive, &g, cap), Ok(()));
}

#[test]
fn a_hang_past_target_windows_cap_or_a_lead_past_the_hold_is_refused() {
    let g = g5();
    let alive = 2_000;
    let e = refuse(held(1_500), alive, &g, g.hang_ms - 1).expect_err("past the cap");
    assert!(e.contains("hang_max_ms"), "{e}");
    let e = refuse(held(g.hang_lead_ms), alive, &g, g.hang_ms).expect_err("lead past the hold");
    assert!(e.contains("hang_lead_ms"), "{e}");
}

#[test]
fn a_drag_that_stays_within_the_radius_and_its_margin_is_refused() {
    let g = g5();
    let (alive, cap) = (2_000, g.hang_ms);
    let wide = Hold {
        ms: 1_500,
        radius_px: g.drag_px - g.still_margin_px,
    };
    let e = refuse(wide, alive, &g, cap).expect_err("the drag would stay still");
    assert!(e.contains("drag_px"), "{e}");
    let fits = Hold {
        radius_px: wide.radius_px - 1,
        ..wide
    };
    assert_eq!(refuse(fits, alive, &g, cap), Ok(()));
}

#[test]
fn a_failed_play_and_a_failed_switch_off_both_show() {
    let (play, off) = (
        || Err::<u8, _>("play".to_string()),
        || Err("off".to_string()),
    );
    assert_eq!(both(Ok(1), Ok(())), Ok(1));
    assert_eq!(both(play(), Ok(())), play());
    assert_eq!(both(Ok(1), off()), Err("off".to_string()));
    assert_eq!(both(play(), off()), Err("play; off".to_string()));
}

/// A run where every case played matched.
fn matched() -> Run {
    Run {
        hold: Hold {
            ms: 1_500,
            radius_px: 9,
        },
        still_px: 7,
        played: Played {
            lines: Vec::new(),
            ok: PLAYED,
            count: PLAYED,
        },
        left_open: Vec::new(),
    }
}

/// How a test build ended that quit or not, with `lines`.
fn ended(quit: bool, lines: Result<Vec<HookLine>, String>) -> Closed {
    Closed {
        quit,
        owner_back: None,
        lines,
    }
}

#[test]
fn admin_is_the_owners_manual_check_and_every_plain_result_says_so() {
    assert_eq!(target_of(PLAIN), Ok(()));
    assert_eq!(target_of(ADMIN), Err(ADMIN_NOTE.to_string()));
    assert!(target_of("x").is_err());
    let (per, run) = (g5().min_events_per_case, matched());
    let closed = |lines| ended(true, lines);
    let good = closed(Ok(vec![line(per * PLAYED as u64, 0, 0)]));
    let v = result(&run, &good, per);
    assert_eq!(v["admin"].as_str(), Some(ADMIN_NOTE));
    assert_eq!(
        (v["pass"].as_bool(), v["hook_lines"].as_u64()),
        (Some(true), Some(1))
    );
    assert_eq!(v["test_build"]["quit"].as_bool(), Some(true));
    let unread = result(&run, &closed(Err("no log".into())), per);
    assert_eq!(unread["pass"].as_bool(), Some(false), "no line, no pass");
    assert_eq!(unread["admin"].as_str(), Some(ADMIN_NOTE));
}

#[test]
fn a_test_build_that_did_not_quit_fails_g5() {
    let per = g5().min_events_per_case;
    let lines = || Ok(vec![line(per * PLAYED as u64, 0, 0)]);
    assert_eq!(
        result(&matched(), &ended(true, lines()), per)["pass"],
        json!(true)
    );
    let hung = result(&matched(), &ended(false, lines()), per);
    assert_eq!(hung["pass"], json!(false), "a quit that hung");
}
