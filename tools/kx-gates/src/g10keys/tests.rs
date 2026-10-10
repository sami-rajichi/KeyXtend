//! Tests of G10's key plan, over the product's own board.

use kx_mod_keyboard::{LangKey, Role};
use kx_module_api::ports::Scan;

use super::*;
use crate::board;

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
fn every_key_but_the_character_keys_and_the_language_key_gets_a_step() {
    let want: Vec<String> = roles()
        .into_iter()
        .filter(|(_, r, _)| !matches!(r, Role::Char | Role::Lang))
        .map(|(id, ..)| id)
        .collect();
    let got: Vec<String> = steps(false).into_iter().map(|s| s.key).collect();
    assert_eq!(got, want);
    assert!(got.contains(&"power".to_string()) && got.contains(&"right".to_string()));
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
fn win_is_latched_and_let_go_with_nothing_sent_since_its_shortcuts_reach_windows() {
    let win = step_of(false, "win");
    assert_eq!(win.presses, ids(&["win", "win"]));
    assert_eq!(win.expect, Expect::Nothing);
    assert!(win.toggles);
}

#[test]
fn a_key_of_a_later_version_must_send_nothing() {
    let mic = step_of(false, "mic");
    assert_eq!((mic.presses, mic.expect), (ids(&["mic"]), Expect::Nothing));
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
