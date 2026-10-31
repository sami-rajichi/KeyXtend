//! Tests of G10's key plan, over the product's own board.

use kx_mod_keyboard::{LangKey, Latch, LatchRole, Role};
use kx_module_api::ports::Scan;

use super::*;
use crate::board;

mod unlatch;

/// The key pressed after each modifier, as the shipped settings name it.
const PROBE: &str = "enter";

/// The shipped board's keys.
fn roles() -> Vec<(String, Role, Option<Scan>)> {
    board::roles().expect("the board reads")
}

/// The steps for the shipped board, with modifiers that let go (`stay` false) or stay.
fn steps(stay: bool) -> Vec<Step> {
    plan(&roles(), PROBE, stay).expect("enter is a send key")
}

/// The step for key `id`.
fn step_of(stay: bool, id: &str) -> Step {
    let all = steps(stay);
    all.into_iter()
        .find(|s| s.key == id)
        .expect("a step for the key")
}

/// `ids` as owned strings.
fn ids(ids: &[&str]) -> Vec<String> {
    ids.iter().map(ToString::to_string).collect()
}

#[test]
fn every_key_but_the_character_keys_the_language_key_and_the_power_key_gets_a_step() {
    let want: Vec<String> = roles()
        .into_iter()
        .filter(|(_, r, _)| !matches!(r, Role::Char | Role::Lang | Role::Power))
        .map(|(id, ..)| id)
        .collect();
    let got: Vec<String> = steps(false).into_iter().map(|s| s.key).collect();
    assert_eq!(got, want);
    assert!(got.contains(&"shortcuts".to_string()) && got.contains(&"right".to_string()));
    assert!(
        !got.contains(&"power".to_string()),
        "it would open the shut-down dialog"
    );
}

#[test]
fn the_shortcuts_key_shows_the_layer_and_hides_it_again_sending_nothing() {
    let layer = step_of(false, "shortcuts");
    assert_eq!(layer.presses, ids(&["shortcuts", "shortcuts"]));
    assert_eq!(layer.expect, Expect::Nothing);
    assert!(layer.puts_back && !layer.toggles && layer.spare.is_none());
}

#[test]
fn a_send_key_must_send_its_own_scan_code() {
    let esc = step_of(false, "esc");
    assert_eq!(esc.presses, ids(&["esc"]));
    assert_eq!(esc.expect, Expect::Keys(vec![0x01]));
    assert!(!esc.toggles);
    assert_eq!(step_of(false, "up").expect, Expect::Keys(vec![0xE048]));
}

#[test]
fn caps_lock_is_pressed_twice_so_its_state_comes_back() {
    let caps = step_of(false, "caps");
    assert_eq!(caps.presses, ids(&["caps", "caps"]));
    assert_eq!(caps.expect, Expect::Keys(vec![0x3A, 0x3A]));
}

#[test]
fn a_latch_goes_out_with_the_probe_and_is_let_go_again_when_modifiers_stay() {
    let shift = step_of(false, "shift");
    assert_eq!(shift.presses, ids(&["shift", PROBE]));
    assert_eq!(shift.expect, Expect::Keys(vec![0x2A, 0x1C]));
    assert!(shift.toggles);
    assert_eq!(
        step_of(true, "altgr").presses,
        ids(&["altgr", PROBE, "altgr"])
    );
    assert_eq!(
        step_of(true, "altgr").expect,
        Expect::Keys(vec![0xE038, 0x1C])
    );
}

#[test]
fn win_is_used_by_another_latch_before_it_lets_go_so_nothing_goes_down() {
    for stay in [false, true] {
        let win = step_of(stay, "win");
        assert_eq!(win.presses, ids(&["win", "shift", "win", "shift"]));
        assert_eq!(win.expect, Expect::Nothing, "neither a shortcut nor Start");
        assert!(win.toggles);
    }
}

#[test]
fn a_board_with_no_latch_but_win_has_no_plan() {
    let win = Role::Latch(LatchRole {
        lone: true,
        ..LatchRole::of(Latch::Win)
    });
    let roles = [
        ("win".to_string(), win, Some(Scan::new(0xE05B))),
        (PROBE.to_string(), Role::Send, Some(Scan::new(0x1C))),
    ];
    assert!(plan(&roles, PROBE, false).is_err());
}

#[test]
fn a_latch_a_failed_step_left_on_is_pressed_once_more() {
    let shift = step_of(false, "shift");
    assert!(shift.left_latched(Some(true)));
    assert!(!shift.left_latched(Some(false)), "already let go");
    assert!(!shift.left_latched(None), "unread: the owner checks it");
    assert!(
        !step_of(false, "win").left_latched(Some(true)),
        "Win lets go only by its undo, as a lone press opens Start"
    );
    assert!(
        !step_of(false, "esc").left_latched(Some(true)),
        "a send key latches nothing"
    );
}

