//! Tests of `g5play`: the cases straddle the still radius at each DPI, and each case wants what target-window logs.

use super::*;
use kx_mod_mouse::MouseSettings;

const P: Pt = Pt { x: 100, y: 50 };
/// Screens at 100 %, 125 %, 150 %, 200 %, 300 %, 400 % and 500 %.
const DPIS: [u32; 7] = [96, 120, 144, 192, 288, 384, 480];

fn at(press: Press, x: i32, y: i32) -> Mouse {
    Mouse { press, us: 0, x, y }
}

fn g5() -> G5 {
    crate::config::load().expect("kx-gates.toml loads").g5
}

#[test]
fn a_drag_must_start_at_the_press_and_end_where_it_moved() {
    let got = [at(Press::LeftDown, 100, 50), at(Press::LeftUp, 130, 51)];
    assert!(case_ok(LEFT, false, &got, P, 30, 1));
    assert!(!case_ok(LEFT, false, &got, P, 0, 1));
    let late = [at(Press::LeftDown, 103, 50), at(Press::LeftUp, 130, 50)];
    assert!(!case_ok(LEFT, false, &late, P, 30, 1));
}

#[test]
fn a_hold_must_log_a_right_click_all_at_the_press() {
    let right = [
        at(Press::RightDown, 100, 50),
        at(Press::RightUp, 100, 50),
        at(Press::Menu, 100, 50),
    ];
    assert!(case_ok(RIGHT, true, &right, P, 6, 1));
    let moved = [right[0], at(Press::RightUp, 106, 50), right[2]];
    assert!(!case_ok(RIGHT, true, &moved, P, 6, 1));
    let also_left = [at(Press::LeftDown, 100, 50), right[0], right[1], right[2]];
    assert!(!case_ok(RIGHT, true, &also_left, P, 0, 1));
    assert!(!case_ok(RIGHT, true, &[], P, 0, 1));
}

#[test]
fn the_log_shows_offsets_from_the_test_point_never_a_screen_point() {
    let got = [at(Press::LeftDown, 100, 50), at(Press::LeftUp, 130, 49)];
    let names = (Press::LeftDown.name(), Press::LeftUp.name());
    assert_eq!(
        offsets(&got, P),
        [format!("{}+0,+0", names.0), format!("{}+30,-1", names.1)]
    );
}

#[test]
fn at_each_dpi_the_cases_straddle_the_physical_still_radius_by_more_than_the_slack() {
    let g = g5();
    let still = MouseSettings::from_defaults().expect("defaults").still_px;
    for dpi in DPIS {
        let r = radius(still, dpi);
        let [_, drag, _, wiggle, past, _] = cases(&g, r);
        assert!(
            r - wiggle.dx > g.point_slack_px,
            "{dpi}: the wiggle stays inside"
        );
        assert!(
            past.dx - r > g.point_slack_px,
            "{dpi}: the short move leaves"
        );
        assert!(wiggle.dx > 0 && drag.dx > past.dx, "{dpi}");
    }
}

#[test]
fn the_radius_scales_with_the_dpi_as_the_engine_measures_it() {
    assert_eq!(radius(7, BASE_DPI), 7);
    assert_eq!(radius(7, 144), 11, "10.5 rounds up");
    assert_eq!(radius(7, 0), radius(7, BASE_DPI), "no DPI reads as 100 %");
}

#[test]
fn the_hung_case_holds_still_and_wants_a_right_click() {
    let hung: Vec<Case> = cases(&g5(), 9).into_iter().filter(|c| c.hang).collect();
    let [case] = hung.as_slice() else {
        panic!("one hung case")
    };
    assert!(case.long && case.dx == 0 && case.want == RIGHT);
}

#[test]
fn the_cases_match_only_when_every_one_played_was_ok() {
    let played = |ok, count| Played {
        lines: Vec::new(),
        ok,
        count,
    };
    assert!(played(6, 6).matched());
    assert!(!played(5, 6).matched());
    assert!(played(0, 0).matched(), "nothing played, nothing failed");
}
