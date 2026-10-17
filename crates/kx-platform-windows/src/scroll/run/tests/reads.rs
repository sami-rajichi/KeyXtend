//! The run's reads on the fake desk: one a check, capped, never of a hung app, its box checked and kept while a read
//! misses, and the wheel settings read for each new target.

use super::*;

#[test]
fn one_read_a_check_serves_the_aim_and_its_reaches_along_one_axis() {
    let fake = desk();
    let mut r = run(&fake);
    let aim = r.aim().unwrap();
    assert_eq!(r.reach(&aim, Dir::Down), Reach::More);
    assert_eq!(r.reach(&aim, Dir::Up), Reach::More);
    assert_eq!(fake.with(|s| (s.reads.len(), s.clock_reads)), (1, 1));
    assert_eq!(r.reach(&aim, Dir::Left), Reach::End, "nothing left of 0 %");
    assert_eq!(r.reach(&aim, Dir::Right), Reach::More);
    let ways: Vec<Dir> = fake.with(|s| s.reads.iter().map(|r| r.1).collect());
    assert_eq!(ways, [Dir::Down, Dir::Left], "the other axis reads once");
    r.turn(&aim, Dir::Down, NOTCH_LINES).unwrap();
    r.reach(&aim, Dir::Down);
    let counts = fake.with(|s| (s.reads.len(), s.clock_reads));
    assert_eq!(counts, (3, 3), "a step moved it");
}

/// Over a wide table in a long page, down reads the page and sideways the table: each read looks along the latest way.
#[test]
fn each_read_looks_along_the_latest_ways_axis() {
    let fake = desk();
    let mut r = run(&fake);
    let first = r.aim().unwrap();
    r.turn(&first, Dir::Right, NOTCH_LINES).unwrap();
    let aim = r.aim().unwrap();
    r.reach(&aim, Dir::Left);
    let ways: Vec<Dir> = fake.with(|s| s.reads.iter().map(|r| r.1).collect());
    assert_eq!(
        ways,
        [Dir::Down, Dir::Right],
        "down at first, then the turn's way"
    );
}

#[test]
fn a_reach_on_an_older_target_reads_it_once() {
    let fake = desk();
    let mut r = run(&fake);
    let old = r.aim().unwrap();
    fake.with(|s| s.cursor = Some(Point { x: 1700, y: 200 }));
    assert!(r.aim().unwrap().ours);
    r.reach(&old, Dir::Down);
    r.reach(&old, Dir::Up);
    assert_eq!(fake.with(|s| s.reads.len()), 2, "one for each check");
}

#[test]
fn a_hung_target_is_never_read_and_keeps_its_last_box() {
    let fake = desk();
    let mut r = run(&fake);
    r.aim().unwrap();
    fake.with(|s| s.hung.push(APP_ID));
    let aim = r.aim().unwrap();
    assert_eq!(aim.area, Some(BOX), "the box from before");
    assert_eq!(r.reach(&aim, Dir::Down), Reach::Unknown);
    assert_eq!(fake.with(|s| s.reads.len()), 1);
}

#[test]
fn a_read_that_misses_keeps_only_the_same_targets_box() {
    let fake = desk();
    let mut r = run(&fake);
    r.aim().unwrap();
    fake.with(|s| s.read = None);
    assert_eq!(r.aim().unwrap().area, Some(BOX));
    fake.with(|s| s.cursor = Some(Point { x: 1200, y: 300 }));
    let other = r.aim().unwrap();
    assert_eq!((other.id, other.area), (TargetId::new(DESK_ID), None));
    assert_eq!(r.reach(&other, Dir::Down), Reach::Unknown);
}

#[test]
fn a_bad_box_or_nothing_that_scrolls_gives_no_box() {
    let fake = desk();
    let mut r = run(&fake);
    let huge = Read {
        area: Some([0, 0, i32::MAX, 10]),
        ..READ
    };
    fake.with(|s| s.read = Some(huge));
    assert_eq!(r.aim().unwrap().area, None);
    let none = Read {
        area: None,
        pair: None,
    };
    fake.with(|s| s.read = Some(none));
    let aim = r.aim().unwrap();
    assert_eq!((aim.area, r.reach(&aim, Dir::Down)), (None, Reach::Unknown));
}

#[test]
fn the_wheel_settings_are_read_at_open_and_for_each_new_target() {
    let fake = desk();
    let mut r = run(&fake);
    assert_eq!(fake.with(|s| s.wheel_reads), 1);
    let app = r.aim().unwrap();
    r.aim().unwrap();
    assert_eq!(fake.with(|s| s.wheel_reads), 2);
    fake.with(|s| s.wheel = Some(wheel(1, 1)));
    r.turn(&app, Dir::Down, 1.0).unwrap();
    fake.with(|s| s.cursor = Some(Point { x: 1200, y: 300 }));
    let desk_aim = r.aim().unwrap();
    r.turn(&desk_aim, Dir::Down, 1.0).unwrap();
    let (a_line, a_notch) = (
        -i32::try_from(NOTCH / THREE).unwrap(),
        -i32::try_from(NOTCH).unwrap(),
    );
    let sent: Vec<i32> = fake.with(|s| s.sent.iter().map(|t| t.1).collect());
    assert_eq!(
        sent,
        [a_line, a_notch],
        "the new setting from the new target on"
    );
    assert_eq!(fake.with(|s| s.wheel_reads), 3);
}

#[test]
fn a_read_shows_its_box_and_hides_it_in_debug() {
    let shown = format!("{READ:?}");
    assert!(!shown.contains("990") && shown.contains("50.0"), "{shown}");
}
