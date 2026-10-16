//! The fake scroll target, held to the port: scripted aims and reaches, recorded turns, close on drop, checked lines.

use super::*;
use kx_module_api::ports::{Point, Rect};
use std::panic::{AssertUnwindSafe, catch_unwind};

/// Test values; the real ones come from the scroll module's settings.
const CONFIG: ScrollConfig = ScrollConfig {
    check_ms: 1,
    read_ms: 2,
    edge_px: 3,
};

const A: Aim = Aim {
    at: Point { x: 4, y: 5 },
    on_pointer: true,
    area: None,
    ours: false,
    id: TargetId::new(1),
};

const B: Aim = Aim {
    at: Point { x: 6, y: 7 },
    on_pointer: false,
    area: Some(Rect {
        x: 0,
        y: 0,
        w: 8,
        h: 9,
    }),
    ours: true,
    id: TargetId::new(2),
};

/// The message `run` panics with.
fn panic_text(run: impl FnOnce()) -> String {
    let hit = catch_unwind(AssertUnwindSafe(run)).expect_err("it should panic");
    let text = hit.downcast_ref::<String>().cloned();
    text.or_else(|| hit.downcast_ref::<&str>().map(|s| (*s).to_owned()))
        .unwrap_or_default()
}

#[test]
fn with_nothing_scripted_aim_finds_no_target() {
    let target = FakeScrollTarget::new();
    let mut run = target.open(CONFIG).unwrap();
    assert_eq!(run.aim(), Err(ScrollError::NoTarget));
}

#[test]
fn scripted_aims_come_back_in_order_and_the_last_repeats() {
    let target = FakeScrollTarget::new();
    target.push_aim(Ok(A));
    target.push_aim(Ok(B));
    let mut run = target.open(CONFIG).unwrap();
    assert_eq!([run.aim(), run.aim(), run.aim()], [Ok(A), Ok(B), Ok(B)]);
    target.push_aim(Err(ScrollError::Gone));
    let gone = Err(ScrollError::Gone);
    assert_eq!([run.aim(), run.aim()], [gone, gone], "an error repeats too");
}

#[test]
fn reaches_are_unknown_until_set_and_a_target_one_wins() {
    let target = FakeScrollTarget::new();
    let mut run = target.open(CONFIG).unwrap();
    assert!(Dir::ALL.iter().all(|&d| run.reach(&A, d) == Reach::Unknown));
    target.set_reach(Dir::Down, Reach::More);
    target.set_reach_for(B.id, Dir::Down, Reach::End);
    assert_eq!(run.reach(&A, Dir::Down), Reach::More);
    assert_eq!(run.reach(&B, Dir::Down), Reach::End);
    assert_eq!(run.reach(&B, Dir::Up), Reach::Unknown);
    target.set_reach(Dir::Down, Reach::End);
    assert_eq!(run.reach(&A, Dir::Down), Reach::End, "the latest set wins");
}

#[test]
fn turns_are_recorded_in_order_and_taken_once() {
    let target = FakeScrollTarget::new();
    let mut run = target.open(CONFIG).unwrap();
    run.turn(&A, Dir::Down, 0.25).unwrap();
    run.turn(&B, Dir::Left, 0.0).unwrap();
    let want = [
        Turn {
            id: A.id,
            dir: Dir::Down,
            lines: 0.25,
        },
        Turn {
            id: B.id,
            dir: Dir::Left,
            lines: 0.0,
        },
    ];
    assert_eq!(target.turns(), want);
    assert_eq!(target.take_turns(), want);
    assert!(target.turns().is_empty(), "taken");
}

#[test]
fn a_turn_error_refuses_turns_until_cleared() {
    let target = FakeScrollTarget::new();
    let mut run = target.open(CONFIG).unwrap();
    target.set_turn_error(Some(ScrollError::Gone));
    assert_eq!(run.turn(&A, Dir::Up, 1.0), Err(ScrollError::Gone));
    assert!(target.turns().is_empty(), "a refused turn is not recorded");
    target.set_turn_error(None);
    assert_eq!(run.turn(&A, Dir::Up, 1.0), Ok(()));
    assert_eq!(target.turns().len(), 1);
}

#[test]
fn a_dropped_run_closes_and_a_clone_sees_it() {
    let target = FakeScrollTarget::new();
    let handle = target.clone();
    assert_eq!((handle.running(), handle.config()), (false, None));
    let run = target.open(CONFIG).unwrap();
    assert_eq!(
        (handle.running(), handle.opens(), handle.closes()),
        (true, 1, 0)
    );
    assert_eq!(handle.config(), Some(CONFIG));
    drop(run);
    assert_eq!(
        (handle.running(), handle.opens(), handle.closes()),
        (false, 1, 1)
    );
    assert_eq!(handle.config(), Some(CONFIG), "kept after the close");
}

#[test]
fn it_runs_while_any_run_is_held() {
    let target = FakeScrollTarget::new();
    let first = target.open(CONFIG).unwrap();
    let second = target.open(CONFIG).unwrap();
    drop(first);
    assert!(target.running(), "the second still runs");
    drop(second);
    assert_eq!((target.running(), target.closes()), (false, 2));
}

#[test]
fn fail_next_refuses_exactly_one_open() {
    let target = FakeScrollTarget::new();
    target.fail_next(ScrollError::Refused);
    assert_eq!(target.open(CONFIG).err(), Some(ScrollError::Refused));
    let after = (target.opens(), target.running(), target.config());
    assert_eq!(after, (0, false, None), "a refusal counts nothing");
    assert!(target.open(CONFIG).is_ok(), "only one refusal");
    assert_eq!(target.opens(), 1);
}

#[test]
fn a_turn_by_lines_not_finite_or_below_zero_panics() {
    let target = FakeScrollTarget::new();
    let mut run = target.open(CONFIG).unwrap();
    for lines in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -0.5] {
        let text = panic_text(|| {
            let _ = run.turn(&A, Dir::Down, lines);
        });
        assert!(text.contains(BAD_LINES), "{lines}: {text}");
    }
    assert!(target.turns().is_empty(), "nothing recorded");
    drop(run);
    assert_eq!(target.closes(), 1, "the run still closes");
}
