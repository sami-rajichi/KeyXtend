//! What a run holds back: a window that owes a ping, a wheel Windows gives the focus, steps not taken, and another
//! app's mark.

use super::*;

/// One notch down.
fn notch_down() -> i32 {
    -i32::try_from(NOTCH).unwrap()
}

#[test]
fn each_aim_at_another_apps_window_pings_it_and_ours_never() {
    let fake = desk();
    let mut r = run(&fake);
    r.aim().unwrap();
    r.aim().unwrap();
    fake.with(|s| s.cursor = Some(Point { x: 1700, y: 200 }));
    r.aim().unwrap();
    assert_eq!(fake.with(|s| s.pings.clone()), [APP_ID, APP_ID]);
}

#[test]
fn reach_and_each_turn_ping_the_last_target_with_the_pointer_on_our_list() {
    let fake = desk();
    let mut r = run(&fake);
    let aim = r.aim().unwrap();
    fake.with(|s| s.cursor = Some(Point { x: 1700, y: 200 }));
    r.reach(&aim, Dir::Down);
    r.turn(&aim, Dir::Down, NOTCH_LINES).unwrap();
    fake.with(|s| s.owing.push(APP_ID));
    r.turn(&aim, Dir::Down, NOTCH_LINES).unwrap();
    let pings = fake.with(|s| s.pings.clone());
    assert_eq!(pings, [APP_ID; 4], "a busy target is asked again");
    assert_eq!(fake.with(|s| s.posted.len()), 1, "the busy spell is seen");
}

#[test]
fn a_window_that_owes_a_ping_gets_no_turn_or_read_and_its_course_waits() {
    let fake = desk();
    let mut r = run(&fake);
    r.aim().unwrap();
    fake.with(|s| s.owing.push(APP_ID));
    let aim = r.aim().unwrap();
    assert_eq!(aim.area, Some(BOX), "the last box stays");
    for _ in 0..5 {
        assert_eq!(r.turn(&aim, Dir::Down, NOTCH_LINES), Ok(()));
    }
    assert_eq!(r.reach(&aim, Dir::Down), Reach::Unknown);
    fake.with(|s| {
        assert_eq!(
            (s.reads.len(), s.clock_reads),
            (1, 1),
            "only the first aim read"
        );
        assert!(s.sent.is_empty() && s.posted.is_empty() && s.steps.is_empty());
        s.owing.clear();
    });
    r.turn(&aim, Dir::Down, NOTCH_LINES).unwrap();
    let sent = fake.with(|s| s.sent.clone());
    assert_eq!(
        sent,
        [(Dir::Down, notch_down())],
        "nothing owed from before"
    );
}

#[test]
fn a_wheel_windows_gives_the_focus_is_posted_even_with_the_pointer_on_the_target() {
    let fake = desk();
    let focus = Wheel {
        at_pointer: false,
        ..wheel(THREE, THREE)
    };
    fake.with(|s| s.wheel = Some(focus));
    let mut r = run(&fake);
    let aim = r.aim().unwrap();
    r.turn(&aim, Dir::Down, NOTCH_LINES).unwrap();
    fake.with(|s| {
        assert!(s.sent.is_empty());
        assert_eq!(s.posted, [(APP_ID, ON_APP, Dir::Down, notch_down())]);
    });
}

#[test]
fn steps_ui_automation_did_not_take_are_refused() {
    let fake = desk();
    fake.with(|s| (s.wheel, s.steps_refused) = (Some(wheel(0, 0)), true));
    let mut r = run(&fake);
    let aim = r.aim().unwrap();
    assert_eq!(r.turn(&aim, Dir::Down, 2.0), Err(ScrollError::Refused));
}

#[test]
fn only_our_own_watched_window_is_the_keyboard() {
    assert_eq!(Whose::of(true, true), Whose::Keyboard);
    assert_eq!(Whose::of(true, false), Whose::Ours);
    assert_eq!(Whose::of(false, true), Whose::Theirs, "another app's mark");
    assert_eq!(Whose::of(false, false), Whose::Theirs);
}
