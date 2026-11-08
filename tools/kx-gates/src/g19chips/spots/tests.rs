//! Tests of the place cases' sums and decisions: the floor, the lift, the second screen, and which window may move.

use super::*;

fn r(x: i32, y: i32, w: i32, h: i32) -> Rect {
    Rect { x, y, w, h }
}

#[test]
fn the_floor_is_the_plate_under_the_line_else_the_screens_bottom() {
    let (work, line) = (r(0, 0, 1920, 1032), r(100, 200, 400, 24));
    assert_eq!(floor(work, &[], line), 1032);
    let plate = r(0, 700, 1920, 332);
    assert_eq!(floor(work, &[plate], line), 700);
    let aside = r(1000, 700, 900, 332);
    assert_eq!(floor(work, &[aside], line), 1032, "not under the line");
    let above = r(0, 0, 1920, 100);
    assert_eq!(floor(work, &[above], line), 1032, "over the line");
}

#[test]
fn the_line_moves_to_sit_near_px_above_the_floor() {
    let line = r(100, 200, 400, 24);
    assert_eq!(lift(line, 700, 4), 700 - 4 - 224);
    assert_eq!(lift(line, 210, 4), -18, "up when it sits too low");
}

#[test]
fn only_a_window_new_since_the_gate_started_it_is_its_own() {
    let before: HashSet<isize> = [7, 9].into_iter().collect();
    assert!(own(&before, 12), "a new window");
    assert!(!own(&before, 9), "a window of the owner's");
}

#[test]
fn a_case_that_did_not_run_says_why_and_holds_no_verdict() {
    let v = unrun("one screen");
    assert_eq!(
        (&v[EXERCISED], &v["reason"]),
        (&json!(false), &json!("one screen"))
    );
    assert!(v.get("ok").is_none(), "never a pass");
}

#[test]
fn unread_rows_fail_the_near_bottom_case_and_never_fall_back_to_another_window() {
    let (near, second) = on_rows(Err::<(), _>("no box".into()), |()| -> (Value, Value) {
        panic!("no rows to run on")
    });
    assert_eq!(
        (&near["ok"], &near["error"]),
        (&json!(false), &json!("no box"))
    );
    assert_eq!(second["error"], "no box");
    assert!(near.get(EXERCISED).is_none() && second.get(EXERCISED).is_none());
    let ran = on_rows(Ok(3), |n| (json!({ "rows": n }), json!({})));
    assert_eq!(ran.0["rows"], 3, "read rows run the cases");
}

#[test]
fn a_window_not_the_gates_own_or_maximised_stays_and_says_why() {
    assert_eq!(stays(true, false), None);
    let not_ours = stays(false, false).expect("stays");
    assert!(not_ours.contains("not the gate's own"), "{not_ours}");
    assert_eq!(stays(false, true), Some(not_ours), "not ours comes first");
    assert!(stays(true, true).is_some_and(|w| w.contains("maximised")));
}

#[test]
fn only_our_own_new_window_with_our_tab_in_front_moves() {
    use crate::ownkeys::fake::{Fake, OUR_TAB, OWNER_TAB, hwnd, notepad};
    let app = notepad(None);
    let ours = Fake::new(app.hwnd, &[OUR_TAB]);
    assert_eq!(movable(&ours, &app, (true, false)), Ok(None));
    let theirs = movable(&ours, &app, (false, false)).expect("a reason");
    assert!(theirs.is_some_and(|w| w.contains("not the gate's own")));
    assert!(movable(&ours, &app, (true, true)).is_ok_and(|w| w.is_some()));
    let tab = Fake::new(app.hwnd, &[OWNER_TAB]);
    assert!(
        movable(&tab, &app, (true, false)).is_err(),
        "the owner's tab"
    );
    let away = Fake::new(hwnd(9), &[OUR_TAB]);
    assert!(movable(&away, &app, (true, false)).is_err(), "not in front");
}

#[test]
fn the_second_screen_is_the_first_area_away_from_the_window() {
    let one = [r(0, 0, 1920, 1032)];
    assert_eq!(other_screen(&one, Point { x: 10, y: 10 }), None);
    let two = [r(0, 0, 1920, 1032), r(1920, 0, 1280, 984)];
    assert_eq!(other_screen(&two, Point { x: 10, y: 10 }), Some(two[1]));
    assert_eq!(other_screen(&two, Point { x: 2000, y: 10 }), Some(two[0]));
}