#[test]
fn only_wins_step_has_a_spare_and_it_starts_only_with_the_spare_let_go() {
    for stay in [false, true] {
        let win = step_of(stay, "win");
        assert_eq!(win.spare.as_deref(), Some("shift"));
        assert_eq!(win.ready(Some(false)), Ok(()));
        assert!(win.ready(Some(true)).is_err(), "Win would go out alone");
        assert!(win.ready(None).is_err(), "unread: it may be on");
        let others = steps(stay).into_iter().filter(|s| s.key != "win");
        for s in others {
            assert_eq!(s.spare, None, "{}", s.key);
            assert_eq!(s.ready(Some(true)), Ok(()), "{}", s.key);
        }
    }
}

#[test]
fn the_owner_checks_the_spare_when_a_failed_step_may_have_left_it_on() {
    let win = step_of(false, "win");
    assert_eq!(win.spare_why(true, Some(true)), Some(("shift", MAY_BE_ON)));
    assert_eq!(win.spare_why(true, None), Some(("shift", MAY_BE_ON)));
    assert_eq!(win.spare_why(true, Some(false)), None, "let go");
    assert_eq!(win.spare_why(false, None), None, "passed");
    assert_eq!(step_of(false, "shift").spare_why(true, None), None);
}

#[test]
fn the_owner_checks_a_latch_that_never_read_or_that_a_failed_step_may_have_left_on() {
    let shift = step_of(false, "shift");
    assert_eq!(shift.owner_why(true, false, None), Some(NO_TOGGLE));
    assert_eq!(
        shift.owner_why(false, true, None),
        Some(MAY_BE_ON),
        "failed, and unread after"
    );
    assert_eq!(
        shift.owner_why(false, true, Some(true)),
        Some(MAY_BE_ON),
        "failed, and still on after"
    );
    assert_eq!(shift.owner_why(false, true, Some(false)), None, "let go");
    assert_eq!(shift.owner_why(false, false, None), None, "passed");
    assert_eq!(
        step_of(false, "esc").owner_why(false, true, None),
        None,
        "a send key latches nothing"
    );
}

#[test]
fn a_key_of_a_later_version_must_send_nothing() {
    let mic = step_of(false, "mic");
    assert_eq!((mic.presses, mic.expect), (ids(&["mic"]), Expect::Nothing));
}

#[test]
fn a_mouse_assist_key_is_pressed_twice_so_its_mode_comes_back_and_must_send_nothing() {
    for id in ["rclick", "grab"] {
        let s = step_of(false, id);
        let want = (ids(&[id, id]), Expect::Nothing, false);
        assert_eq!((s.presses, s.expect, s.toggles), want, "{id}");
    }
}

#[test]
fn the_clipboard_key_opens_the_panel_and_closes_it_again_sending_nothing() {
    let s = step_of(false, "clip");
    let want = (ids(&["clip", "clip"]), Expect::Nothing, false);
    assert_eq!((s.presses, s.expect, s.toggles), want);
}

#[test]
fn only_the_keys_pressed_twice_put_their_mode_back_even_after_the_guard_stops() {
    for id in ["rclick", "grab", "shortcuts", "clip"] {
        assert!(step_of(false, id).puts_back, "{id}");
    }
    for id in ["esc", "caps", "shift", "mic"] {
        assert!(!step_of(true, id).puts_back, "{id}");
    }
}

#[test]
fn the_probe_must_be_a_send_key_of_the_board() {
    assert_eq!(probe_scan(&roles(), PROBE), Ok(Scan::new(0x1C)));
    assert!(
        plan(&roles(), "shift", false).is_err(),
        "a latch is no probe"
    );
    assert!(plan(&roles(), "nope", false).is_err());
}

#[test]
fn expected_keys_must_go_down_in_order_with_others_between_allowed() {
    let altgr = Expect::Keys(vec![0xE038, 0x1C]);
    assert!(
        altgr.met(&[0x1D, 0xE038, 0x1C]),
        "Windows adds a Ctrl before AltGr"
    );
    assert!(!altgr.met(&[0x1C, 0xE038]), "out of order");
    assert!(!altgr.met(&[0xE038]), "the probe never came");
    assert!(Expect::Nothing.met(&[]));
    assert!(!Expect::Nothing.met(&[0x01]));
}

#[test]
fn right_shift_matches_as_windows_reports_it_and_left_shift_stays_plain() {
    let rshift = step_of(false, "rshift").expect;
    assert_eq!(rshift, Expect::Keys(vec![0x36, 0x1C]), "the board's code");
    assert!(rshift.met(&[0xE036, 0x1C]), "Windows marks it extended");
    let lshift = step_of(false, "shift").expect;
    assert!(lshift.met(&[0x2A, 0x1C]));
    assert!(!lshift.met(&[0xE02A, 0x1C]), "left Shift stays plain");
}

#[test]
fn the_language_key_steps_opens_its_list_or_does_nothing_with_one_layout() {
    assert_eq!(lang_want(LangKey::Carousel, 2), LangWant::Steps);
    assert_eq!(lang_want(LangKey::Cycle, 3), LangWant::Steps);
    assert_eq!(lang_want(LangKey::List, 2), LangWant::Opens);
    assert_eq!(lang_want(LangKey::List, 1), LangWant::Nothing);
    assert_eq!(lang_want(LangKey::Carousel, 0), LangWant::Nothing);
}
