//! The ping rules on examples and over random asks, answers and times; no test waits.

use super::*;
use proptest::collection::vec;
use proptest::prelude::*;

/// The window pinged.
const A: u64 = 1;
/// Another window.
const B: u64 = 2;
/// How long a ping may stay out: the `[scroll] check_ms` default.
const WAIT: Duration = Duration::from_millis(250);
/// A millisecond.
const MS: Duration = Duration::from_millis(1);

#[test]
fn one_ping_goes_at_a_time() {
    let (mut p, now) = (Pings::default(), Instant::now());
    assert!(p.ask(A, now));
    assert!(p.out());
    assert!(!p.ask(B, now), "one is out");
    p.done(true);
    assert!(!p.out());
    assert!(p.ask(B, now), "free again");
}

#[test]
fn a_ping_out_past_the_wait_makes_only_its_window_busy() {
    let (mut p, t0) = (Pings::default(), Instant::now());
    p.ask(A, t0);
    assert!(!p.busy(A, t0 + WAIT, WAIT), "still in time");
    assert!(p.busy(A, t0 + WAIT + MS, WAIT));
    assert!(!p.busy(B, t0 + WAIT + MS, WAIT));
}

#[test]
fn an_unanswered_ping_keeps_its_window_busy_until_one_is_answered() {
    let (mut p, t0) = (Pings::default(), Instant::now());
    p.ask(A, t0);
    p.done(false);
    assert!(p.busy(A, t0, WAIT) && !p.busy(B, t0, WAIT));
    p.ask(A, t0 + MS);
    assert!(p.busy(A, t0 + MS, WAIT), "still owed");
    p.done(true);
    assert!(!p.busy(A, t0 + WAIT * 4, WAIT));
}

#[test]
fn a_ping_ends_once_and_an_end_with_none_out_changes_nothing() {
    let (mut p, t0) = (Pings::default(), Instant::now());
    p.done(false);
    assert!(!p.busy(A, t0, WAIT));
    p.ask(A, t0);
    p.done(true);
    p.done(false);
    assert!(!p.busy(A, t0, WAIT), "the late end had no ping");
}

/// One step of a ping's life: ask a window, end the ping, or let time pass.
#[derive(Clone, Copy, Debug)]
enum Op {
    Ask(u64),
    Done(bool),
    Wait(u64),
}

/// Any step, over three windows and up to twice the wait.
fn op() -> impl Strategy<Value = Op> {
    let wait = u64::try_from(WAIT.as_millis()).unwrap();
    prop_oneof![
        (1..=3_u64).prop_map(Op::Ask),
        any::<bool>().prop_map(Op::Done),
        (0..=wait * 2).prop_map(Op::Wait),
    ]
}

proptest! {
    /// A window never asked is never busy, an ask goes only with none out, and an answered ping frees its window.
    #[test]
    fn only_a_window_whose_ping_failed_or_is_late_is_busy(ops in vec(op(), 0..64)) {
        let (mut p, mut now) = (Pings::default(), Instant::now());
        let mut asked = Vec::new();
        for op in ops {
            match op {
                Op::Ask(w) => {
                    let was_out = p.out();
                    prop_assert_eq!(p.ask(w, now), !was_out);
                    if !was_out { asked.push(w); }
                }
                Op::Done(answered) => {
                    let last = asked.last().copied().filter(|_| p.out());
                    p.done(answered);
                    if let Some(w) = last.filter(|_| answered) {
                        prop_assert!(!p.busy(w, now, WAIT));
                    }
                }
                Op::Wait(ms) => now += Duration::from_millis(ms),
            }
            for w in 1..=3_u64 {
                prop_assert!(asked.contains(&w) || !p.busy(w, now, WAIT));
            }
        }
    }
}
